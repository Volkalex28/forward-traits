use std::collections::HashMap;

use syn
::{
	Type,
	Lifetime,
	Expr,
	Path,
	PathArguments,
	Signature,
	WherePredicate,
	Token,
	parse_quote
};
use syn::punctuated::Punctuated;
use syn::fold::{Fold, fold_lifetime, fold_type, fold_path, fold_expr};

use super::generics::{ParameterInfo, ParameterValue};

use crate::syntax::TypedIdent;

pub struct PartialEval
{
	pub parameters: HashMap <ParameterInfo, ParameterValue>
}

macro_rules! fn_fold_punctuated
{
	($fn_name:ident, $T:ty, $P:ty, $t_fn_name:ident) =>
	{
		pub fn $fn_name (&mut self, node: Punctuated <$T, $P>)
		-> Punctuated <$T, $P>
		{
			node . into_iter () . map (|item| self . $t_fn_name (item)) . collect ()
		}
	}
}

impl PartialEval
{
	pub fn fold_parameter_value (&mut self, node: ParameterValue) -> ParameterValue
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

	pub fn fold_typed_ident (&mut self, node: TypedIdent) -> TypedIdent
	{
		TypedIdent
		{
			ident: node . ident,
			colon: node . colon,
			ty: self . fold_type (node . ty)
		}
	}

	fn_fold_punctuated! (fold_parameter_values, ParameterValue, Token! [,], fold_parameter_value);
	fn_fold_punctuated! (fold_predicates, WherePredicate, Token! [,], fold_where_predicate);

	fn_fold_punctuated! (fold_methods, Signature, Token! [;], fold_signature);
	fn_fold_punctuated! (fold_associated_constants, TypedIdent, Token! [,], fold_typed_ident);
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
							&ParameterInfo::TypeOrConst (ident . clone ())
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
			. get (&ParameterInfo::TypeOrConst (first_segment . ident . clone ()))
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
							&ParameterInfo::TypeOrConst (ident . clone ())
						)
					{
						return expr . clone ();
					}
				}
			}
		}

		fold_expr (self, node)
	}
}
