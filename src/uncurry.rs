use syn::{Attribute, Visibility, Path, Ident, parse_quote};
use syn::parse::{Result, Error};
use quote::{quote, ToTokens, format_ident};

use crate::fold::mangle::mangle_ident;

pub fn get_macro_ident (ident: &Ident) -> Ident
{
	format_ident! ("uncurry_trait_forwarding_info_for_{}", ident)
}

pub fn get_path_ident (path: &Path) -> Result <Ident>
{
	match path . segments . last ()
	{
		None => Err
		(
			Error::new_spanned (path, "Path must be nonempty")
		),
		Some (segment) => Ok (segment . ident . clone ())
	}
}

pub fn get_macro_path (path: &Path) -> Result <Path>
{
	let ident = get_path_ident (path)?;
	let macro_ident = get_macro_ident (&ident);
	let mut macro_path = path . clone ();
	macro_path . segments . pop ();
	macro_path . segments . push_value (parse_quote! (#macro_ident));

	Ok (macro_path)
}

pub fn gen_uncurry_macro <T>
(
	visibility: Visibility,
	macro_ident: Ident,
	injected_data: T
)
-> proc_macro2::TokenStream
where T: ToTokens
{
	let mangled_ident = mangle_ident (&macro_ident);

	let export_attribute: Option <Attribute> = match visibility
	{
		Visibility::Public (_) => Some (parse_quote! (#[macro_export])),
		_ => None
	};

	quote!
	{
		#[doc (hidden)]
		#export_attribute
		macro_rules! #mangled_ident
		{
			($receiver_path:path, $($receiver_args:tt)*) =>
			{
				$receiver_path! ($($receiver_args)*, #injected_data);
			}
		}

		#visibility use #mangled_ident as #macro_ident;
	}
}
