use syn::{Attribute, Visibility, Ident, parse_quote};
use quote::{quote, ToTokens, format_ident};

pub fn uncurry_macro_ident (base_info_ident: &Ident) -> Ident
{
	format_ident! ("uncurry_trait_forwarding_info_for_{}", base_info_ident)
}

fn mangle_ident (ident: &Ident) -> Ident
{
	format_ident! ("__{}__", ident)
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
				$receiver_path! ($($receiver_args)*, #injected_data)
			}
		}

		#visibility use #mangled_ident as #macro_ident;
	}
}
