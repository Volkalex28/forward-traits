use syn::Type;
use syn::fold::{Fold, fold_type};

pub struct TypeTransformer <F>
where F: FnMut (&Type) -> Option <Type>
{
	pub transformations: F
}

impl <F> TypeTransformer <F>
where F: FnMut (&Type) -> Option <Type>
{
	pub fn new (transformations: F) -> Self
	{
		Self {transformations}
	}
}

impl <F> Fold for TypeTransformer <F>
where F: FnMut (&Type) -> Option <Type>
{
	fn fold_type (&mut self, node: Type) -> Type
	{
		match (self . transformations) (&node)
		{
			None => fold_type (self, node),
			Some (replacement) => replacement
		}
	}
}
