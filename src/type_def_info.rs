use syn::{Generics, Fields, FieldsNamed, FieldsUnnamed, Token};
use syn::token::{Brace, Paren};
use syn::parse::{Parse, ParseStream, Result};
use quote::ToTokens;

pub struct TypeDefInfo
{
	pub type_token: Token! [type],
	pub generics: Generics,
	pub fields: Fields
}

impl Parse for TypeDefInfo
{
	fn parse (input: ParseStream <'_>) -> Result <Self>
	{
		let type_token = input . parse ()?;

		let mut generics: Generics = input . parse ()?;
		generics . where_clause = input . parse ()?;

		let lookahead = input . lookahead1 ();
		let fields = if lookahead . peek (Brace)
		{
			Fields::from (input . parse::<FieldsNamed> ()?)
		}
		else if lookahead . peek (Paren)
		{
			let paren_fields = input . parse::<FieldsUnnamed> ()?;
			if paren_fields . unnamed . len () == 0
			{
				Fields::Unit
			}
			else
			{
				Fields::from (paren_fields)
			}
		}
		else
		{
			return Err (lookahead . error ())
		};

		Ok (Self {type_token, generics, fields})
	}
}

impl ToTokens for TypeDefInfo
{
	fn to_tokens (&self, tokens: &mut proc_macro2::TokenStream)
	{
		self . type_token . to_tokens (tokens);
		self . generics . to_tokens (tokens);
		self . generics . where_clause . to_tokens (tokens);
		self . fields . to_tokens (tokens);
	}
}
