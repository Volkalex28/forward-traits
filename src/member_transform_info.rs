use syn::{Type, Token};
use syn_derive::{Parse, ToTokens};

use crate::member::Member;

use crate::member_transformer::MemberTransformer;

#[derive (Parse, ToTokens)]
pub struct MemberTransformInfo
{
	from_type: Type,
	dot_token: Token! [.],
	member: Member,
	colon_token: Token! [:],
	to_type: Type
}

impl MemberTransformInfo
{
	pub fn into_value_transformer (self)
	-> (Type, Type, MemberTransformer)
	{
		(self . from_type, self . to_type, MemberTransformer::new (self . member))
	}
}
