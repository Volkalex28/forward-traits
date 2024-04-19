use syn::{Ident, Type, Generics, Fields, Token, parse_quote};
use syn::parse::Result;
use syn_derive::{Parse, ToTokens};

use crate::uncurry::get_macro_ident;

use crate::syn::member::Member;

use crate::value_transformer
::{
	conversion_transformer::ConversionTransformer,
	member_transformer::MemberTransformer,
	value_transformer::ValueTransformer
};

use super::independent_type_transformer::IndependentTypeTransformer;

#[derive (Parse, ToTokens)]
pub enum BaseTransformType
{
	#[parse (peek = Token! [->])]
	Conversion {arrow_token: Token! [->], to_type: Type},

	#[parse (peek = Token! [.])]
	Member {dot_token: Token! [.], member: Member}
}

#[derive (Parse, ToTokens)]
pub struct BaseTypeTransformer
{
	base_type_ident: Ident,
	transform_type: BaseTransformType
}

impl BaseTypeTransformer
{
	pub fn get_type_macro_ident (&self) -> Ident
	{
		get_macro_ident (&self . base_type_ident)
	}

	pub fn into_type_transformer
	(
		self, base_type_generics:
		&Generics, fields: &Fields
	)
	-> Result <(Type, Type, IndependentTypeTransformer)>
	{
		let base_type_ident = &self . base_type_ident;
		let base_type: Type = parse_quote! (#base_type_ident #base_type_generics);

		let from_type: Type = parse_quote! (Self);

		let (to_type, value_transformer) = match self . transform_type
		{
			BaseTransformType::Conversion {to_type, ..} =>
			(
				to_type,
				ValueTransformer::from (ConversionTransformer::new ())
			),
			BaseTransformType::Member {member, ..} =>
			(
				member . get_member_type (fields)?,
				ValueTransformer::from (MemberTransformer::new (member))
			)
		};

		let delegated_type = to_type . clone ();

		let independent_type_transformer = IndependentTypeTransformer
		{
			lifetimes: None,
			from_type,
			to_type,
			value_transformer
		};

		Ok ((base_type, delegated_type, independent_type_transformer))
	}
}
