use syn::{Generics, Path, Token};
use syn::parse::{Parse, ParseStream, Result};
use quote::ToTokens;

pub struct ForwardedTraitInfo
{
	pub for_token: Option <Token! [for]>,
	pub generics: Generics,
	pub trait_path: Path
}

impl Parse for ForwardedTraitInfo
{
	fn parse (input: ParseStream <'_>) -> Result <Self>
	{
		let for_token: Option <Token! [for]> = input . parse ()?;
		let mut generics =
			if for_token . is_some () { input . parse ()? }
			else { Generics::default () };

		let trait_path = input . parse ()?;

		generics . where_clause = input . parse ()?;

		Ok (ForwardedTraitInfo {for_token, generics, trait_path})
	}
}

impl ToTokens for ForwardedTraitInfo
{
	fn to_tokens (&self, tokens: &mut proc_macro2::TokenStream)
	{
		if ! self . generics . params . is_empty ()
		{
			self . for_token . unwrap_or_default () . to_tokens (tokens);

			self . generics . to_tokens (tokens);
		}

		self . trait_path . to_tokens (tokens);

		self . generics . where_clause . to_tokens (tokens);
	}
}
