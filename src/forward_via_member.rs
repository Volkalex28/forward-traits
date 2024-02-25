use syn::{Ident, Type, Token, parse_quote, parse};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn::fold::Fold;
use syn_derive::Parse;
use quote::{quote, ToTokens};

use crate::generics::combine_generics;
use crate::partial_eval::get_evaluator;
use crate::mangle::mangle_generics;
use crate::uncurry::{uncurry_macro_ident, get_trait_macro_path};

use crate::type_def_info::TypeDefInfo;
use crate::trait_def_info::TraitDefInfo;
use crate::forwarded_trait_info::ForwardedTraitInfo;

use crate::member::{Member, get_member_type};

use crate::transformer::Transformer;
use crate::member_transformer::MemberTransformer;

#[allow (dead_code)]
#[derive (Parse)]
struct ForwardTraitViaMember
{
	base_type_ident: Ident,
	bt_comma_token: Token! [,],

	member: Member,
	m_comma_token: Token! [,],

	forwarded_trait_info: ForwardedTraitInfo,
	ft_comma_token: Token! [,],

	type_def_info: TypeDefInfo,
	tyd_comma_token: Token! [,],

	trait_def_info: TraitDefInfo,
}

fn try_forward_trait_via_member_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraitViaMember
	{
		base_type_ident,
		member,
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

	let delegated_type = mangler . fold_type
	(
		get_member_type (&type_def_info . fields, &member)?
	);

	let mut evaluator = get_evaluator
	(
		trait_def_info . generics,
		&forwarded_trait
	)?;

	let mut member_transformer = MemberTransformer::new (member);

	let mut items = Vec::new ();

	for item in trait_def_info
		. items
		. into_iter ()
		. map (|item| evaluator . fold_trait_item (item))
	{
		items . push
		(
			member_transformer
				. transform_item (&delegated_type, &forwarded_trait, item)?
		);
	}

	let base_type: Type =
	{
		let (_, type_generics, _) = type_def_generics . split_for_impl ();
		parse_quote! (#base_type_ident #type_generics)
	};

	generics
		. make_where_clause ()
		. predicates
		. push (parse_quote! (#delegated_type: #forwarded_trait));

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

pub fn __forward_trait_via_member_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_trait_via_member_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}

#[allow (dead_code)]
#[derive (Parse)]
struct ForwardTraitsViaMember
{
	base_type_ident: Ident,
	dot_token: Token! [.],
	member: Member,

	comma_token: Token! [,],

	#[parse (Punctuated::parse_terminated)]
	forwarded_traits: Punctuated <ForwardedTraitInfo, Token! [,]>
}

fn try_forward_traits_via_member_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraitsViaMember
	{
		base_type_ident,
		member,
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
				forward_traits::__forward_trait_via_member,
				#base_type_ident,
				#member,
				#forwarded_trait_info
			);
		}
			. to_tokens (&mut tokens);
	}

	Ok (tokens)
}

pub fn forward_traits_via_member_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_traits_via_member_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
