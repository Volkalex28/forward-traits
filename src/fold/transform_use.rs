use syn::{UseName, UseRename, Token};
use syn::fold::Fold;

use crate::uncurry::get_macro_ident;

pub struct TransformUse {}

impl Fold for TransformUse
{
	fn fold_use_name (&mut self, node: UseName) -> UseName
	{
		UseName {ident: get_macro_ident (&node . ident)}
	}

	fn fold_use_rename (&mut self, node: UseRename) -> UseRename
	{
		UseRename
		{
			ident: get_macro_ident (&node . ident),
			as_token: <Token! [as]>::default (),
			rename: get_macro_ident (&node . rename)
		}
	}
}
