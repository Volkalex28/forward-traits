use std::collections::HashMap;

use syn::Type;
use syn::fold::{Fold, fold_type};

pub struct TypeTransformer
{
	pub transformations: HashMap <Type, Type>
}

impl TypeTransformer
{
	pub fn new () -> Self
	{
		Self {transformations: HashMap::new ()}
	}
}

impl Fold for TypeTransformer
{
	fn fold_type (&mut self, node: Type) -> Type
	{
		if let Some (ty) = self . transformations . get (&node)
		{
			ty . clone ()
		}
		else
		{
			fold_type (self, node)
		}
	}
}
