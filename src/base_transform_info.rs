use syn::{Ident, Generics, Type, Fields, Token};
use syn::parse::Result;
use syn_derive::{Parse, ToTokens};

use crate::value_transformer::ValueTransformer;

use crate::base_conversion_transform_info::BaseConversionTransformInfo;
use crate::base_member_transform_info::BaseMemberTransformInfo;

#[derive (Parse, ToTokens)]
#[parse (prefix = |parse_stream| parse_stream . parse::<Ident> ())]
pub enum BaseTransformInfo
{
	#[parse (peek = Token! [->])]
	Conversion (BaseConversionTransformInfo),

	#[parse (peek = Token! [.])]
	Member (BaseMemberTransformInfo)
}

impl BaseTransformInfo
{
	pub fn get_type_macro_ident (&self) -> Ident
	{
		match self
		{
			Self::Conversion (base_conversion_transform_info) =>
				base_conversion_transform_info . get_macro_ident (),
			Self::Member (base_member_transform_info) =>
				base_member_transform_info . get_macro_ident ()
		}
	}

	pub fn into_value_transformer
	(
		self,
		from_type_generics: &Generics,
		fields: &Fields
	)
	-> Result <(Type, Type, ValueTransformer)>
	{
		let value_transformer = match self
		{
			Self::Conversion (conversion_transform_info) =>
			{
				let (from_type, to_type, conversion_transformer) =
					conversion_transform_info . into_value_transformer (from_type_generics);

				let value_transformer =
					ValueTransformer::from (conversion_transformer);

				(from_type, to_type, value_transformer)
			},
			Self::Member (base_member_transform_info) =>
			{
				let (from_type, to_type, member_transformer) =
					base_member_transform_info . into_value_transformer (from_type_generics, fields)?;

				let value_transformer =
					ValueTransformer::from (member_transformer);

				(from_type, to_type, value_transformer)
			}
		};

		Ok (value_transformer)
	}
}
