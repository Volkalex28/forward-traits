use syn::{Attribute, Visibility, Path, Ident, parse_quote};
use syn::parse::{Result, Error};
use quote::{quote, ToTokens, format_ident};

use crate::mangle::mangle_ident;

pub fn uncurry_macro_ident (base_info_ident: &Ident) -> Ident
{
	format_ident! ("uncurry_trait_forwarding_info_for_{}", base_info_ident)
}

pub fn get_trait_ident (trait_path: &Path) -> Result <Ident>
{
	match trait_path . segments . last ()
	{
		None => Err
		(
			Error::new_spanned (trait_path, "Path to trait must be nonempty")
		),
		Some (segment) => Ok (segment . ident . clone ())
	}
}

pub fn get_trait_macro_path (trait_path: &Path) -> Result <Path>
{
	let trait_ident = get_trait_ident (trait_path)?;
	let trait_macro_ident = uncurry_macro_ident (&trait_ident);
	let mut trait_macro_path = trait_path . clone ();
	trait_macro_path . segments . pop ();
	trait_macro_path . segments . push_value (parse_quote! (#trait_macro_ident));

	Ok (trait_macro_path)
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
