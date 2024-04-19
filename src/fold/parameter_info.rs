use syn::{Ident, Lifetime, GenericArgument, GenericParam, Token, parse2};
use syn::parse::{Result, Error};
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

#[derive (Clone, PartialEq, Eq, Hash, Parse, ToTokens)]
pub enum ParameterInfo
{
	#[parse (peek = Lifetime)]
	Lifetime (Lifetime),

	#[parse (peek = Ident)]
	Type (Ident),

	#[parse (peek = Token! [const])]
	Const (Token! [const], Ident)
}

impl From <GenericParam> for ParameterInfo
{
	fn from (generic_param: GenericParam) -> Self
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) =>
				ParameterInfo::Lifetime (lifetime_param . lifetime),
			GenericParam::Type (type_param) =>
				ParameterInfo::Type (type_param . ident),
			GenericParam::Const (const_param) =>
				ParameterInfo::Const (const_param . const_token, const_param . ident)
		}
	}
}

impl TryFrom <GenericArgument> for ParameterInfo
{	type Error = Error;

	fn try_from (generic_argument: GenericArgument) -> Result <Self>
	{
		match generic_argument
		{
			GenericArgument::Lifetime (lifetime) =>
				Ok (ParameterInfo::Lifetime (lifetime)),
			GenericArgument::Type (ty) =>
			{
				let ident: Ident = parse2 (ty . to_token_stream ())?;

				Ok (ParameterInfo::Type (ident))
			},
			GenericArgument::Const (expr) =>
			{
				let ident: Ident = parse2 (expr . to_token_stream ())?;

				Ok (ParameterInfo::Const (<Token! [const]>::default (), ident))
			},
			_ => Err
			(
				Error::new_spanned
				(
					generic_argument,
					"Constraints make no sense in this context"
				)
			)
		}
	}
}
