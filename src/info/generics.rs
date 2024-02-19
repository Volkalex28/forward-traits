use syn
::{
	Ident,
	Type,
	Lifetime,
	Expr,
	Generics,
	GenericParam,
	GenericArgument,
	WherePredicate,
	Token,
	parse_quote
};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn_derive::{Parse, ToTokens};

use crate::syntax::mangle_ident;

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

impl ParameterInfo
{
	pub fn to_mangled (&self) -> Self
	{
		match self
		{
			ParameterInfo::Lifetime (lifetime) => ParameterInfo::Lifetime
			(
				Lifetime
				{
					apostrophe: lifetime . apostrophe . clone (),
					ident: mangle_ident (&lifetime . ident)
				}
			),
			ParameterInfo::Type (ident) =>
				ParameterInfo::Type (mangle_ident (ident)),
			ParameterInfo::Const (const_token, ident) =>
				ParameterInfo::Const (const_token . clone (), mangle_ident (ident))
		}
	}
}

pub fn parse_generics (generics: Generics) ->
(
	Punctuated <ParameterInfo, Token! [,]>,
	Punctuated <ParameterValue, Token! [,]>,
	Punctuated <WherePredicate, Token! [,]>
)
{
	let mut parameters = Punctuated::new ();

	let mut default_values = Punctuated::new ();

	let mut predicates = match generics . where_clause
	{
		None => Punctuated::new (),
		Some (where_clause) => where_clause . predicates
	};

	for generic_param in generics . params
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) =>
			{
				if lifetime_param . colon_token . is_some ()
				{
					let lifetime = &lifetime_param . lifetime;
					let bounds = lifetime_param . bounds;
					predicates . push (parse_quote! (#lifetime: #bounds));
				}

				parameters . push (ParameterInfo::Lifetime (lifetime_param . lifetime));
			},
			GenericParam::Type (type_param) =>
			{
				if type_param . colon_token . is_some ()
				{
					let ident = &type_param . ident;
					let bounds = type_param . bounds;
					predicates . push (parse_quote! (#ident: #bounds));
				}

				if let Some (ty) = type_param . default
				{
					default_values . push (ParameterValue::Type (ty));
				}

				parameters . push (ParameterInfo::Type (type_param . ident));
			},
			GenericParam::Const (const_param) =>
			{
				if let Some (expr) = const_param . default
				{
					default_values . push (ParameterValue::Const (expr));
				}

				parameters . push (ParameterInfo::Const (const_param . const_token, const_param . ident));
			}
		}
	}

	(parameters, default_values, predicates)
}

#[derive (Clone, PartialEq, Eq, Hash, Parse, ToTokens)]
pub enum ParameterValue
{
	#[parse (peek = Lifetime)]
	Lifetime (Lifetime),

	#[parse (peek_func = |input| input . fork () . parse::<Type> () . is_ok ())]
	Type (Type),

	Const (Expr)
}

impl TryFrom <GenericArgument> for ParameterValue
{
	type Error = Error;

	fn try_from (value: GenericArgument) -> Result <Self>
	{
		match value
		{
			GenericArgument::Lifetime (lifetime) =>
				Ok (ParameterValue::Lifetime (lifetime)),
			GenericArgument::Type (ty) => Ok (ParameterValue::Type (ty)),
			GenericArgument::Const (expr) => Ok (ParameterValue::Const (expr)),
			_ => Err (Error::new_spanned (value, "Constraints make no sense in this context"))
		}
	}
}

impl <'a> From <&'a ParameterInfo> for ParameterValue
{
	fn from (info: &'a ParameterInfo) -> Self
	{
		match info
		{
			ParameterInfo::Lifetime (lifetime) => ParameterValue::Lifetime (lifetime . clone ()),
			ParameterInfo::Type (ident) => ParameterValue::Type (parse_quote! (#ident)),
			ParameterInfo::Const (_, ident) => ParameterValue::Const (parse_quote! (#ident))
		}
	}
}
