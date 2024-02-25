use syn::{Type, Expr, parse_quote};
use syn::parse::{Result, Error};

use crate::transformer::Transformer;
use crate::member::Member;

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

impl Transformer for MemberTransformer
{
	fn transform_input_self
	(
		&mut self,
		_delegated_type: &Type,
		input: Expr,
		_input_type: &Type
	)
	-> Result <Expr>
	{
		let member = &self . member;
		Ok (parse_quote! (#input . #member))
	}

	fn transform_input_ref_self
	(
		&mut self,
		_delegated_type: &Type,
		input: Expr,
		_input_type: &Type
	)
	-> Result <Expr>
	{
		let member = &self . member;
		Ok (parse_quote! (&#input . #member))
	}

	fn transform_input_ref_mut_self
	(
		&mut self,
		_delegated_type: &Type,
		input: Expr,
		_input_type: &Type
	)
	-> Result <Expr>
	{
		let member = &self . member;
		Ok (parse_quote! (&mut #input . #member))
	}

	fn transform_output_self
	(
		&mut self,
		_delegated_type: &Type,
		_output: Expr,
		output_type: &Type
	)
	-> Result <Expr>
	{
		Err
		(
			Error::new_spanned
			(
				output_type,
				"Cannot convert return values of `Self` for traits forwarded via member"
			)
		)
	}
}
