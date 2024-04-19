use syn::{Attribute, Visibility, ItemTrait, ItemUse, Token, parse};
use syn::parse::{Result, Error};
use syn::fold::Fold;
use syn_derive::Parse;
use quote::ToTokens;

use crate::uncurry::{get_macro_ident, gen_uncurry_macro};
use crate::syn::trait_def_info::TraitDefInfo;
use crate::fold::transform_use::TransformUse;

#[derive (Parse)]
#[parse (
	prefix = |parse_stream|
	{
		Attribute::parse_outer (parse_stream)?;
		parse_stream . parse::<Visibility> ()?;
		Ok (())
	}
)]
enum Forwardable
{
	#[parse (peek = Token! [trait])]
	ItemTrait (ItemTrait),

	#[parse (peek = Token! [use])]
	ItemUse (ItemUse)
}

fn try_forwardable_impl
(
	_attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> Result <proc_macro2::TokenStream>
{
	let mut tokens = proc_macro2::TokenStream::from (item . clone ());

	match parse (item)?
	{
		Forwardable::ItemTrait (item_trait) =>
		{
			let vis = item_trait . vis . clone ();

			let macro_ident = get_macro_ident (&item_trait . ident);

			let trait_def_info = TraitDefInfo::try_from (item_trait)?;

			tokens . extend
			(
				gen_uncurry_macro (vis, macro_ident, trait_def_info)
			);
		},
		Forwardable::ItemUse (item_use) =>
		{
			TransformUse {}
				. fold_item_use (item_use)
				. to_tokens (&mut tokens);
		}
	}

	Ok (tokens)
}

pub fn forwardable_impl
(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> proc_macro::TokenStream
{
	try_forwardable_impl (attr, item)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
