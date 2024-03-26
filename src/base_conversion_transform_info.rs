use syn::{Ident, Generics, Type, Token, parse_quote};
use syn_derive::{Parse, ToTokens};

use crate::uncurry::get_macro_ident;

use crate::conversion_transformer::ConversionTransformer;

#[derive (Parse, ToTokens)]
pub struct BaseConversionTransformInfo
{
	from_type_ident: Ident,
	arrow_token: Token! [->],
	to_type: Type
}

impl BaseConversionTransformInfo
{
	pub fn get_macro_ident (&self) -> Ident
	{
		get_macro_ident (&self . from_type_ident)
	}

	fn get_from_type (&self, from_type_generics: &Generics) -> Type
	{
		let from_type_ident = &self . from_type_ident;
		let (_, from_type_generics, _) = from_type_generics . split_for_impl ();
		parse_quote! (#from_type_ident #from_type_generics)
	}

	pub fn into_value_transformer (self, from_type_generics: &Generics)
	-> (Type, Type, ConversionTransformer)
	{
		(
			self . get_from_type (from_type_generics),
			self . to_type,
			ConversionTransformer::new ()
		)
	}
}
