use syn::{Type, Token};
use syn_derive::{Parse, ToTokens};

use crate::value_transformer::ValueTransformer;

use crate::conversion_transform_info::ConversionTransformInfo;
use crate::member_transform_info::MemberTransformInfo;

#[derive (Parse, ToTokens)]
#[parse (prefix = |parse_stream| parse_stream . parse::<Type> ())]
pub enum TransformInfo
{
	#[parse (peek = Token! [->])]
	Conversion (ConversionTransformInfo),

	#[parse (peek = Token! [.])]
	Member (MemberTransformInfo)
}

impl From <ConversionTransformInfo> for TransformInfo
{
	fn from (conversion_transform_info: ConversionTransformInfo) -> Self
	{
		Self::Conversion (conversion_transform_info)
	}
}

impl From <MemberTransformInfo> for TransformInfo
{
	fn from (member_transform_info: MemberTransformInfo) -> Self
	{
		Self::Member (member_transform_info)
	}
}

impl TransformInfo
{
	pub fn into_value_transformer (self) -> (Type, Type, ValueTransformer)
	{
		match self
		{
			Self::Conversion (conversion_transform_info) =>
			{
				let (from_type, to_type, conversion_transformer) =
					conversion_transform_info . into_value_transformer ();

				let value_transformer =
					ValueTransformer::from (conversion_transformer);

				(from_type, to_type, value_transformer)
			},
			Self::Member (member_transform_info) =>
			{
				let (from_type, to_type, member_transformer) =
					member_transform_info . into_value_transformer ();

				let value_transformer =
					ValueTransformer::from (member_transformer);

				(from_type, to_type, value_transformer)
			}
		}
	}
}

#[cfg (test)]
mod tests
{
	use super::*;

	use syn::parse_quote;

	#[test]
	fn parse_transform_info ()
	{
		let _: TransformInfo = parse_quote! (A <T> . 0: T);
		let _: TransformInfo = parse_quote! (A -> B);
	}

	#[test]
	#[should_panic]
	fn fail_parse_not_transform_info ()
	{
		let _: TransformInfo = parse_quote! (A <T>);
	}

	#[allow (dead_code)]
	#[derive (Parse)]
	struct CommaInput
	{
		ti: TransformInfo,
		comma: Token! [,]
	}

	#[test]
	fn parse_transform_info_followed_by_comma ()
	{
		let _: CommaInput = parse_quote! (A <T> . 0: T,);
	}
}
