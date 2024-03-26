use syn::{Ident, Generics, Type, Fields, Token, parse_quote};
use syn::parse::Result;
use syn_derive::{Parse, ToTokens};

use crate::uncurry::get_macro_ident;

use crate::member::Member;

use crate::member_transformer::MemberTransformer;

#[derive (Parse, ToTokens)]
pub struct BaseMemberTransformInfo
{
	from_type_ident: Ident,
	dot_token: Token! [.],
	member: Member
}

impl BaseMemberTransformInfo
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

	pub fn into_value_transformer
	(
		self,
		from_type_generics: &Generics,
		fields: &Fields
	)
	-> Result <(Type, Type, MemberTransformer)>
	{
		Ok ((
			self . get_from_type (from_type_generics),
			self . member . get_member_type (fields)?,
			MemberTransformer::new (self . member)
		))
	}
}
