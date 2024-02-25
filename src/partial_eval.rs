use std::collections::HashMap;
use std::iter::repeat;

use syn
::{
	Ident,
	Lifetime,
	Type,
	Expr,
	Path,
	PathArguments,
	Generics,
	GenericArgument,
	GenericParam,
	TypeParam,
	ConstParam,
	Token,
	parse_quote
};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn::fold
::{
	Fold,
	fold_lifetime,
	fold_type,
	fold_path,
	fold_type_param,
	fold_expr,
	fold_const_param
};
use syn_derive::{Parse, ToTokens};

use crate::generics::get_num_required_arguments;

#[derive (Clone, PartialEq, Eq, Hash, Parse, ToTokens)]
pub enum ParameterInfo
{
	#[parse (peek = Lifetime)]
	Lifetime (Lifetime),

	#[parse (peek = Ident)]
	Type (Ident),

	#[parse (peek = Token! [const])]
	Const (Token! [const], Ident)
}

impl From <GenericParam> for ParameterInfo
{
	fn from (generic_param: GenericParam) -> Self
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) =>
				ParameterInfo::Lifetime (lifetime_param . lifetime),
			GenericParam::Type (type_param) =>
				ParameterInfo::Type (type_param . ident),
			GenericParam::Const (const_param) =>
				ParameterInfo::Const (const_param . const_token, const_param . ident)
		}
	}
}

#[derive (Clone, PartialEq, Eq, Hash, Parse, ToTokens)]
pub enum ParameterValue
{
	#[parse (peek = Lifetime)]
	Lifetime (Lifetime),

	#[parse (peek_func = |input| input . fork () . parse::<Type> () . is_ok ())]
	Type (Type),

	Const (Expr)
}

impl TryFrom <GenericParam> for ParameterValue
{
	type Error = Error;

	fn try_from (generic_param: GenericParam) -> Result <Self>
	{
		match generic_param
		{
			GenericParam::Lifetime (lifetime_param) => Err
			(
				Error::new_spanned
				(
					lifetime_param,
					"Lifetime parameters cannot have default values"
				)
			),
			GenericParam::Type (type_param) => if let Some (ty) = type_param . default
			{
				Ok (ParameterValue::Type (ty))
			}
			else
			{
				Err
				(
					Error::new_spanned
					(
						type_param,
						"Type parameter lacks a default argument"
					)
				)
			},
			GenericParam::Const (const_param) => if let Some (expr) = const_param . default
			{
				Ok (ParameterValue::Const (expr))
			}
			else
			{
				Err
				(
					Error::new_spanned
					(
						const_param,
						"Const parameter lacks a default_argument"
					)
				)
			}
		}
	}
}

impl TryFrom <GenericArgument> for ParameterValue
{
	type Error = Error;

	fn try_from (generic_argument: GenericArgument) -> Result <Self>
	{
		match generic_argument
		{
			GenericArgument::Lifetime (lifetime) =>
				Ok (ParameterValue::Lifetime (lifetime)),
			GenericArgument::Type (ty) => Ok (ParameterValue::Type (ty)),
			GenericArgument::Const (expr) => Ok (ParameterValue::Const (expr)),
			_ => Err
			(
				Error::new_spanned
				(
					generic_argument,
					"Constraints make no sense in this context"
				)
			)
		}
	}
}

impl <'a> From <&'a ParameterInfo> for ParameterValue
{
	fn from (info: &'a ParameterInfo) -> Self
	{
		match info
		{
			ParameterInfo::Lifetime (lifetime) =>
				ParameterValue::Lifetime (lifetime . clone ()),
			ParameterInfo::Type (ident) =>
				ParameterValue::Type (parse_quote! (#ident)),
			ParameterInfo::Const (_, ident) =>
				ParameterValue::Const (parse_quote! (#ident))
		}
	}
}

#[derive (Clone, PartialEq, Eq)]
pub struct PartialEval
{
	pub parameters: HashMap <ParameterInfo, ParameterValue>
}

impl PartialEval
{
	pub fn new () -> Self
	{
		Self {parameters: HashMap::new ()}
	}

	fn fold_parameter_value (&mut self, node: ParameterValue) -> ParameterValue
	{
		match node
		{
			ParameterValue::Lifetime (lifetime) =>
				ParameterValue::Lifetime (self . fold_lifetime (lifetime)),
			ParameterValue::Type (ty) =>
				ParameterValue::Type (self . fold_type (ty)),
			ParameterValue::Const (expr) =>
				ParameterValue::Const (self . fold_expr (expr))
		}
	}

	fn fold_partial_eval (&mut self, node: PartialEval) -> PartialEval
	{
		PartialEval
		{
			parameters: node
				. parameters
				. into_iter ()
				. map (|(info, value)| (info, self . fold_parameter_value (value)))
				. collect ()
		}
	}
}

impl Fold for PartialEval
{
	fn fold_lifetime (&mut self, node: Lifetime) -> Lifetime
	{
		if let Some (ParameterValue::Lifetime (lifetime)) =
			self . parameters . get (&ParameterInfo::Lifetime (node . clone ()))
		{
			return lifetime . clone ();
		}

		fold_lifetime (self, node)
	}

	fn fold_type (&mut self, node: Type) -> Type
	{
		if let Type::Path (ref type_path) = node
		{
			if type_path . qself . is_none ()
			{
				if let Some (ident) = type_path . path . get_ident ()
				{
					if let Some (ParameterValue::Type (ty)) =
						self . parameters . get
						(
							&ParameterInfo::Type (ident . clone ())
						)
					{
						return ty . clone ();
					}
				}
			}
		}

		fold_type (self, node)
	}

	fn fold_path (&mut self, node: Path) -> Path
	{
		if node . leading_colon . is_some () { return fold_path (self, node); }

		let first_segment = match node . segments . first ()
		{
			None => return fold_path (self, node),
			Some (first_segment) => first_segment
		};

		match first_segment . arguments
		{
			PathArguments::None => {},
			_ => return fold_path (self, node)
		};

		let ty = match self
			. parameters
			. get (&ParameterInfo::Type (first_segment . ident . clone ()))
		{
			Some (ParameterValue::Type (ty)) => ty,
			_ => return fold_path (self, node),
		};

		let mut new_path: Path = parse_quote! (<#ty>);
		new_path . segments . extend
		(
			node
				. segments
				. into_iter ()
				. skip (1)
				. map (|segment| self . fold_path_segment (segment))
		);
		new_path
	}

	fn fold_type_param (&mut self, node: TypeParam) -> TypeParam
	{
		if let Some (ParameterValue::Type (ty)) = self
			. parameters
			. get (&ParameterInfo::Type (node . ident . clone ()))
		{
			if let Type::Path (type_path) = ty
			{
				if type_path . qself . is_none ()
				{
					if let Some (ident) = type_path . path . get_ident ()
					{
						return TypeParam
						{
							attrs: node . attrs,
							ident: ident . clone (),
							colon_token: node . colon_token,
							bounds: node
								. bounds
								. into_iter ()
								. map (|bound| self . fold_type_param_bound (bound))
								. collect (),
							eq_token: node . eq_token,
							default: node
								. default
								. map (|ty| self . fold_type (ty))
						};
					}
				}
			}
		}

		fold_type_param (self, node)
	}

	fn fold_expr (&mut self, node: Expr) -> Expr
	{
		if let Expr::Path (ref expr_path) = node
		{
			if expr_path . qself . is_none ()
			{
				if let Some (ident) = expr_path . path . get_ident ()
				{
					if let Some (ParameterValue::Const (expr)) =
						self . parameters . get
						(
							&ParameterInfo::Const
							(
								<Token! [const]>::default (),
								ident . clone ()
							)
						)
					{
						return expr . clone ();
					}
				}
			}
		}

		fold_expr (self, node)
	}

	fn fold_const_param (&mut self, node: ConstParam) -> ConstParam
	{
		if let Some (ParameterValue::Const (expr)) = self
			. parameters
			. get
			(
				&ParameterInfo::Const
				(
					<Token! [const]>::default (),
					node . ident . clone ()
				)
			)
		{
			if let Expr::Path (expr_path) = expr
			{
				if expr_path . qself . is_none ()
				{
					if let Some (ident) = expr_path . path . get_ident ()
					{
						return ConstParam
						{
							attrs: node . attrs,
							const_token: node . const_token,
							ident: ident . clone (),
							colon_token: node . colon_token,
							ty: self . fold_type (node . ty),
							eq_token: node . eq_token,
							default: node
								. default
								. map (|expr| self . fold_expr (expr))
						};
					}
				}
			}
		}

		fold_const_param (self, node)
	}
}

pub fn get_evaluator (trait_generics: Generics, trait_path: &Path)
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
				ParameterValue::try_from (trait_parameter)?
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
