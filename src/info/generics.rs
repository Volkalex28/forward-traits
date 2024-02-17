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

#[derive (Clone, PartialEq, Eq, Hash, Parse, ToTokens)]
pub enum ParameterInfo
{
	#[parse (peek = Lifetime)]
	Lifetime (Lifetime),

	#[parse (peek = Ident)]
	TypeOrConst (Ident)
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

				parameters . push (ParameterInfo::TypeOrConst (type_param . ident));
			},
			GenericParam::Const (const_param) =>
			{
				if let Some (expr) = const_param . default
				{
					default_values . push (ParameterValue::Const (expr));
				}

				parameters . push (ParameterInfo::TypeOrConst (const_param . ident));
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
	// This being syn::Error.
	type Error = Error;

	fn try_from (value: GenericArgument) -> Result <ParameterValue>
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
