use syn::{Ident, Generics, Type, Token, parse2};
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

#[derive (Parse, ToTokens)]
pub struct AssociatedType
{
	pub self_token: Token! [Self],
	pub double_colon_token: Token! [::],
	pub ident: Ident,
	pub generics: Generics
}

impl AssociatedType
{
	pub fn match_type (ty: &Type) -> Option <Self>
	{
		parse2 (ty . to_token_stream ()) . ok ()
	}
}
