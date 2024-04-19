use std::iter::repeat;

use proc_macro2::TokenStream;
use syn::{Path, PathArguments, Generics, GenericParam};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use quote::ToTokens;

use crate::generics::get_num_required_arguments;

use super::parameter_info::ParameterInfo;
use super::parameter_value::ParameterValue;
use super::partial_eval::PartialEval;

pub fn get_trait_path_evaluator (trait_generics: Generics, trait_path: &Path)
-> Result <PartialEval>
{
	let trait_arguments = if let Some (segment) =
		trait_path . segments . last ()
	{
		match &segment . arguments
		{
			PathArguments::AngleBracketed (arguments) =>
				arguments . args . clone (),
			PathArguments::Parenthesized (_) => return Err
			(
				Error::new_spanned (trait_path, "Fn* traits cannot be forwarded")
			),
			_ => Punctuated::new ()
		}
	}
	else
	{
		return Err
		(
			Error::new_spanned (trait_path, "Path to trait must be nonempty")
		);
	};

	let num_provided_arguments = trait_arguments . len ();
	let num_available_arguments = trait_generics . params . len ();
	let num_required_arguments = get_num_required_arguments (&trait_generics);

	if num_provided_arguments < num_required_arguments
	{
		return Err
		(
			Error::new_spanned
			(
				trait_arguments,
				format!
				(
					"Trait requires {} arguments, {} were provided",
					num_required_arguments,
					num_provided_arguments
				)
			)
		);
	}

	if num_provided_arguments > num_available_arguments
	{
		return Err
		(
			Error::new_spanned
			(
				trait_arguments,
				format!
				(
					"Trait only takes {} arguments, {} were provided",
					num_available_arguments,
					num_provided_arguments
				)
			)
		);
	}

	let mut evaluator = PartialEval::new ();

	for (trait_parameter, trait_argument)
	in trait_generics
		. params
		. iter ()
		. cloned ()
		. zip
		(
			trait_arguments
				. into_iter ()
				. map (Option::from)
				. chain (repeat (None))
		)
	{
		if let Some (trait_argument) = trait_argument
		{
			evaluator . parameters . insert
			(
				ParameterInfo::from (trait_parameter),
				ParameterValue::try_from (trait_argument)?
			);
		}
		else
		{
			evaluator . parameters . insert
			(
				ParameterInfo::from (trait_parameter . clone ()),

				// If someone somehow manages to mix the parameters in silly
				// ways, attempting to pull the default arguments could still
				// fail.
				ParameterValue::try_from_default_value (trait_parameter)?
			);
		}
	}

	// In the event that the default arguments contain references to other
	// generic parameters, we've got to substitute in all of those values
	// properly.  This could theoretically take an unbounded number of steps,
	// though most of the time it should take about 1 in practice, with 1 more
	// to verify that there are no more substitutions needed.
	const MAX_ITERATIONS: usize = 100;
	let mut num_iterations = 0;
	loop
	{
		let new_evaluator = evaluator . fold_partial_eval (evaluator . clone ());

		num_iterations += 1;

		if new_evaluator == evaluator
		{
			return Ok (evaluator);
		}

		if num_iterations >= MAX_ITERATIONS
		{
			return Err
			(
				Error::new_spanned
				(
					trait_generics,
					"Iteration limit reached evaluating default arguments"
				)
			);
		}

		evaluator = new_evaluator;
	}
}

pub fn get_associated_type_evaluator
(
	associated_type_generics: &Generics,
	trait_def_generics: &Generics
)
-> Result <PartialEval>
{
	if associated_type_generics . params . len ()
		!= trait_def_generics . params . len ()
	{
		return Err
		(
			Error::new_spanned
			(
				TokenStream::from_iter
				([
					associated_type_generics . to_token_stream (),
					trait_def_generics . to_token_stream ()
				]),
				"Parameter count mis-match"
			)
		);
	}

	let mut evaluator = PartialEval::new ();

	for (associated_type_parameter, trait_def_parameter)
	in associated_type_generics
		. params
		. iter ()
		. zip (trait_def_generics . params . iter ())
	{

		match (associated_type_parameter, trait_def_parameter)
		{
			(GenericParam::Lifetime (_), GenericParam::Lifetime (_)) => {},
			(GenericParam::Type (_), GenericParam::Type (_)) => {},
			(GenericParam::Const (_), GenericParam::Const (_)) => {},
			_ => return Err
			(
				Error::new_spanned
				(
					TokenStream::from_iter
					([
						associated_type_parameter . to_token_stream (),
						trait_def_parameter . to_token_stream ()
					]),
					"Parameter type mis-match"
				)
			)
		}

		evaluator . parameters . insert
		(
			ParameterInfo::from (associated_type_parameter . clone ()),
			ParameterValue::from (trait_def_parameter . clone ())
		);
	}

	return Ok (evaluator);
}
