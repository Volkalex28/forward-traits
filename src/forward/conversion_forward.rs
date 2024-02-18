use syn::{Ident, Type, Path, Signature, FnArg, Token, parse_quote, parse};
use syn::token::Paren;
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn_derive::{Parse, ToTokens};
use quote::{quote, ToTokens};

use super::common
::{
	ReceiverTransforms,
	get_trait_parameter_values,
	gen_forwarded_trait
};
use crate::info::{TraitDefInfo, TypeInfo};
use crate::uncurry::{get_trait_macro_path, uncurry_macro_ident};

struct TraitReceiverTypes
{
	ref_self: bool,
	ref_mut_self: bool,
	owned_self: bool
}

impl <'a, I> From <I> for TraitReceiverTypes
where I: IntoIterator <Item = &'a Signature>
{
	fn from (methods: I) -> TraitReceiverTypes
	{
		let mut receiver_types = TraitReceiverTypes
		{
			ref_self: false,
			ref_mut_self: false,
			owned_self: false
		};

		for method_signature in methods
		{
			let Signature {inputs, ..} = method_signature;

			if let Some (FnArg::Receiver (receiver)) = inputs . first ()
			{
				if receiver . ty == parse_quote! (&Self)
				{
					receiver_types . ref_self = true;
				}
				else if receiver . ty == parse_quote! (&mut Self)
				{
					receiver_types . ref_mut_self = true;
				}
				else if receiver . ty == parse_quote! (Self)
				{
					receiver_types . owned_self = true;
				}
				else { unreachable! (); }
			}
		}

		receiver_types
	}
}

#[derive (Parse)]
struct ForwardTraitViaConversion
{
	base_type_ident: Ident,
	_bt_comma: Token! [,],

	delegated_type: Type,
	_dt_comma: Token! [,],

	forwarded_trait: Path,
	_ft_comma: Token! [,],

	type_info: TypeInfo,
	_ti_comma: Token! [,],

	forwarded_trait_info: TraitDefInfo
}

fn try_forward_trait_via_conversion_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraitViaConversion
	{
		base_type_ident,
		delegated_type,
		forwarded_trait,
		type_info,
		forwarded_trait_info,
		..
	}
		= parse (input)?;

	let base_type_parameters = &type_info . parameters;
	let base_type = parse_quote! (#base_type_ident <#base_type_parameters>);

	let trait_parameter_values = get_trait_parameter_values (&forwarded_trait)?;

	let forwarded_trait_info =
		forwarded_trait_info . substitute (trait_parameter_values)?;

	let trait_receiver_types =
		TraitReceiverTypes::from (&forwarded_trait_info . methods);

	let mut receiver_predicates = Vec::new ();
	receiver_predicates . push (parse_quote! (#delegated_type: #forwarded_trait));

	if trait_receiver_types . ref_self
	{
		receiver_predicates . push
		(
			parse_quote! (#base_type: std::borrow::Borrow <#delegated_type>)
		);
	}

	if trait_receiver_types . ref_mut_self
	{
		receiver_predicates . push
		(
			parse_quote! (#base_type: std::borrow::BorrowMut <#delegated_type>)
		);
	}

	if trait_receiver_types . owned_self
	{
		receiver_predicates . push
		(
			parse_quote! (#base_type: std::convert::Into <#delegated_type>)
		);
	}

	let receiver_transforms = ReceiverTransforms
	{
		transform_ref: |self_token| parse_quote!
		(
			<#base_type as std::borrow::Borrow <#delegated_type>>
				::borrow (#self_token)
		),
		transform_ref_mut: |self_token| parse_quote!
		(
			<#base_type as std::borrow::BorrowMut <#delegated_type>>
				::borrow_mut (#self_token)
		),
		transform_owned: |self_token| parse_quote!
		(
			<#base_type as std::convert::Into <#delegated_type>>
				::into (#self_token)
		)
	};

	let tokens = gen_forwarded_trait
	(
		&base_type,
		type_info . parameters,
		type_info . predicates,
		&forwarded_trait,
		forwarded_trait_info,
		&delegated_type,
		receiver_predicates,
		receiver_transforms
	);

	Ok (tokens)
}

pub fn forward_trait_via_conversion_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_trait_via_conversion_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}

#[derive (Parse, ToTokens)]
struct ForwardTraitsViaConversion
{
	base_type_ident: Ident,
	r_arrow: Token! [->],
	delegated_type: Type,

	#[syn (parenthesized)]
	paren: Paren,

	#[syn (in = paren)]
	#[parse (Punctuated::parse_terminated)]
	forwarded_traits: Punctuated <Path, Token! [,]>
}

fn try_forward_traits_via_conversion_impl (input: proc_macro::TokenStream)
-> Result <proc_macro2::TokenStream>
{
	let ForwardTraitsViaConversion
	{
		base_type_ident,
		delegated_type,
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
				forward_traits::forward_trait_via_conversion,
				#base_type_ident,
				#delegated_type,
				#forwarded_trait
			);
		}
			. to_tokens (&mut tokens);
	}

	Ok (tokens)
}

pub fn forward_traits_via_conversion_impl (input: proc_macro::TokenStream)
-> proc_macro::TokenStream
{
	try_forward_traits_via_conversion_impl (input)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
