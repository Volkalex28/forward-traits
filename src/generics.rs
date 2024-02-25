use syn::{Generics, WhereClause, GenericParam, Token};
use syn::punctuated::Punctuated;

pub fn combine_generics <const N: usize> (parts: [Generics; N]) -> Generics
{
	let mut params = Punctuated::new ();
	let mut predicates = Punctuated::new ();

	for part in parts
	{
		params . extend (part . params);
		if let Some (where_clause) = part . where_clause
		{
			predicates . extend (where_clause . predicates);
		}
	}

	let where_clause =
		if predicates . is_empty () { None }
		else
		{
			Some
			(
				WhereClause
				{
					where_token: <Token! [where]>::default (),
					predicates
				}
			)
		};

	Generics {lt_token: None, params, gt_token: None, where_clause}
}

pub fn get_num_required_arguments (generics: &Generics) -> usize
{
	let mut count: usize = 0;

	for param in &generics . params
	{
		match param
		{
			GenericParam::Lifetime (_) => count += 1,
			GenericParam::Type (type_param)
				if ! type_param . default . is_some () => count += 1,
			GenericParam::Const (const_param)
				if ! const_param . default . is_some () => count += 1,
			_ => {}
		}
	}

	count
}
