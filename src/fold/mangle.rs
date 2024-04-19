use syn::{Lifetime, Ident, Generics, GenericParam, parse_quote};
use syn::fold::Fold;
use quote::format_ident;

use super::partial_eval::PartialEval;

pub fn mangle_ident (ident: &Ident) -> Ident
{
	format_ident! ("__{}__", ident)
}

pub fn mangle_lifetime (lifetime: &Lifetime) -> Lifetime
{
	Lifetime
	{
		apostrophe: lifetime . apostrophe . clone (),
		ident: mangle_ident (&lifetime . ident)
	}
}

pub fn mangle_generics (generics: Generics) -> (Generics, PartialEval)
{
	let mut mangler = PartialEval::new ();

	for param in &generics . params
	{
		match param
		{
			GenericParam::Lifetime (lifetime_param) =>
			{
				let lifetime = &lifetime_param . lifetime;
				let mangled_lifetime = mangle_lifetime (lifetime);
				mangler . parameters . insert
				(
					parse_quote! (#lifetime),
					parse_quote! (#mangled_lifetime)
				);
			},
			GenericParam::Type (type_param) =>
			{
				let ident = &type_param . ident;
				let mangled_ident = mangle_ident (ident);
				mangler . parameters . insert
				(
					parse_quote! (#ident),
					parse_quote! (#mangled_ident)
				);
			},
			GenericParam::Const (const_param) =>
			{
				let const_token = &const_param . const_token;
				let ident = &const_param . ident;
				let mangled_ident = mangle_ident (ident);
				mangler . parameters . insert
				(
					parse_quote! (#const_token #ident),
					parse_quote! (#mangled_ident)
				);
			}
		}
	}

	let generics = mangler . fold_generics (generics);

	(generics, mangler)
}
