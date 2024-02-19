use std::num::TryFromIntError;

use syn::{Ident, Type, Path, Index, Signature, ReturnType, Token, parse_quote, parse};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn::fold::Fold;
use syn_derive::{Parse, ToTokens};
use quote::{quote, ToTokens};

use super::common
::{
	ReceiverTransforms,
	get_trait_parameter_values,
	gen_forwarded_trait
};
use crate::syntax::TypedIdent;
use crate::info
::{
	TraitDefInfo,
	TypeInfo,
	MemberInfo,
	MemberInfoStruct,
	MemberInfoTupleStruct
};
use crate::uncurry::{get_trait_macro_path, uncurry_macro_ident};

#[derive (Parse, ToTokens)]
enum Member
{
	#[parse (peek = Ident)]
	Ident (Ident),
	Index (Index)
}

fn get_member_ident_type (struct_info: MemberInfoStruct, member: Member)
-> Result <(Ident, Type)>
{
	let member_ident = match member
	{
		Member::Ident (ident) => ident,
		Member::Index (index) =>
			return Err (Error::new_spanned (index, "Member should be an identifier"))
	};

	let member_type = struct_info
		. members
		. into_iter ()
		. find_map (|TypedIdent {ident, ty, ..}| (&ident == &member_ident) . then_some (ty))
		. ok_or_else
		(
			|| Error::new_spanned (&member_ident, "Member does not exist in type")
		)?;

	Ok ((member_ident, member_type))
}

fn get_member_index_type
(
	tuple_struct_info: MemberInfoTupleStruct,
	member: Member
)
-> Result <(Index, Type)>
{
	let member_index = match member
	{
		Member::Index (index) => index,
		Member::Ident (ident) =>
			return Err (Error::new_spanned (ident, "Member should be an index"))
	};

	let member_index_usize = member_index
		. index
		. try_into ()
		. map_err
		(
			|int_error: TryFromIntError|
			Error::new_spanned (&member_index, int_error . to_string ())
		)?;

	if tuple_struct_info . members . len () < (member_index_usize + 1)
	{
		return Err (Error::new_spanned (&member_index, "Member does not exist in type"));
	}

	let member_type = tuple_struct_info . members [member_index_usize] . clone ();

	Ok ((member_index, member_type))
}

fn validate_method_return_types <'a, I> (methods: I) -> Result <()>
where I: IntoIterator <Item = &'a Signature>
{
	for method_signature in methods
	{
		match method_signature . output
		{
			ReturnType::Type (_, ref boxed_ty) if **boxed_ty == parse_quote! (Self) => return Err
			(
				Error::new_spanned
				(
					boxed_ty,
					"Member forwards cannot convert from delegated type to Self"
				)
			),
			_ => {}
		}
	}

	Ok (())
}

#[derive (Parse)]
struct ForwardTraitViaMember
{
	base_type_ident: Ident,
	_bt_comma: Token! [,],

	member: Member,
	_m_comma: Token! [,],

	forwarded_trait: Path,
	_ft_comma: Token! [,],

	type_info: TypeInfo,
	_ti_comma: Token! [,],

	forwarded_trait_info: TraitDefInfo
}

fn try_forward_trait_via_member_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraitViaMember
	{
		base_type_ident,
		member,
		forwarded_trait,
		type_info,
		forwarded_trait_info,
		..
	}
		= parse (input)?;

	let (type_info, mut partial_eval) = type_info . into_mangled ();
	let forwarded_trait = partial_eval . fold_path (forwarded_trait);

	let base_type_parameters = &type_info . parameters;
	let base_type = parse_quote! (#base_type_ident <#base_type_parameters>);

	let trait_parameter_values = get_trait_parameter_values (&forwarded_trait)?;

	let forwarded_trait_info =
		forwarded_trait_info . substitute (trait_parameter_values)?;

	validate_method_return_types (&forwarded_trait_info . methods)?;

	match type_info . member_info
	{
		MemberInfo::Struct (struct_info) =>
		{
			let (member_ident, member_type) =
				get_member_ident_type (struct_info, member)?;

			let receiver_predicates = vec!
			(
				parse_quote! (#member_type: #forwarded_trait)
			);

			let receiver_transforms = ReceiverTransforms
			{
				transform_ref: |expr| quote! (&#expr . #member_ident),
				transform_ref_mut: |expr| quote! (&mut #expr . #member_ident),
				transform_owned: |expr| quote! (#expr . #member_ident)
			};

			let return_transform = |expr| expr;

			let tokens = gen_forwarded_trait
			(
				&base_type,
				type_info . parameters,
				type_info . predicates,
				&forwarded_trait,
				forwarded_trait_info,
				&member_type,
				receiver_predicates,
				receiver_transforms,
				return_transform
			);

			Ok (tokens)
		},
		MemberInfo::TupleStruct (tuple_struct_info) =>
		{
			let (member_index, member_type) =
				get_member_index_type (tuple_struct_info, member)?;

			let receiver_predicates = vec!
			(
				parse_quote! (#member_type: #forwarded_trait)
			);

			let receiver_transforms = ReceiverTransforms
			{
				transform_ref: |expr| quote! (&#expr . #member_index),
				transform_ref_mut: |expr| quote! (&mut #expr . #member_index),
				transform_owned: |expr| quote! (#expr . #member_index)
			};

			let return_transform = |expr| expr;

			let tokens = gen_forwarded_trait
			(
				&base_type,
				type_info . parameters,
				type_info . predicates,
				&forwarded_trait,
				forwarded_trait_info,
				&member_type,
				receiver_predicates,
				receiver_transforms,
				return_transform
			);

			Ok (tokens)
		}
	}
}

pub fn __forward_trait_via_member_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_trait_via_member_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}

#[derive (Parse, ToTokens)]
struct ForwardTraitsViaMember
{
	base_type_ident: Ident,
	dot: Token! [.],
	member: Member,

	comma: Token! [,],

	#[parse (Punctuated::parse_terminated)]
	forwarded_traits: Punctuated <Path, Token! [,]>
}

fn try_forward_traits_via_member_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraitsViaMember
	{
		base_type_ident,
		member,
		forwarded_traits,
		..
	}
		= parse (input)?;

	let base_type_macro_ident = uncurry_macro_ident (&base_type_ident);

	let mut tokens = proc_macro2::TokenStream::new ();

	for forwarded_trait in forwarded_traits
	{
		let forwarded_trait_macro_path = get_trait_macro_path (&forwarded_trait)?;

		quote!
		{
			#base_type_macro_ident!
			(
				#forwarded_trait_macro_path,
				forward_traits::__forward_trait_via_member,
				#base_type_ident,
				#member,
				#forwarded_trait
			);
		}
			. to_tokens (&mut tokens);
	}

	Ok (tokens)
}

pub fn forward_traits_via_member_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_traits_via_member_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
