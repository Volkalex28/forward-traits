use std::collections::HashMap;

use syn
::{
	Lifetime,
	Type,
	Expr,
	PathArguments,
	TypeParam,
	ConstParam,
	Token,
	parse_quote
};
use syn::fold
::{
	Fold,
	fold_lifetime,
	fold_type,
	fold_type_param,
	fold_expr,
	fold_const_param
};

use crate::syn::{associated_type::AssociatedType, from_type::FromType};

use crate::type_transformer
::{
	independent_type_transformer::IndependentTypeTransformer,
	associated_type_transformer::AssociatedTypeTransformer,
	additional_type_transformer::AdditionalTypeTransformer
};

use super::parameter_info::ParameterInfo;
use super::parameter_value::ParameterValue;

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

	pub fn fold_partial_eval (&mut self, node: PartialEval) -> PartialEval
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

	pub fn fold_additional_type_transformer
	(
		&mut self,
		node: AdditionalTypeTransformer
	)
	-> AdditionalTypeTransformer
	{
		AdditionalTypeTransformer
		{
			lifetimes: node . lifetimes . map
			(
				|bound_lifetimes| self . fold_bound_lifetimes (bound_lifetimes)
			),
			from_type: self . fold_from_type (node . from_type),
			transform_type: node . transform_type,
			to_type: self . fold_type (node . to_type)
		}
	}

	pub fn fold_independent_type_transformer
	(
		&mut self,
		node: IndependentTypeTransformer
	)
	-> IndependentTypeTransformer
	{
		IndependentTypeTransformer
		{
			lifetimes: node . lifetimes . map
			(
				|bound_lifetimes| self . fold_bound_lifetimes (bound_lifetimes)
			),
			from_type: self . fold_type (node . from_type),
			to_type: self . fold_type (node . to_type),
			value_transformer: node . value_transformer
		}
	}

	#[allow (dead_code)]
	pub fn fold_associated_type_transformer
	(
		&mut self,
		node: AssociatedTypeTransformer
	)
	-> AssociatedTypeTransformer
	{
		AssociatedTypeTransformer
		{
			lifetimes: node . lifetimes . map
			(
				|bound_lifetimes| self . fold_bound_lifetimes (bound_lifetimes)
			),
			associated_type: self . fold_associated_type (node . associated_type),
			replacement_type: self . fold_type (node . replacement_type),
			value_transformer: node . value_transformer
		}
	}

	pub fn fold_from_type (&mut self, node: FromType) -> FromType
	{
		match node
		{
			FromType::Independent (ty) =>
				FromType::Independent (self . fold_type (ty)),
			FromType::Associated (associated_type) =>
				FromType::Associated (self . fold_associated_type (associated_type))
		}
	}

	pub fn fold_associated_type (&mut self, node: AssociatedType)
	-> AssociatedType
	{
		AssociatedType
		{
			self_token: node . self_token,
			double_colon_token: node . double_colon_token,
			ident: node . ident,
			generics: self . fold_generics (node . generics)
		}
	}
}

macro_rules! make_type_key
{
	($ident: expr) => { &ParameterInfo::Type ($ident) }
}

macro_rules! make_const_key
{
	($ident: expr) =>
	{
		&ParameterInfo::Const (<Token! [const]>::default (), $ident)
	}
}

macro_rules! fold_qpath
{
	($fold_qpath: ident, $QPath: ident, $PVariant: ident, $make_key: ident) =>
	{
		fn $fold_qpath (&mut self, node: $QPath) -> $QPath
		{
			if let $QPath::Path (qpath) = &node
			{
				if qpath . qself . is_none ()
				{
					if let Some (ident) = qpath . path . get_ident ()
					{
						if let Some (ParameterValue::$PVariant (ty)) = self
							. parameters
							. get ($make_key! (ident . clone ()))
						{
							return ty . clone ();
						}
					}
				}

				if let Some (first_segment) = qpath . path . segments . first ()
				{
					if let PathArguments::None = first_segment . arguments
					{
						let maybe_parameter_value = self
							. parameters
							. get ($make_key! (first_segment . ident . clone ()))
							. cloned ();

						if let Some (ParameterValue::$PVariant (ty)) =
							maybe_parameter_value
						{
							let tail_segments = qpath
								. path
								. segments
								. iter ()
								. skip (1)
								. cloned ()
								. map (|segment| self . fold_path_segment (segment));

							return parse_quote! (<#ty>#(::#tail_segments)*);
						}
					}
				}
			}

			$fold_qpath (self, node)
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

	fold_qpath! (fold_type, Type, Type, make_type_key);

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

	fold_qpath! (fold_expr, Expr, Const, make_const_key);

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
