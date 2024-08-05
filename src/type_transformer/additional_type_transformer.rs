use syn::{Block, BoundLifetimes, Ident, Token, Type};
use syn_derive::{Parse, ToTokens};

use crate::syn::member::Member;
use crate::syn::from_type::FromType;

use crate::value_transformer
::{
	conversion_transformer::ConversionTransformer,
	custom_transformer::CustomTransformer,
	member_transformer::MemberTransformer,
	value_transformer::ValueTransformer
};

use super::independent_type_transformer::IndependentTypeTransformer;
use super::associated_type_transformer::AssociatedTypeTransformer;

#[derive (Parse, ToTokens)]
pub enum TransformType
{
	#[parse (peek = Token! [->])]
	Conversion (Token! [->]),

	#[parse (peek = Token! [|])]
    Custom 
    {
        or1_token: Token! [|],
        ident: Ident,
        or2_token: Token! [|],
        block: Block,
        arrow_token: Token! [->]
    },

	#[parse (peek = Token! [.])]
	Member
	{
		dot_token: Token! [.],
		member: Member,
		colon_token: Token! [:],
        to_type: Type,
	}
}

impl TransformType
{
	pub fn into_value_transformer (self) -> ValueTransformer
	{
		match self
		{
			TransformType::Conversion (_) =>
				ValueTransformer::from (ConversionTransformer::new ()),
            TransformType::Custom {ident, block, ..} =>
                ValueTransformer::from (CustomTransformer::new (ident, block)),
			TransformType::Member {member, ..} =>
                ValueTransformer::from (MemberTransformer::new (member))
		}
	}
}

pub enum SpecializedTypeTransformer
{
	Independent (IndependentTypeTransformer),
	Associated (AssociatedTypeTransformer)
}

#[derive (Parse, ToTokens)]
pub struct AdditionalTypeTransformer
{
	pub lifetimes: Option <BoundLifetimes>,
	pub from_type: FromType,
	pub transform_type: TransformType,
	pub to_type: Type
}

impl AdditionalTypeTransformer
{
	pub fn specialize (self) -> SpecializedTypeTransformer
	{
		match self . from_type
		{
			FromType::Independent (from_type) => SpecializedTypeTransformer::Independent
			(
				IndependentTypeTransformer
				{
					lifetimes: self . lifetimes,
					from_type,
					to_type: self . to_type,
					value_transformer: self . transform_type . into_value_transformer ()
				}
			),
			FromType::Associated (associated_type) => SpecializedTypeTransformer::Associated
			(
				AssociatedTypeTransformer
				{
					lifetimes: self . lifetimes,
					associated_type,
					replacement_type: self . to_type,
					value_transformer: self . transform_type . into_value_transformer ()
				}
			)
		}
	}
}
