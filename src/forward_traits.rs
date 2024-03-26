use syn::{Generics, Token, parse_quote, parse};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result, Error};
use syn::fold::Fold;
use syn_derive::Parse;
use quote::{quote, ToTokens};

use crate::generics::combine_generics;
use crate::partial_eval::get_evaluator;
use crate::mangle::mangle_generics;

use crate::base_transform_info::BaseTransformInfo;
use crate::additional_transform_infos::AdditionalTransformInfos;
use crate::forwarded_trait_info::ForwardedTraitInfo;
use crate::type_def_info::TypeDefInfo;
use crate::trait_def_info::TraitDefInfo;

use crate::transformer::Transformer;

struct TypeTransformInfo
{
	for_token: Token! [for],
	generics: Generics,
	base_transform_info: BaseTransformInfo,
	additional_transform_infos: AdditionalTransformInfos
}

impl Parse for TypeTransformInfo
{
	fn parse (input: ParseStream) -> Result <Self>
	{
		let for_token = input . parse ()?;

		let mut generics: Generics = input . parse ()?;

		let base_transform_info = input . parse ()?;

		let additional_transform_infos = input . parse ()?;

		generics . where_clause = input . parse ()?;

		let type_transform_info = Self
		{
			for_token,
			generics,
			base_transform_info,
			additional_transform_infos
		};

		Ok (type_transform_info)
	}
}

impl ToTokens for TypeTransformInfo
{
	fn to_tokens (&self, tokens: &mut proc_macro2::TokenStream)
	{
		self . for_token . to_tokens (tokens);
		self . generics . to_tokens (tokens);
		self . base_transform_info . to_tokens (tokens);
		self . additional_transform_infos . to_tokens (tokens);
		self . generics . where_clause . to_tokens (tokens);
	}
}

#[allow (dead_code)]
#[derive (Parse)]
struct ForwardTraits
{
	type_transform_info: TypeTransformInfo,
	impl_token: Token! [impl],

	#[parse (Punctuated::parse_separated_nonempty)]
	forwarded_traits: Punctuated <ForwardedTraitInfo, Token! [+]>
}

fn try_forward_traits_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraits
	{
		type_transform_info,
		forwarded_traits,
		..
	}
		= parse (input)?;

	let base_type_macro_ident =
		type_transform_info . base_transform_info . get_type_macro_ident ();

	let mut tokens = proc_macro2::TokenStream::new ();

	for forwarded_trait_info in forwarded_traits
	{
		let forwarded_trait_macro_path =
			forwarded_trait_info . get_macro_path ()?;

		quote!
		{
			#base_type_macro_ident!
			(
				#forwarded_trait_macro_path,
				forward_traits::__forward_trait,
				#type_transform_info impl #forwarded_trait_info
			);
		}
			. to_tokens (&mut tokens);
	}

	Ok (tokens)
}

pub fn forward_traits_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_traits_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}

#[allow (dead_code)]
#[derive (Parse)]
struct ForwardTrait
{
	type_transform_info: TypeTransformInfo,
	impl_token: Token! [impl],
	forwarded_trait_info: ForwardedTraitInfo,
	comma_token_0: Token! [,],

	type_def_info: TypeDefInfo,
	comma_token_1: Token! [,],

	trait_def_info: TraitDefInfo
}

fn try_forward_trait_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTrait
	{
		type_transform_info,
		forwarded_trait_info,
		type_def_info,
		trait_def_info,
		..
	}
		= parse (input)?;

	let generics = combine_generics
	([
		type_def_info . generics . clone (),
		type_transform_info . generics,
		forwarded_trait_info . generics
	]);

	let (mut generics, mut mangler) = mangle_generics (generics);

	let forwarded_trait =
		mangler . fold_path (forwarded_trait_info . trait_path);

	let mut transformer = Transformer::new ();

	let (base_type, delegated_type, base_value_transformer) =
		type_transform_info . base_transform_info . into_value_transformer
		(
			&type_def_info . generics,
			&type_def_info . fields
		)?;

	let base_type = mangler . fold_type (base_type);
	let delegated_type = mangler . fold_type (delegated_type);

	transformer . add_transformation
	(
		parse_quote! (Self),
		delegated_type . clone (),
		base_value_transformer
	);

	for additional_transform_info
	in type_transform_info . additional_transform_infos
	{
		let (from_type, to_type, value_transformer) =
			additional_transform_info . into_value_transformer ();

		let from_type = mangler . fold_type (from_type);
		let to_type = mangler . fold_type (to_type);

		transformer . add_transformation
		(
			from_type,
			to_type,
			value_transformer
		);
	}

	let mut evaluator =
		get_evaluator (trait_def_info . generics, &forwarded_trait)?;

	let mut items = Vec::new ();

	for item
	in trait_def_info
		. items
		. into_iter ()
		. map (|item| evaluator . fold_trait_item (item))
	{
		items . push
		(
			transformer . transform_item
			(
				&delegated_type,
				&forwarded_trait,
				item
			)?
		);
	}

	{
		let predicates = &mut generics . make_where_clause () . predicates;
		predicates . push (parse_quote! (#delegated_type: #forwarded_trait));
		transformer . add_predicates (predicates);
	}

	let (impl_generics, _, where_clause) = generics . split_for_impl ();

	let trait_impl = quote!
	{
		#[automatically_derived]
		impl #impl_generics #forwarded_trait for #base_type
		#where_clause
		{
			#(#items)*
		}
	};

	Ok (trait_impl)
}

pub fn __forward_trait_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_trait_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
