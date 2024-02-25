use syn::{Ident, Type, Token, parse_quote, parse};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn_derive::Parse;
use syn::fold::Fold;
use quote::{quote, ToTokens};

use crate::generics::combine_generics;
use crate::partial_eval::get_evaluator;
use crate::mangle::mangle_generics;
use crate::uncurry::{uncurry_macro_ident, get_trait_macro_path};

use crate::type_def_info::TypeDefInfo;
use crate::trait_def_info::TraitDefInfo;
use crate::forwarded_trait_info::ForwardedTraitInfo;

use crate::transformer::Transformer;
use crate::conversion_transformer::ConversionTransformer;

#[allow (dead_code)]
#[derive (Parse)]
struct ForwardTraitViaConversion
{
	base_type_ident: Ident,
	bt_comma_token: Token! [,],

	delegated_type: Type,
	dt_comma_token: Token! [,],

	forwarded_trait_info: ForwardedTraitInfo,
	ft_comma_token: Token! [,],

	type_def_info: TypeDefInfo,
	tyd_comma_token: Token! [,],

	trait_def_info: TraitDefInfo,
}

fn try_forward_trait_via_conversion_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraitViaConversion
	{
		base_type_ident,
		delegated_type,
		forwarded_trait_info,
		type_def_info,
		trait_def_info,
		..
	}
		= parse (input)?;

	let generics = combine_generics
	([
		type_def_info . generics . clone (),
		forwarded_trait_info . generics
	]);

	let (mut generics, mut mangler) = mangle_generics (generics);

	let type_def_generics = mangler . fold_generics (type_def_info . generics);

	let forwarded_trait =
		mangler . fold_path (forwarded_trait_info . trait_path);

	let delegated_type = mangler . fold_type (delegated_type);

	let mut evaluator = get_evaluator
	(
		trait_def_info . generics,
		&forwarded_trait
	)?;

	let mut conversion_transformer = ConversionTransformer::new ();

	let mut items = Vec::new ();

	for item in trait_def_info
		. items
		. into_iter ()
		. map (|item| evaluator . fold_trait_item (item))
	{
		items . push
		(
			conversion_transformer
				. transform_item (&delegated_type, &forwarded_trait, item)?
		);
	}

	let base_type: Type =
	{
		let (_, type_generics, _) = type_def_generics . split_for_impl ();
		parse_quote! (#base_type_ident #type_generics)
	};

	{
		let predicates = &mut generics
			. make_where_clause ()
			. predicates;

		predicates . push (parse_quote! (#delegated_type: #forwarded_trait));

		conversion_transformer . add_conversion_predicates
		(
			&base_type,
			&delegated_type,
			predicates
		);
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

pub fn __forward_trait_via_conversion_impl (input: proc_macro::TokenStream) -> proc_macro::TokenStream
{
	try_forward_trait_via_conversion_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}

#[allow (dead_code)]
#[derive (Parse)]
struct ForwardTraitsViaConversion
{
	base_type_ident: Ident,
	arrow_token: Token! [->],
	delegated_type: Type,

	comma_token: Token! [,],

	#[parse (Punctuated::parse_terminated)]
	forwarded_traits: Punctuated <ForwardedTraitInfo, Token! [,]>
}

fn try_forward_traits_via_conversion_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraitsViaConversion
	{
		base_type_ident,
		delegated_type,
		forwarded_traits,
		..
	}
		= parse (input)?;

	let base_type_macro_ident = uncurry_macro_ident (&base_type_ident);

	let mut tokens = proc_macro2::TokenStream::new ();

	for forwarded_trait_info in forwarded_traits
	{
		let forwarded_trait_macro_path =
			get_trait_macro_path (&forwarded_trait_info . trait_path)?;

		quote!
		{
			#base_type_macro_ident!
			(
				#forwarded_trait_macro_path,
				forward_traits::__forward_trait_via_conversion,
				#base_type_ident,
				#delegated_type,
				#forwarded_trait_info
			);
		}
			. to_tokens (&mut tokens);
	}

	Ok (tokens)
}

pub fn forward_traits_via_conversion_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_traits_via_conversion_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
