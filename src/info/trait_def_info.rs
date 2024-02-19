use syn
::{
	Attribute,
	Visibility,
	Ident,
	Type,
	Signature,
	Generics,
	WherePredicate,
	ItemTrait,
	TraitItem,
	TraitItemType,
	TraitItemConst,
	ItemUse,
	Token,
	parse_quote,
	parse
};
use syn::ext::IdentExt;
use syn::token::{Paren, Brace, Bracket};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result, Error};
use syn::fold::Fold;
use syn_derive::{Parse, ToTokens};
use quote::ToTokens;

use super::generics::{ParameterInfo, ParameterValue, parse_generics};
use super::partial_eval::PartialEval;
use super::transform_use::TransformUse;
use super::TraitImplInfo;
use crate::syntax::kw;
use crate::uncurry::{uncurry_macro_ident, gen_uncurry_macro};

pub struct TraitAssociatedTypeInfo
{
	pub type_token: Token! [type],
	pub ident: Ident,
	pub generics: Generics
}

impl Parse for TraitAssociatedTypeInfo
{
	fn parse (input: ParseStream <'_>) -> Result <Self>
	{
		let type_token = input . parse ()?;
		let ident = input . parse ()?;
		let mut generics: Generics = input . parse ()?;
		generics . where_clause = input . parse ()?;

		Ok (Self {type_token, ident, generics})
	}
}

impl ToTokens for TraitAssociatedTypeInfo
{
	fn to_tokens (&self, tokens: &mut proc_macro2::TokenStream)
	{
		self . type_token . to_tokens (tokens);
		self . ident . to_tokens (tokens);
		self . generics . to_tokens (tokens);
		self . generics . where_clause . to_tokens (tokens);
	}
}

impl From <TraitItemType> for TraitAssociatedTypeInfo
{
	fn from (item_type: TraitItemType) -> Self
	{
		let TraitItemType {type_token, ident, generics, ..} = item_type;

		Self {type_token, ident, generics}
	}
}

pub struct TraitAssociatedConstInfo
{
	pub const_token: Token! [const],
	pub ident: Ident,
	pub generics: Generics,
	pub colon_token: Token! [:],
	pub ty: Type
}

impl Parse for TraitAssociatedConstInfo
{
	fn parse (input: ParseStream <'_>) -> Result <Self>
	{
		let const_token = input . parse ()?;

		let lookahead = input . lookahead1 ();
		let ident = if lookahead . peek (Ident) || lookahead . peek (Token! [_])
		{
			Ident::parse_any (input)?
		}
		else
		{
			return Err (lookahead . error ());
		};

		let mut generics: Generics = input . parse ()?;
		let colon_token = input . parse ()?;
		let ty = input . parse ()?;
		generics . where_clause = input . parse ()?;

		Ok (Self {const_token, ident, generics, colon_token, ty})
	}
}

impl ToTokens for TraitAssociatedConstInfo
{
	fn to_tokens (&self, tokens: &mut proc_macro2::TokenStream)
	{
		self . const_token . to_tokens (tokens);
		self . ident . to_tokens (tokens);
		self . generics . to_tokens (tokens);
		self . colon_token . to_tokens (tokens);
		self . ty . to_tokens (tokens);
		self . generics . where_clause . to_tokens (tokens);
	}
}

impl From <TraitItemConst> for TraitAssociatedConstInfo
{
	fn from (item_const: TraitItemConst) -> Self
	{
		let TraitItemConst {const_token, ident, generics, colon_token, ty, ..}
			= item_const;

		Self {const_token, ident, generics, colon_token, ty}
	}
}

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
	associated_types: Punctuated <TraitAssociatedTypeInfo, Token! [;]>,

	#[syn (braced)]
	m_brace: Brace,
	#[syn (in = m_brace)]
	#[parse (Punctuated::parse_terminated)]
	methods: Punctuated <Signature, Token! [;]>,

	#[syn (braced)]
	c_brace: Brace,
	#[syn (in = c_brace)]
	#[parse (Punctuated::parse_terminated)]
	associated_constants: Punctuated <TraitAssociatedConstInfo, Token! [;]>
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
		let associated_types =
			p_eval . fold_associated_types (self . associated_types);
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
				TraitItem::Type (associated_type) =>
				{
					associated_types . push
					(
						TraitAssociatedTypeInfo::from (associated_type)
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
				TraitItem::Const (associated_constant) =>
				{
					associated_constants . push
					(
						TraitAssociatedConstInfo::from (associated_constant)
					);
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
