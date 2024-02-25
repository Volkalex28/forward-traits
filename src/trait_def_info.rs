use syn::{Generics, ItemTrait, TraitItem, Token, parse_quote, braced};
use syn::token::Brace;
use syn::parse::{Parse, ParseStream, Result, Error};
use quote::{ToTokens, TokenStreamExt};

fn strip_default (item: &mut TraitItem) -> Result <()>
{
	match item
	{
		TraitItem::Const (item_const) => Ok (item_const . default = None),
		TraitItem::Fn (item_fn) => Ok (item_fn . default = None),
		TraitItem::Type (item_type) => Ok (item_type . default = None),
		_ => Err
		(
			Error::new_spanned
			(
				item,
				"Traits with this type of item are not supported for forwarding"
			)
		)
	}
}

pub struct TraitDefInfo
{
	pub trait_token: Token! [trait],
	pub generics: Generics,
	pub brace_token: Brace,
	pub items: Vec <TraitItem>
}

impl Parse for TraitDefInfo
{
	fn parse (input: ParseStream <'_>) -> Result <Self>
	{
		let trait_token = input . parse ()?;

		let mut generics: Generics = input . parse ()?;
		generics . where_clause = input . parse ()?;

		let content;
		let brace_token = braced! (content in input);

		let mut items = Vec::new ();
		while ! content . is_empty ()
		{
			let mut item = content . parse ()?;
			strip_default (&mut item)?;
			items . push (item);
		}

		Ok (TraitDefInfo {trait_token, generics, brace_token, items})
	}
}

impl ToTokens for TraitDefInfo
{
	fn to_tokens (&self, tokens: &mut proc_macro2::TokenStream)
	{
		self . trait_token . to_tokens (tokens);
		self . generics . to_tokens (tokens);
		self . generics . where_clause . to_tokens (tokens);
		self . brace_token . surround
		(
			tokens,
			|tokens| tokens . append_all (&self . items)
		);
	}
}

impl TryFrom <ItemTrait> for TraitDefInfo
{
	type Error = Error;

	fn try_from (item_trait: ItemTrait) -> Result <TraitDefInfo>
	{
		let trait_token = <Token! [trait]>::default ();

		let mut generics = item_trait . generics;

		if ! item_trait . supertraits . is_empty ()
		{
			let supertraits = &item_trait . supertraits;

			generics
				. make_where_clause ()
				. predicates
				. push (parse_quote! (Self: #supertraits));
		}

		let brace_token = item_trait . brace_token;

		let mut items = item_trait . items;

		for item in &mut items
		{
			strip_default (item)?;
		}

		Ok (TraitDefInfo {trait_token, generics, brace_token, items})
	}
}
