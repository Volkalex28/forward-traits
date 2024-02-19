use syn::{Signature, WherePredicate, Token};
use syn::punctuated::Punctuated;

use super::{TraitAssociatedTypeInfo, TraitAssociatedConstInfo};

pub struct TraitImplInfo
{
	pub predicates: Punctuated <WherePredicate, Token! [,]>,
	pub associated_types: Punctuated <TraitAssociatedTypeInfo, Token! [;]>,
	pub methods: Punctuated <Signature, Token! [;]>,
	pub associated_constants: Punctuated <TraitAssociatedConstInfo, Token! [;]>
}
