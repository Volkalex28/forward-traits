use syn::{Generics, Token, parse_quote, parse};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result, Error};
use syn::fold::Fold;
use syn_derive::Parse;
use quote::{quote, ToTokens};

use crate::generics::combine_generics;

use crate::syn
::{
	trait_def_info::TraitDefInfo,
	type_def_info::TypeDefInfo,
	forwarded_trait_info::ForwardedTraitInfo,
	additional_type_transformers::AdditionalTypeTransformers
};

use crate::fold::mangle::mangle_generics;
use crate::fold::evaluator::get_trait_path_evaluator;

use crate::type_transformer::base_type_transformer::BaseTypeTransformer;

use crate::transformer::TransformerBuilder;

struct TypeTransformInfo
{
	for_token: Token! [for],
	generics: Generics,
	base_type_transformer: BaseTypeTransformer,
	additional_type_transformers: AdditionalTypeTransformers
}

impl Parse for TypeTransformInfo
{
	fn parse (input: ParseStream) -> Result <Self>
	{
		let for_token = input . parse ()?;

		let mut generics: Generics = input . parse ()?;

		let base_type_transformer = input . parse ()?;

		let additional_type_transformers = input . parse ()?;

		generics . where_clause = input . parse ()?;

		let type_transform_info = Self
		{
			for_token,
			generics,
			base_type_transformer,
			additional_type_transformers
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
		self . base_type_transformer . to_tokens (tokens);
		self . additional_type_transformers . to_tokens (tokens);
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
		type_transform_info . base_type_transformer . get_type_macro_ident ();

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

	let mut transformer_builder = TransformerBuilder::new ();

	// The name of this method sucks, in context.
	let (base_type, delegated_type, independent_type_transformer) =
		type_transform_info . base_type_transformer . into_type_transformer
		(
			&type_def_info . generics,
			&type_def_info . fields
		)?;

	let independent_type_transformer = mangler
		. fold_independent_type_transformer (independent_type_transformer);
	let base_type = mangler . fold_type (base_type);
	let delegated_type = mangler . fold_type (delegated_type);

	transformer_builder . add_independent_type_transformer
	(
		independent_type_transformer
	);

	for additional_type_transformer
	in type_transform_info . additional_type_transformers
	{
		let additional_type_transformer =
			mangler . fold_additional_type_transformer (additional_type_transformer);
		transformer_builder . add_additional_type_transformer (additional_type_transformer);
	}

	let mut transformer = transformer_builder . into_transformer
	(
		delegated_type . clone (),
		forwarded_trait . clone ()
	);

	let mut evaluator =
		get_trait_path_evaluator (trait_def_info . generics, &forwarded_trait)?;

	let mut items = Vec::new ();

	for item in trait_def_info
		. items
		. into_iter ()
		. map (|item| evaluator . fold_trait_item (item))
	{
		items . push (transformer . transform_trait_item (item)?);
	}

	// The transformer transforms the forwarded trait as a side-effect of being
	// constructed.
	let transformed_forwarded_trait =
		transformer . get_transformed_forwarded_trait ();

	{
		let predicates = &mut generics . make_where_clause () . predicates;
		predicates . push
		(
			parse_quote! (#delegated_type: #transformed_forwarded_trait)
		);
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
