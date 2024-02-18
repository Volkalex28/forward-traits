use syn
::{
	Attribute,
	Visibility,
	Ident,
	Signature,
	WherePredicate,
	ItemTrait,
	TraitItem,
	ItemUse,
	Token,
	parse_quote,
	parse
};
use syn::token::{Paren, Brace, Bracket};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn::fold::Fold;
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

use super::generics::{ParameterInfo, ParameterValue, parse_generics};
use super::partial_eval::PartialEval;
use super::transform_use::TransformUse;
use super::TraitImplInfo;
use crate::syntax::{TypedIdent, kw};
use crate::uncurry::{uncurry_macro_ident, gen_uncurry_macro};

#[derive (Parse, ToTokens)]
pub struct TraitDefInfo
{
	trait_info_kw: kw::trait_info,

	#[syn (parenthesized)]
	p_paren: Paren,
	#[syn (in = p_paren)]
	#[parse (Punctuated::parse_terminated)]
	parameters: Punctuated <ParameterInfo, Token! [,]>,

	#[syn (parenthesized)]
	d_paren: Paren,
	#[syn (in = d_paren)]
	#[parse (Punctuated::parse_terminated)]
	default_values: Punctuated <ParameterValue, Token! [,]>,

	#[syn (bracketed)]
	p_bracket: Bracket,
	#[syn (in = p_bracket)]
	#[parse (Punctuated::parse_terminated)]
	predicates: Punctuated <WherePredicate, Token! [,]>,

	#[syn (braced)]
	t_brace: Brace,
	#[syn (in = t_brace)]
	#[parse (Punctuated::parse_terminated)]
	associated_types: Punctuated <Ident, Token! [,]>,

	#[syn (braced)]
	m_brace: Brace,
	#[syn (in = m_brace)]
	#[parse (Punctuated::parse_terminated)]
	methods: Punctuated <Signature, Token! [,]>,

	#[syn (braced)]
	c_brace: Brace,
	#[syn (in = c_brace)]
	#[parse (Punctuated::parse_terminated)]
	associated_constants: Punctuated <TypedIdent, Token! [,]>
}

impl TraitDefInfo
{
	pub fn substitute
	(
		self,
		parameter_values: Punctuated <ParameterValue, Token! [,]>
	)
	-> Result <TraitImplInfo>
	{
		if self . parameters . is_empty ()
		{
			if ! parameter_values . is_empty ()
			{
				return Err
				(
					Error::new_spanned
					(
						parameter_values,
						"Forwarded trait does not take generic parameters"
					)
				);
			}

			return Ok
			(
				TraitImplInfo
				{
					predicates: self . predicates,
					associated_types: self . associated_types,
					methods: self . methods,
					associated_constants: self . associated_constants
				}
			);
		}

		let num_values = parameter_values . len ();
		let num_parameters = self . parameters . len ();

		if num_values > num_parameters
		{
			return Err
			(
				Error::new_spanned
				(
					parameter_values,
					format!
					(
						"Forwarded trait only takes {} parameters, {} were provided",
						num_parameters,
						num_values
					)
				)
			);
		}

		let num_default_values = self . default_values . len ();

		if (num_values + num_default_values) < num_parameters
		{
			return Err
			(
				Error::new_spanned
				(
					parameter_values,
					format!
					(
						"Forwarded trait requires {} parameters, {} were provided",
						num_parameters - num_default_values,
						num_values
					)
				)
			);
		}

		let num_defaults_needed = num_parameters - num_values;

		let mut p_eval = if num_defaults_needed == 0
		{
			PartialEval
			{
				parameters: self
					. parameters
					. into_iter ()
					. zip (parameter_values)
					. collect ()
			}
		}
		else
		{
			let defaulted_parameters: Vec <_> = self
				. parameters
				. iter ()
				. skip (num_values)
				. cloned ()
				. collect ();

			let defaulted_values: Punctuated <_, Token! [,]> = self
				. default_values
				. into_iter ()
				. skip (num_default_values - num_defaults_needed)
				. collect ();

			let mut p_eval = PartialEval
			{
				parameters: self
					. parameters
					. into_iter ()
					. zip (parameter_values)
					. collect ()
			};

			let defaulted_values = p_eval . fold_parameter_values (defaulted_values);

			p_eval . parameters . extend
			(
				defaulted_parameters . into_iter () . zip (defaulted_values)
			);

			p_eval
		};

		let predicates = p_eval . fold_predicates (self . predicates);
		let associated_types = self . associated_types;
		let methods = p_eval . fold_methods (self . methods);
		let associated_constants =
			p_eval . fold_associated_constants (self . associated_constants);

		Ok
		(
			TraitImplInfo
			{
				predicates,
				associated_types,
				methods,
				associated_constants
			}
		)
	}
}

impl TryFrom <ItemTrait> for TraitDefInfo
{
	type Error = Error;

	fn try_from (item_trait: ItemTrait) -> Result <TraitDefInfo>
	{
		let (parameters, default_values, mut predicates) =
			parse_generics (item_trait . generics);

		if ! item_trait . supertraits . is_empty ()
		{
			let supertraits = item_trait . supertraits;
			predicates . push (parse_quote! (Self: #supertraits))
		}

		let mut associated_types = Punctuated::new ();
		let mut methods = Punctuated::new ();
		let mut associated_constants = Punctuated::new ();

		for item in item_trait . items
		{
			match item
			{
				TraitItem::Const (associated_constant) =>
				{
					associated_constants . push
					(
						TypedIdent::new
						(
							associated_constant . ident,
							associated_constant . ty
						)
					);
				},
				TraitItem::Fn (method) =>
				{
					if let Some (receiver) = method . sig . receiver ()
					{
						if receiver . ty != parse_quote! (&Self)
							&& receiver . ty != parse_quote! (&mut Self)
							&& receiver . ty != parse_quote! (Self)
						{
							return Err
							(
								Error::new_spanned
								(
									receiver . ty . as_ref (),
									"Containerized receivers are not supported"
								)
							);
						}
					}

					methods . push (method . sig);
				},
				TraitItem::Type (associated_type) =>
				{
					associated_types . push (associated_type . ident);
				},
				_ => {}
			}
		}

		let trait_def_info = TraitDefInfo
		{
			trait_info_kw: kw::trait_info::default (),

			p_paren: Paren::default (),
			parameters,

			d_paren: Paren::default (),
			default_values,

			p_bracket: Bracket::default (),
			predicates,

			t_brace: Brace::default (),
			associated_types,

			m_brace: Brace::default (),
			methods,

			c_brace: Brace::default (),
			associated_constants
		};

		Ok (trait_def_info)
	}
}

#[derive (Parse)]
#[parse (
	prefix = |parse_stream|
	{
		Attribute::parse_outer (parse_stream)?;
		parse_stream . parse::<Visibility> ()?;
		Ok (())
	}
)]
enum ForwardableItem
{
	#[parse (peek = Token! [trait])]
	ItemTrait (ItemTrait),

	#[parse (peek = Token! [use])]
	ItemUse (ItemUse)
}

fn try_forwardable_impl
(
	_attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> Result <proc_macro2::TokenStream>
{
	let mut tokens = proc_macro2::TokenStream::from (item . clone ());

	match parse (item)?
	{
		ForwardableItem::ItemTrait (item_trait) =>
		{
			let vis = item_trait . vis . clone ();

			let macro_ident = uncurry_macro_ident (&item_trait . ident);

			let trait_def_info = TraitDefInfo::try_from (item_trait)?;

			tokens . extend (gen_uncurry_macro (vis, macro_ident, trait_def_info));
		},
		ForwardableItem::ItemUse (item_use) =>
		{
			TransformUse {} . fold_item_use (item_use) . to_tokens (&mut tokens);
		}
	}

	Ok (tokens)
}

pub fn forwardable_impl
(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream
)
-> proc_macro::TokenStream
{
	try_forwardable_impl (attr, item)
		. unwrap_or_else (Error::into_compile_error)
		. into ()
}
