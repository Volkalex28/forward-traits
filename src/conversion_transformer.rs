use syn::{Type, Expr, WherePredicate, Token, parse_quote};
use syn::punctuated::Punctuated;
use syn::Result;

use crate::transformer::Transformer;

pub struct ConversionTransformer
{
	use_into: bool,
	use_borrow: bool,
	use_borrow_mut: bool,
	use_from: bool,
}

impl ConversionTransformer
{
	pub fn new () -> Self
	{
		Self
		{
			use_into: false,
			use_borrow: false,
			use_borrow_mut: false,
			use_from: false,
		}
	}

	pub fn add_conversion_predicates
	(
		&self,
		base_type: &Type,
		delegated_type: &Type,
		predicates: &mut Punctuated <WherePredicate, Token! [,]>
	)
	{
		if self . use_into
		{
			predicates . push
			(
				parse_quote! (#base_type: std::convert::Into <#delegated_type>)
			);
		}

		if self . use_borrow
		{
			predicates . push
			(
				parse_quote! (#base_type: std::borrow::Borrow <#delegated_type>)
			);
		}

		if self . use_borrow_mut
		{
			predicates . push
			(
				parse_quote! (#base_type: std::borrow::BorrowMut <#delegated_type>)
			);
		}

		if self . use_from
		{
			predicates . push
			(
				parse_quote! (#base_type: std::convert::From <#delegated_type>)
			);
		}
	}
}

impl Transformer for ConversionTransformer
{
	fn transform_input_self
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		input_type: &Type
	)
	-> Result <Expr>
	{
		self . use_into = true;

		let input = parse_quote!
		(
			<#input_type as std::convert::Into <#delegated_type>>::into (#input)
		);

		Ok (input)
	}

	fn transform_input_ref_self
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		input_type: &Type
	)
	-> Result <Expr>
	{
		self . use_borrow = true;

		let input = parse_quote!
		(
			<#input_type as std::borrow::Borrow <#delegated_type>>::borrow (#input)
		);

		Ok (input)
	}

	fn transform_input_ref_mut_self
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		input_type: &Type
	)
	-> Result <Expr>
	{
		self . use_borrow_mut = true;

		let input = parse_quote!
		(
			<#input_type as std::borrow::BorrowMut <#delegated_type>>::borrow_mut (#input)
		);

		Ok (input)
	}

	fn transform_output_self
	(
		&mut self,
		delegated_type: &Type,
		output: Expr,
		output_type: &Type
	)
	-> Result <Expr>
	{
		self . use_from = true;

		let output = parse_quote!
		(
			<#output_type as std::convert::From <#delegated_type>>::from (#output)
		);

		Ok (output)
	}
}
