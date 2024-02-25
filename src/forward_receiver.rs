use syn::{ItemStruct, Token, parse};
use syn::parse::{Result, Error};

use crate::uncurry::{uncurry_macro_ident, gen_uncurry_macro};
use crate::type_def_info::TypeDefInfo;

fn try_forward_receiver_impl
(
	_attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> Result <proc_macro2::TokenStream>
{
	let ItemStruct {vis, ident, generics, fields, ..} = parse (item . clone ())?;

	let macro_ident = uncurry_macro_ident (&ident);

	let type_info = TypeDefInfo
	{
		type_token: <Token! [type]>::default (),
		generics,
		fields
	};

	let mut tokens = proc_macro2::TokenStream::from (item);
	tokens . extend (gen_uncurry_macro (vis, macro_ident, type_info));

	Ok (tokens)
}

pub fn forward_receiver_impl
(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> proc_macro::TokenStream
{
	try_forward_receiver_impl (attr, item)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
