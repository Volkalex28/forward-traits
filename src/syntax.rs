use syn::{Ident, Type, Token};
use syn_derive::{Parse, ToTokens};
use quote::format_ident;

pub mod kw
{
	use syn::custom_keyword;

	custom_keyword! (trait_info);
	custom_keyword! (type_info);

	custom_keyword! (tuple_struct);
}

#[derive (Clone, PartialEq, Eq, Hash, Parse, ToTokens)]
pub struct TypedIdent
{
	pub ident: Ident,
	pub colon: Token! [:],
	pub ty: Type
}

impl TypedIdent
{
	pub fn new (ident: Ident, ty: Type) -> Self
	{
		Self {ident, colon: <Token! [:]>::default (), ty}
	}
}

pub fn mangle_ident (ident: &Ident) -> Ident
{
	format_ident! ("__{}__", ident)
}
