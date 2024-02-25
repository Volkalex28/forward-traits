use syn::{Ident, Index, Fields, Type};
use syn::{Result, Error};
use syn_derive::{Parse, ToTokens};

#[derive (Parse, ToTokens)]
pub enum Member
{
	#[parse (peek = Ident)]
	Ident (Ident),

	// #[parse (peek = Index)]
	Index (Index)
}

pub fn get_member_type (fields: &Fields, member: &Member) -> Result <Type>
{
	match fields
	{
		Fields::Named (named_fields) =>
		{
			if let Member::Ident (member_ident) = member
			{
				named_fields
					. named
					. iter ()
					. find_map
					(
						|field|
						(field . ident . as_ref () . unwrap () == member_ident)
							. then (|| field . ty . clone ())
					)
					. ok_or_else
					(
						|| Error::new_spanned
						(
							member_ident,
							"Member not found in base type"
						)
					)
			}
			else
			{
				Err
				(
					Error::new_spanned
					(
						member,
						"Must use ident to name member of regular struct"
					)
				)
			}
		},
		Fields::Unnamed (unnamed_fields) =>
		{
			if let Member::Index (member_index) = member
			{
				let usize_index: usize = member_index
					. index
					. try_into ()
					. map_err
					(
						|err: <usize as TryFrom <u32>>::Error|
						{
							Error::new_spanned
							(
								member_index,
								err . to_string ()
							)
						}
					)?;

				if unnamed_fields . unnamed . len () > usize_index
				{
					Ok (unnamed_fields . unnamed [usize_index] . ty . clone ())
				}
				else
				{
					Err
					(
						Error::new_spanned
						(
							member_index,
							"Member not found in base type"
						)
					)
				}
			}
			else
			{
				Err
				(
					Error::new_spanned
					(
						member,
						"Must use index to name member of a tuple struct"
					)
				)
			}
		},
		Fields::Unit => Err
		(
			Error::new_spanned (member, "Member not found in base type")
		)
	}
}
