use syn::{Type, Expr, PathSegment, Token, parse2};
use syn::token::{Paren, Bracket};
use syn::punctuated::Punctuated;
use syn_derive::Parse;
use quote::ToTokens;

use super::kw;

fn last (ty: &Type) -> Option <&PathSegment>
{
	if let Type::Path (type_path) = ty
	{
		match type_path . qself
		{
			None => type_path . path . segments . last (),
			Some (_) => None
		}
	}
	else
	{
		None
	}
}

#[derive (Parse)]
pub struct BoxType
{
	pub box_token: kw::Box,
	pub l_angle_token: Token! [<],
	pub boxed_type: Type,
	pub r_angle_token: Token! [>]
}

impl BoxType
{
	pub fn match_type (ty: &Type) -> Option <BoxType>
	{
		match last (ty)
		{
			Some (path_segment) =>
				parse2 (path_segment . to_token_stream ()) . ok (),
			None => None
		}
	}
}

#[derive (Parse)]
pub struct OptionType
{
	pub option_token: kw::Option,
	pub l_angle_token: Token! [<],
	pub option_type: Type,
	pub r_angle_token: Token! [>]
}

impl OptionType
{
	pub fn match_type (ty: &Type) -> Option <OptionType>
	{
		match last (ty)
		{
			Some (path_segment) =>
				parse2 (path_segment . to_token_stream ()) . ok (),
			None => None
		}
	}
}

#[derive (Parse)]
pub struct ResultType
{
	pub result_token: kw::Result,
	pub l_angle_token: Token! [<],
	pub result_type: Type,
	pub comma_token: Token! [,],
	pub error_type: Type,
	pub r_angle_token: Token! [>]
}

impl ResultType
{
	pub fn match_type (ty: &Type) -> Option <ResultType>
	{
		match last (ty)
		{
			Some (path_segment) =>
				parse2 (path_segment . to_token_stream ()) . ok (),
			None => None
		}
	}
}

#[derive (Parse)]
pub struct TupleType
{
	#[syn (parenthesized)]
	pub paren: Paren,

	#[syn (in = paren)]
	#[parse (Punctuated::parse_terminated)]
	pub types: Punctuated <Type, Token! [,]>
}

impl TupleType
{
	pub fn match_type (ty: &Type) -> Option <TupleType>
	{
		parse2 (ty . to_token_stream ()) . ok ()
	}
}

#[derive (Parse)]
pub struct ArrayType
{
	#[syn (bracketed)]
	pub bracket: Bracket,

	#[syn (in = bracket)]
	pub ty: Type,

	#[syn (in = bracket)]
	pub semi_token: Token! [;],

	#[syn (in = bracket)]
	pub count: Expr
}

impl ArrayType
{
	pub fn match_type (ty: &Type) -> Option <ArrayType>
	{
		parse2 (ty . to_token_stream ()) . ok ()
	}
}
