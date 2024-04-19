use syn::{Type, Token};
use syn::parse::{Parse, ParseStream, Result};
use syn_derive::ToTokens;

use super::associated_type::AssociatedType;

#[derive (ToTokens)]
pub enum FromType
{
	Independent (Type),
	Associated (AssociatedType)
}

impl Parse for FromType
{
	fn parse (input: ParseStream) -> Result <Self>
	{
		if input . peek (Token! [for]) || input . peek (Token! [Self])
		{
			Ok (FromType::Associated (input . parse ()?))
		}
		else
		{
			Ok (FromType::Independent (input . parse ()?))
		}
	}
}
