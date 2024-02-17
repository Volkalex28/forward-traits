use syn
::{
	Ident,
	Type,
	Path,
	PathArguments,
	Expr,
	WherePredicate,
	Signature,
	FnArg,
	Token,
	parse_quote
};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use quote::{quote, ToTokens};

use crate::syntax::TypedIdent;
use crate::info::generics::{ParameterInfo, ParameterValue};
use crate::info::TraitImplInfo;
use crate::uncurry::uncurry_macro_ident;

pub struct ReceiverTransforms
{
	pub transform_ref: Expr,
	pub transform_ref_mut: Expr,
	pub transform_owned: Expr
}

pub fn get_trait_parameter_values (forwarded_trait: &Path)
-> Result <Punctuated <ParameterValue, Token! [,]>>
{
	let last_segment = match forwarded_trait . segments . last ()
	{
		None => return Ok (Punctuated::new ()),
		Some (segment) => segment . clone ()
	};

	let mut parameter_values = Punctuated::new ();

	match last_segment . arguments
	{
		PathArguments::AngleBracketed (angle_arguments) =>
			for generic_arg in angle_arguments . args
			{
				parameter_values . push (ParameterValue::try_from (generic_arg)?);
			},
		PathArguments::None => {},
		PathArguments::Parenthesized (tokens) => return Err
		(
			Error::new_spanned (tokens, "Parenthesized arguments are invalid for forwardable traits")
		)
	}

	Ok (parameter_values)
}

fn get_trait_ident (trait_path: &Path) -> Result <Ident>
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

fn gen_forwarded_associated_type
(
	delegated_type: &Type,
	forwarded_trait: &Path,
	associated_type_ident: Ident
)
-> proc_macro2::TokenStream
{
	quote!
	{
		type #associated_type_ident =
			<#delegated_type as #forwarded_trait>::#associated_type_ident;
	}
}

fn gen_forwarded_method
(
	delegated_type: &Type,
	forwarded_trait: &Path,
	method_signature: Signature,
	receiver_transforms: &ReceiverTransforms
)
-> proc_macro2::TokenStream
{
	let Signature {asyncness, ident, generics, inputs, output, ..} =
		method_signature;

	let args = inputs
		. iter ()
		. map
		(
			|fn_arg| match fn_arg
			{
				FnArg::Receiver (receiver) => match
				(
					receiver . reference . is_some (),
					receiver . mutability . is_some ()
				)
				{
					(false, false) =>
						receiver_transforms . transform_owned . clone (),
					(false, true) =>
						receiver_transforms . transform_owned . clone (),
					(true, false) =>
						receiver_transforms . transform_ref . clone (),
					(true, true) =>
						receiver_transforms . transform_ref_mut . clone ()
				}
					. into_token_stream (),
				FnArg::Typed (pat_type) => pat_type . pat . to_token_stream ()
			}
		);

	let (impl_generics, type_generics, where_clause) =
		generics . split_for_impl ();

	quote!
	{
		fn #asyncness #ident #impl_generics (#inputs) -> #output
		#where_clause
		{
			<#delegated_type as #forwarded_trait>
				::#ident::#type_generics (#(#args),*)
		}
	}
}

fn gen_forwarded_const
(
	delegated_type: &Type,
	forwarded_trait: &Path,
	const_ident: Ident,
	const_type: Type
)
-> proc_macro2::TokenStream
{
	quote!
	{
		const #const_ident: #const_type =
			<#delegated_type as #forwarded_trait>::#const_ident;
	}
}

pub fn gen_forwarded_trait
(
	base_type: Type,
	type_parameters: Punctuated <ParameterInfo, Token! [,]>,
	type_predicates: Punctuated <WherePredicate, Token! [,]>,
	forwarded_trait: Path,
	trait_info: TraitImplInfo,
	delegated_type: Type,
	receiver_predicates: Vec <WherePredicate>,
	receiver_transforms: ReceiverTransforms
)
-> proc_macro2::TokenStream
{
	let mut predicates = type_predicates;
	predicates
		. extend (trait_info . predicates);
	<Punctuated <WherePredicate, Token! [,]> as Extend <WherePredicate>>
		::extend (&mut predicates, receiver_predicates);

	let where_clause =
		if predicates . is_empty () { proc_macro2::TokenStream::new () }
		else { quote! (where #predicates) };

	let forwarded_types = trait_info
		. associated_types
		. into_iter ()
		. map
		(
			|associated_type_ident| gen_forwarded_associated_type
			(
				&delegated_type,
				&forwarded_trait,
				associated_type_ident
			)
		);

	let forwarded_methods = trait_info
		. methods
		. into_iter ()
		. map
		(
			|method_signature| gen_forwarded_method
			(
				&delegated_type,
				&forwarded_trait,
				method_signature,
				&receiver_transforms
			)
		);

	let forwarded_constants = trait_info
		. associated_constants
		. into_iter ()
		. map
		(
			|TypedIdent {ident, ty, ..}| gen_forwarded_const
			(
				&delegated_type,
				&forwarded_trait,
				ident,
				ty
			)
		);

	quote!
	{
		#[automatically_generated]
		impl <#type_parameters> #forwarded_trait for #base_type
		#where_clause
		{
			#(#forwarded_types)*

			#(#forwarded_methods)*

			#(#forwarded_constants)*
		}
	}
}
