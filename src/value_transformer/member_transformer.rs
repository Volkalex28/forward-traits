use syn::{Expr, parse_quote};
use syn::parse::Result;

use crate::syn::member::Member;

pub struct MemberTransformer
{
	member: Member
}

impl MemberTransformer
{
	pub fn new (member: Member) -> Self
	{
		Self {member}
	}
}

impl MemberTransformer
{
	pub fn transform_input
	(
		&mut self,
		input: Expr
	)
	-> Result <Expr>
	{
		let member = &self . member;
		Ok (parse_quote! (#input . #member))
	}

	pub fn transform_input_ref
	(
		&mut self,
		input: Expr
	)
	-> Result <Expr>
	{
		let member = &self . member;
		Ok (parse_quote! (&#input . #member))
	}

	pub fn transform_input_ref_mut
	(
		&mut self,
		input: Expr
	)
	-> Result <Expr>
	{
		let member = &self . member;
		Ok (parse_quote! (&mut #input . #member))
	}
}
