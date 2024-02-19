use syn
::{
	Type,
	Path,
	PathArguments,
	WherePredicate,
	Signature,
	FnArg,
	ReturnType,
	Token,
	parse_quote
};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use quote::{quote, ToTokens};

use crate::info::generics::{ParameterInfo, ParameterValue};
use crate::info
::{
	TraitAssociatedTypeInfo,
	TraitAssociatedConstInfo,
	TraitImplInfo
};

pub struct ReceiverTransforms <FR, FM, FO>
where
	FR: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream,
	FM: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream,
	FO: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream
{
	pub transform_ref: FR,
	pub transform_ref_mut: FM,
	pub transform_owned: FO
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

fn gen_forwarded_associated_type
(
	delegated_type: &Type,
	forwarded_trait: &Path,
	associated_type_info: TraitAssociatedTypeInfo
)
-> proc_macro2::TokenStream
{
	let TraitAssociatedTypeInfo {ident, generics, ..} = associated_type_info;

	let (impl_generics, type_generics, where_clause) =
		generics . split_for_impl ();

	quote!
	{
		type #ident #impl_generics =
			<#delegated_type as #forwarded_trait>::#ident #type_generics
		#where_clause;
	}
}

fn gen_forwarded_method <FR, FM, FO, FRT>
(
	delegated_type: &Type,
	forwarded_trait: &Path,
	method_signature: Signature,
	receiver_transforms: &ReceiverTransforms <FR, FM, FO>,
	return_transform: &FRT
)
-> proc_macro2::TokenStream
where
	FR: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream,
	FM: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream,
	FO: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream,
	FRT: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream
{
	let Signature {asyncness, ident, generics, inputs, output, ..} =
		method_signature;

	let args = inputs
		. iter ()
		. map
		(
			|fn_arg| match fn_arg
			{
				FnArg::Receiver (receiver) =>
				{
					let expr = receiver . self_token . to_token_stream ();

					if receiver . ty == parse_quote! (&Self)
					{
						(receiver_transforms . transform_ref) (expr)
					}
					else if receiver . ty == parse_quote! (&mut Self)
					{
						(receiver_transforms . transform_ref_mut) (expr)
					}
					else if receiver . ty == parse_quote! (Self)
					{
						(receiver_transforms . transform_owned) (expr)
					}
					else { unreachable! (); }
				}
				FnArg::Typed (pat_type) =>
				{
					let expr = pat_type . pat . to_token_stream ();

					if pat_type . ty == parse_quote! (&Self)
					{
						(receiver_transforms . transform_ref) (expr)
					}
					else if pat_type . ty == parse_quote! (&mut Self)
					{
						(receiver_transforms . transform_ref_mut) (expr)
					}
					else if pat_type . ty == parse_quote! (Self)
					{
						(receiver_transforms . transform_owned) (expr)
					}
					else { expr }
				}
			}
		);

	let (impl_generics, type_generics, where_clause) =
		generics . split_for_impl ();

	let type_generics = type_generics . as_turbofish ();

	let body_expr = quote!
	(
		<#delegated_type as #forwarded_trait>::#ident #type_generics (#(#args),*)
	);

	let body_expr = match output
	{
		ReturnType::Type (_, ref boxed_type) if **boxed_type == parse_quote! (Self) =>
			return_transform (body_expr),
		_ => body_expr
	};

	let tokens = quote!
	{
		#asyncness fn #ident #impl_generics (#inputs) #output
		#where_clause
		{
			#body_expr
		}
	};

	tokens
}

fn gen_forwarded_const
(
	delegated_type: &Type,
	forwarded_trait: &Path,
	associated_const_info: TraitAssociatedConstInfo
)
-> proc_macro2::TokenStream
{
	let TraitAssociatedConstInfo {ident, generics, ty, ..} =
		associated_const_info;

	let (impl_generics, type_generics, where_clause) =
		generics . split_for_impl ();

	quote!
	{
		const #ident #impl_generics: #ty =
			<#delegated_type as #forwarded_trait>::#ident #type_generics
		#where_clause;
	}
}

pub fn gen_forwarded_trait <FR, FM, FO, FRT>
(
	base_type: &Type,
	type_parameters: Punctuated <ParameterInfo, Token! [,]>,
	type_predicates: Punctuated <WherePredicate, Token! [,]>,
	forwarded_trait: &Path,
	trait_info: TraitImplInfo,
	delegated_type: &Type,
	receiver_predicates: Vec <WherePredicate>,
	receiver_transforms: ReceiverTransforms <FR, FM, FO>,
	return_transform: FRT
)
-> proc_macro2::TokenStream
where
	FR: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream,
	FM: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream,
	FO: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream,
	FRT: Fn (proc_macro2::TokenStream) -> proc_macro2::TokenStream
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
			|associated_type| gen_forwarded_associated_type
			(
				delegated_type,
				forwarded_trait,
				associated_type
			)
		);

	let forwarded_methods = trait_info
		. methods
		. into_iter ()
		. map
		(
			|method_signature| gen_forwarded_method
			(
				delegated_type,
				forwarded_trait,
				method_signature,
				&receiver_transforms,
				&return_transform
			)
		);

	let forwarded_constants = trait_info
		. associated_constants
		. into_iter ()
		. map
		(
			|associated_const| gen_forwarded_const
			(
				delegated_type,
				forwarded_trait,
				associated_const
			)
		);

	quote!
	{
		#[automatically_derived]
		impl <#type_parameters> #forwarded_trait for #base_type
		#where_clause
		{
			#(#forwarded_types)*

			#(#forwarded_methods)*

			#(#forwarded_constants)*
		}
	}
}
