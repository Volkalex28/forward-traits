use syn::{Ident, Signature, WherePredicate, Token};
use syn::punctuated::Punctuated;

use crate::syntax::TypedIdent;

pub struct TraitImplInfo
{
	pub predicates: Punctuated <WherePredicate, Token! [,]>,
	pub associated_types: Punctuated <Ident, Token! [,]>,
	pub methods: Punctuated <Signature, Token! [,]>,
	pub associated_constants: Punctuated <TypedIdent, Token! [,]>
}
