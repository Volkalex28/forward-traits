use syn::{Type, Expr, WherePredicate, Token};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};

use crate::conversion_transformer::ConversionTransformer;
use crate::member_transformer::MemberTransformer;

pub enum ValueTransformer
{
	Conversion (ConversionTransformer),
	Member (MemberTransformer)
}

impl From <ConversionTransformer> for ValueTransformer
{
	fn from (conversion_transformer: ConversionTransformer) -> Self
	{
		Self::Conversion (conversion_transformer)
	}
}

impl From <MemberTransformer> for ValueTransformer
{
	fn from (member_transformer: MemberTransformer) -> Self
	{
		Self::Member (member_transformer)
	}
}

impl ValueTransformer
{
	pub fn transform_input
	(
		&mut self,
		input: Expr,
		from_type: &Type,
		to_type: &Type
	)
	-> Result <Expr>
	{
		match self
		{
			Self::Conversion (conversion_transformer) => conversion_transformer
				. transform_input (input, from_type, to_type),
			Self::Member (member_transformer) => member_transformer
				. transform_input (input)
		}
	}

	pub fn transform_input_ref
	(
		&mut self,
		input: Expr,
		from_type: &Type,
		to_type: &Type
	)
	-> Result <Expr>
	{
		match self
		{
			Self::Conversion (conversion_transformer) => conversion_transformer
				. transform_input_ref (input, from_type, to_type),
			Self::Member (member_transformer) => member_transformer
				. transform_input_ref (input)
		}
	}

	pub fn transform_input_ref_mut
	(
		&mut self,
		input: Expr,
		from_type: &Type,
		to_type: &Type
	)
	-> Result <Expr>
	{
		match self
		{
			Self::Conversion (conversion_transformer) => conversion_transformer
				. transform_input_ref_mut (input, from_type, to_type),
			Self::Member (member_transformer) => member_transformer
				. transform_input_ref_mut (input)
		}
	}

	pub fn transform_output
	(
		&mut self,
		output: Expr,
		from_type: &Type,
		to_type: &Type
	)
	-> Result <Expr>
	{
		match self
		{
			Self::Conversion (conversion_transformer) => conversion_transformer
				. transform_output (output, from_type, to_type),
			Self::Member (_member_transformer) => Err
			(
				Error::new_spanned
				(
					from_type,
					"Member delegation cannot transform return values for forwarding"
				)
			)
		}
	}

	pub fn add_predicates
	(
		&self,
		predicates: &mut Punctuated <WherePredicate, Token! [,]>,
		from_type: &Type,
		to_type: &Type
	)
	{
		match self
		{
			Self::Conversion (conversion_transformer) => conversion_transformer
				. add_predicates (predicates, from_type, to_type),
			Self::Member (_member_transformer) => {}
		}
	}
}
