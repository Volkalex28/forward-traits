use syn
::{
	Type,
	Generics,
	WherePredicate,
	ItemStruct,
	Fields,
	Token,
	parse
};
use syn::token::{Paren, Brace, Bracket};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn_derive::{Parse, ToTokens};

use super::generics::{ParameterInfo, parse_generics};
use crate::syntax::{TypedIdent, kw};
use crate::uncurry::{uncurry_macro_ident, gen_uncurry_macro};

#[derive (Parse, ToTokens)]
pub struct TypeInfo
{
	pub type_info_kw: kw::type_info,

	#[syn (parenthesized)]
	pub p_paren: Paren,

	#[syn (in = p_paren)]
	#[parse (Punctuated::parse_terminated)]
	pub parameters: Punctuated <ParameterInfo, Token! [,]>,

	#[syn (bracketed)]
	pub p_bracket: Bracket,

	#[syn (in = p_bracket)]
	#[parse (Punctuated::parse_terminated)]
	pub predicates: Punctuated <WherePredicate, Token! [,]>,

	pub member_info: MemberInfo
}

#[derive (Parse, ToTokens)]
pub enum MemberInfo
{
	#[parse (peek = Token! [struct])]
	Struct (MemberInfoStruct),

	#[parse (peek = kw::tuple_struct)]
	TupleStruct (MemberInfoTupleStruct),
}

#[derive (Parse, ToTokens)]
pub struct MemberInfoStruct
{
	pub struct_kw: Token! [struct],

	#[syn (braced)]
	pub brace: Brace,

	#[syn (in = brace)]
	#[parse (Punctuated::parse_terminated)]
	pub members: Punctuated <TypedIdent, Token! [,]>
}

#[derive (Parse, ToTokens)]
pub struct MemberInfoTupleStruct
{
	pub tuple_struct_kw: kw::tuple_struct,

	#[syn (parenthesized)]
	pub paren: Paren,

	#[syn (in = paren)]
	#[parse (Punctuated::parse_terminated)]
	pub members: Punctuated <Type, Token! [,]>
}

impl TypeInfo
{
	fn try_from (generics: Generics, fields: Fields) -> Result <TypeInfo>
	{
		let (parameters, _, predicates) =
			parse_generics (generics);

		let member_info = match fields
		{
			Fields::Named (named) => MemberInfo::Struct
			(
				MemberInfoStruct
				{
					struct_kw: <Token! [struct]>::default (),
					brace: Brace::default (),
					members: named
						. named
						. into_iter ()
						. map
						(
							|field| TypedIdent::new
							(
								field . ident . unwrap (),
								field . ty
							)
						)
						. collect ()
				}
			),
			Fields::Unnamed (unnamed) => MemberInfo::TupleStruct
			(
				MemberInfoTupleStruct
				{
					tuple_struct_kw: kw::tuple_struct::default (),
					paren: Paren::default (),
					members: unnamed
						. unnamed
						. into_iter ()
						. map (|field| field . ty)
						. collect ()
				}
			),
			Fields::Unit => MemberInfo::TupleStruct
			(
				MemberInfoTupleStruct
				{
					tuple_struct_kw: kw::tuple_struct::default (),
					paren: Paren::default (),
					members: Punctuated::new ()
				}
			)
		};

		Ok
		(
			TypeInfo
			{
				type_info_kw: kw::type_info::default (),

				p_paren: Paren::default (),
				parameters,

				p_bracket: Bracket::default (),
				predicates,

				member_info
			}
		)
	}
}

fn try_type_info_impl
(
	_attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> Result <proc_macro2::TokenStream>
{
	let ItemStruct {vis, ident, generics, fields, ..} = parse (item . clone ())?;

	let macro_ident = uncurry_macro_ident (&ident);

	let type_info = TypeInfo::try_from (generics, fields)?;

	let mut tokens = proc_macro2::TokenStream::from (item);
	tokens . extend (gen_uncurry_macro (vis, macro_ident, type_info));

	Ok (tokens)
}

pub fn type_info_impl
(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> proc_macro::TokenStream
{
	try_type_info_impl (attr, item)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
