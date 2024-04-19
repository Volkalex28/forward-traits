use syn::{Lifetime, Type, Expr, GenericArgument, GenericParam, parse_quote};
use syn::parse::{Result, Error};
use syn_derive::{Parse, ToTokens};

use super::parameter_info::ParameterInfo;

#[derive (Clone, PartialEq, Eq, Hash, Parse, ToTokens)]
pub enum ParameterValue
{
	#[parse (peek = Lifetime)]
	Lifetime (Lifetime),

	#[parse (peek_func = |input| input . fork () . parse::<Type> () . is_ok ())]
	Type (Type),

	Const (Expr)
}

impl ParameterValue
{
	pub fn try_from_default_value (generic_param: GenericParam) -> Result <Self>
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) => Err
			(
				Error::new_spanned
				(
					lifetime_param,
					"Lifetime parameters cannot have default values"
				)
			),
			GenericParam::Type (type_param) => if let Some (ty) = type_param . default
			{
				Ok (ParameterValue::Type (ty))
			}
			else
			{
				Err
				(
					Error::new_spanned
					(
						type_param,
						"Type parameter lacks a default argument"
					)
				)
			},
			GenericParam::Const (const_param) => if let Some (expr) = const_param . default
			{
				Ok (ParameterValue::Const (expr))
			}
			else
			{
				Err
				(
					Error::new_spanned
					(
						const_param,
						"Const parameter lacks a default_argument"
					)
				)
			}
		}
	}
}

impl From <GenericParam> for ParameterValue
{
	fn from (generic_param: GenericParam) -> Self
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) =>
				ParameterValue::Lifetime (lifetime_param . lifetime),
			GenericParam::Type (type_param) =>
			{
				let ident = type_param . ident;
				ParameterValue::Type (parse_quote! (#ident))
			},
			GenericParam::Const (const_param) =>
			{
				let ident = const_param . ident;
				ParameterValue::Const (parse_quote! (#ident))
			}
		}
	}
}

impl TryFrom <GenericArgument> for ParameterValue
{
	type Error = Error;

	fn try_from (generic_argument: GenericArgument) -> Result <Self>
	{
		match generic_argument
		{
			GenericArgument::Lifetime (lifetime) =>
				Ok (ParameterValue::Lifetime (lifetime)),
			GenericArgument::Type (ty) => Ok (ParameterValue::Type (ty)),
			GenericArgument::Const (expr) => Ok (ParameterValue::Const (expr)),
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

impl <'a> From <&'a ParameterInfo> for ParameterValue
{
	fn from (info: &'a ParameterInfo) -> Self
	{
		match info
		{
			ParameterInfo::Lifetime (lifetime) =>
				ParameterValue::Lifetime (lifetime . clone ()),
			ParameterInfo::Type (ident) =>
				ParameterValue::Type (parse_quote! (#ident)),
			ParameterInfo::Const (_, ident) =>
				ParameterValue::Const (parse_quote! (#ident))
		}
	}
}
