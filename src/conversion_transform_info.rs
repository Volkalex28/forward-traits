use syn::{Type, Token};
use syn_derive::{Parse, ToTokens};

use crate::conversion_transformer::ConversionTransformer;

#[derive (Parse, ToTokens)]
pub struct ConversionTransformInfo
{
	from_type: Type,
	arrow_token: Token! [->],
	to_type: Type
}

impl ConversionTransformInfo
{
	pub fn into_value_transformer (self)
	-> (Type, Type, ConversionTransformer)
	{
		(self . from_type, self . to_type, ConversionTransformer::new ())
	}
}
