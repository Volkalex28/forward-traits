use syn
::{
	Ident,
	Type,
	Expr,
	Path,
	Signature,
	FnArg,
	Receiver,
	PatType,
	ReturnType,
	TraitItem,
	TraitItemConst,
	TraitItemFn,
	TraitItemType,
	ImplItem,
	ImplItemConst,
	ImplItemFn,
	ImplItemType,
	WherePredicate,
	Index,
	Token,
	parse_quote,
	parse2
};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use syn::fold::Fold;
use quote::{quote, ToTokens};

use crate::syn::transformable_types::*;

use crate::value_transformer::value_transformer::ValueTransformer;

use crate::type_transformer
::{
	associated_type_transformer::*,
	independent_type_transformer::*,
	additional_type_transformer::*
};

pub struct TransformerBuilder
{
	associated_type_transformers: AssociatedTypeTransformers,
	independent_type_transformers: IndependentTypeTransformers
}

impl TransformerBuilder
{
	pub fn new () -> Self
	{
		Self
		{
			associated_type_transformers: AssociatedTypeTransformers::new (),
			independent_type_transformers: IndependentTypeTransformers::new ()
		}
	}

	pub fn add_independent_type_transformer
	(
		&mut self,
		independent_type_transformer: IndependentTypeTransformer
	)
	{
		self
			. independent_type_transformers
			. insert (independent_type_transformer);
	}

	pub fn add_additional_type_transformer
	(
		&mut self,
		additional_type_transformer: AdditionalTypeTransformer
	)
	{
		match additional_type_transformer . specialize ()
		{
			SpecializedTypeTransformer::Independent (independent_type_transformer) =>
				self . independent_type_transformers . insert (independent_type_transformer),
			SpecializedTypeTransformer::Associated (associated_type_transformer) =>
				self . associated_type_transformers . insert (associated_type_transformer)
		}
	}

	pub fn into_transformer (self, delegated_type: Type, forwarded_trait: Path)
	-> Transformer
	{
		let Self {associated_type_transformers, independent_type_transformers} = self;

		let forwarded_trait = independent_type_transformers
			. get_type_transformer ()
			. fold_path (forwarded_trait);

		Transformer
		{
			associated_type_transformers,
			independent_type_transformers,
			delegated_type,
			forwarded_trait
		}
	}
}

pub struct Transformer
{
	associated_type_transformers: AssociatedTypeTransformers,
	independent_type_transformers: IndependentTypeTransformers,
	delegated_type: Type,
	forwarded_trait: Path
}

impl Transformer
{
	pub fn get_transformed_forwarded_trait (&self) -> &Path
	{
		&self . forwarded_trait
	}

	fn get_transformer_for_type <'a, 'b> (&'a mut self, ty: &'b Type)
	-> Option <(&'b Type, Type, &'a mut ValueTransformer)>
	{
		self
			. associated_type_transformers
			. get_transformation (ty, &self . delegated_type, &self . forwarded_trait)
			. or (self . independent_type_transformers . get_transformation (ty))
	}

	fn get_transformer_for_ref_type <'a, 'b> (&'a mut self, ty: &'b Type)
	-> Option <(&'b Type, Type, &'a mut ValueTransformer)>
	{
		if let Type::Reference (ty_ref) = ty
		{
			if ty_ref . mutability . is_none ()
			{
				return self . get_transformer_for_type (&ty_ref . elem);
			}
		}

		None
	}

	fn get_transformer_for_ref_mut_type <'a, 'b> (&'a mut self, ty: &'b Type)
	-> Option <(&'b Type, Type, &'a mut ValueTransformer)>
	{
		if let Type::Reference (ty_ref) = ty
		{
			if ty_ref . mutability . is_some ()
			{
				return self . get_transformer_for_type (&ty_ref . elem);
			}
		}

		None
	}

	fn transform_input_box (&mut self, input: Expr, inner_type: &Type)
	-> Result <(Expr, bool)>
	{
		let inner_input = parse_quote! ((*#input));

		if let (inner_input, true) =
			self . transform_input (inner_input, inner_type)?
		{
			let input = parse_quote!
			(
				Box::new (#inner_input)
			);

			return Ok ((input, true));
		}

		Ok ((input, false))
	}

	fn transform_input_option (&mut self, input: Expr, inner_type: &Type)
	-> Result <(Expr, bool)>
	{
		let inner_input = parse_quote! (v);

		if let (inner_input, true) =
			self . transform_input (inner_input, inner_type)?
		{
			let input = parse_quote!
			(
				#input . map (|v| #inner_input)
			);

			return Ok ((input, true));
		}

		Ok ((input, false))
	}

	fn transform_input_result (&mut self, input: Expr, inner_type: &Type)
	-> Result <(Expr, bool)>
	{
		let inner_input = parse_quote! (v);

		if let (inner_input, true) =
			self . transform_input (inner_input, inner_type)?
		{
			let input = parse_quote!
			(
				#input . map (|v| #inner_input)
			);

			return Ok ((input, true));
		}

		Ok ((input, false))
	}

	fn transform_input_tuple
	(
		&mut self,
		input: Expr,
		inner_types: &Punctuated <Type, Token! [,]>
	)
	-> Result <(Expr, bool)>
	{
		let mut any_transformed: bool = false;

		let expr_var: Ident = parse_quote! (v);
		let mut inner_inputs = Punctuated::<Expr, Token! [,]>::new ();

		for i in 0..(inner_types . len ())
		{
			let idx = Index::from (i);
			let inner_input = parse_quote! (#expr_var . #idx);
			let inner_type = &inner_types [i];
			let (inner_input, input_transformed) =
				self . transform_input (inner_input, inner_type)?;

			inner_inputs . push (inner_input);
			any_transformed |= input_transformed;
		}

		if any_transformed
		{
			let input = parse_quote!
			(
				{
					let #expr_var = #input;

					(#inner_inputs)
				}
			);

			Ok ((input, true))
		}
		else
		{
			Ok ((input, false))
		}
	}

	fn transform_input_array (&mut self, input: Expr, inner_type: &Type)
	-> Result <(Expr, bool)>
	{
		let inner_input = parse_quote! (v);

		if let (inner_input, true) =
			self . transform_input (inner_input, inner_type)?
		{
			let input = parse_quote!
			(
				#input . map (|v| #inner_input)
			);

			return Ok ((input, true))
		}

		Ok ((input, false))
	}

	fn transform_input
	(
		&mut self,
		input: Expr,
		input_type: &Type
	)
	-> Result <(Expr, bool)>
	{
		if let Some ((from_type, to_type, value_transformer)) =
			self . get_transformer_for_type (input_type)
		{
			return Ok
			((
				value_transformer . transform_input
				(
					input,
					from_type,
					&to_type
				)?,
				true
			));
		}
		else if let Some ((from_type, to_type, value_transformer)) =
			self . get_transformer_for_ref_type (input_type)
		{
			return Ok
			((
				value_transformer . transform_input_ref
				(
					input,
					from_type,
					&to_type
				)?,
				true
			));
		}
		else if let Some ((from_type, to_type, value_transformer)) =
			self . get_transformer_for_ref_mut_type (input_type)
		{
			return Ok
			((
				value_transformer . transform_input_ref_mut
				(
					input,
					from_type,
					&to_type
				)?,
				true
			));
		}
		else if let Some (BoxType {boxed_type, ..}) =
			BoxType::match_type (input_type)
		{
			return self . transform_input_box (input, &boxed_type);
		}
		else if let Some (OptionType {option_type, ..}) =
			OptionType::match_type (input_type)
		{
			return self . transform_input_option (input, &option_type);
		}
		else if let Some (ResultType {result_type, ..}) =
			ResultType::match_type (input_type)
		{
			return self . transform_input_result (input, &result_type);
		}
		else if let Some (TupleType {types, ..}) =
			TupleType::match_type (input_type)
		{
			return self . transform_input_tuple (input, &types);
		}
		else if let Some (ArrayType {ty, ..}) =
			ArrayType::match_type (input_type)
		{
			return self . transform_input_array (input, &ty);
		}

		Ok ((input, false))
	}

	fn transform_output_box (&mut self, output: Expr, inner_type: &Type)
	-> Result <(Expr, bool)>
	{
		let inner_output = parse_quote! ((*#output));

		if let (inner_output, true) =
			self . transform_output (inner_output, inner_type)?
		{
			let output = parse_quote!
			(
				Box::new (#inner_output)
			);

			return Ok ((output, true));
		}

		Ok ((output, false))
	}

	fn transform_output_option (&mut self, output: Expr, inner_type: &Type)
	-> Result <(Expr, bool)>
	{
		let inner_output = parse_quote! (v);

		if let (inner_output, true) =
			self . transform_output (inner_output, inner_type)?
		{
			let output = parse_quote!
			(
				#output . map (|v| #inner_output)
			);

			return Ok ((output, true));
		}

		Ok ((output, false))
	}

	fn transform_output_result
    (
        &mut self,
        mut output: Expr,
        inner_type: &Type,
        error_type: &Type
    )
	-> Result <(Expr, bool)>
	{
		let mut any_transformed: bool = false;

		let inner_output = parse_quote! (v);
		let error_output = parse_quote! (v);

		if let (inner_output, true) =
			self . transform_output (inner_output, inner_type)?
		{
			output = parse_quote!
			(
				#output . map (|v| #inner_output)
			);

			any_transformed = true;
		}

		if let (error_output, true) =
			self . transform_output (error_output, error_type)?
		{
			output = parse_quote!
			(
				#output . map_err (|v| #error_output)
			);

			any_transformed = true;
		}

        Ok ((output, any_transformed))
	}

	fn transform_output_tuple
	(
		&mut self,
		output: Expr,
		inner_types: &Punctuated <Type, Token! [,]>
	)
	-> Result <(Expr, bool)>
	{
		let mut any_transformed: bool = false;

		let expr_var: Ident = parse_quote! (v);
		let mut inner_outputs = Punctuated::<Expr, Token! [,]>::new ();

		for i in 0..(inner_types . len ())
		{
			let idx = Index::from (i);
			let inner_output = parse_quote! (#expr_var . #idx);
			let inner_type = &inner_types [i];
			let (inner_output, output_transformed) =
				self . transform_output (inner_output, inner_type)?;

			inner_outputs . push (inner_output);
			any_transformed |= output_transformed;
		}

		if any_transformed
		{
			let output = parse_quote!
			(
				{
					let #expr_var = #output;

					(#inner_outputs)
				}
			);

			Ok ((output, true))
		}
		else
		{
			Ok ((output, false))
		}
	}

	fn transform_output_array (&mut self, output: Expr, inner_type: &Type)
	-> Result <(Expr, bool)>
	{
		let inner_output = parse_quote! (v);

		if let (inner_output, true) =
			self . transform_output (inner_output, inner_type)?
		{
			let output = parse_quote!
			(
				#output . map (|v| #inner_output)
			);

			return Ok ((output, true))
		}

		Ok ((output, false))
	}

	fn transform_output (&mut self, output: Expr, output_type: &Type)
	-> Result <(Expr, bool)>
	{
		if let Some ((from_type, to_type, value_transformer)) =
			self . get_transformer_for_type (output_type)
		{
			return Ok
			((
				value_transformer . transform_output
				(
					output,
					from_type,
					&to_type
				)?,
				true
			));
		}
		else if let Some (_) = self . get_transformer_for_ref_type (output_type)
		{
			return Err
			(
				Error::new_spanned
				(
					output_type,
					"Borrowed return values cannot be transformed for forwarding"
				)
			);
		}
		else if let Some (_) =
			self . get_transformer_for_ref_mut_type (output_type)
		{
			return Err
			(
				Error::new_spanned
				(
					output_type,
					"Borrowed return values cannot be transformed for forwarding"
				)
			);
		}
		else if let Some (BoxType {boxed_type, ..}) =
			BoxType::match_type (output_type)
		{
			return self . transform_output_box (output, &boxed_type);
		}
		else if let Some (OptionType {option_type, ..}) =
			OptionType::match_type (output_type)
		{
			return self . transform_output_option (output, &option_type);
		}
		else if let Some (ResultType {result_type, error_type, ..}) =
			ResultType::match_type (output_type)
		{
			return self . transform_output_result (output, &result_type, &error_type);
		}
		else if let Some (TupleType {types, ..}) =
			TupleType::match_type (output_type)
		{
			return self . transform_output_tuple (output, &types);
		}
		else if let Some (ArrayType {ty, ..}) =
			ArrayType::match_type (output_type)
		{
			return self . transform_output_array (output, &ty);
		}

		Ok ((output, false))
	}

	fn construct_arg (&mut self, input: &FnArg) -> Result <Expr>
	{
		match input
		{
			FnArg::Receiver (receiver) =>
			{
				let Receiver {self_token, ty, ..} = receiver;

				let arg = self . transform_input
				(
					parse_quote! (#self_token),
					ty . as_ref ()
				)?
					. 0;

				Ok (arg)
			},
			FnArg::Typed (pat_type) =>
			{
				let PatType {pat, ty, ..} = pat_type;

				let arg = self . transform_input
				(
					parse2 (pat . to_token_stream ())?,
					ty . as_ref ()
				)?
					. 0;

				Ok (arg)
			}
		}
	}

	fn transform_item_type (&self, item_type: TraitItemType)
	-> Result <ImplItemType>
	{
		let TraitItemType {ident, generics, ..} = item_type;

		let (impl_generics, _, where_clause) =
			generics . split_for_impl ();

		let assigned_type = self
			. associated_type_transformers
			. get_assigned_type
			(
				&ident,
				&generics,
				&self . delegated_type,
				&self . forwarded_trait
			)?;

		let item_type = parse_quote!
		{
			type #ident #impl_generics = #assigned_type
			#where_clause;
		};

		Ok (item_type)
	}

	fn transform_item_fn (&mut self, item_fn: TraitItemFn) -> Result <ImplItemFn>
	{
		let TraitItemFn
		{
			sig: Signature
			{
				constness,
				asyncness,
				unsafety,
				ident,
				generics,
				inputs,
				output,
				..
			},
            attrs,
			..
		}
			= item_fn;

		let mut args = Punctuated::<Expr, Token! [,]>::new ();
		for input in &inputs
		{
			args . push (self . construct_arg (input)?);
		}

		let call_expr =
		{
			let Self {delegated_type, forwarded_trait, ..} = &*self;
            let await_expr = asyncness.is_some().then(|| quote! { .await });

			parse_quote!
			(
				<#delegated_type as #forwarded_trait>::#ident (#args) #await_expr
			)
		};

		let body_expr = if let ReturnType::Type (_, boxed_ty) = &output
		{
			self . transform_output (call_expr, boxed_ty . as_ref ())? . 0
		}
		else
		{
			call_expr
		};

		let (impl_generics, _, where_clause) = generics . split_for_impl ();

		let item_fn = parse_quote!
		{
            #(#attrs)*
			#constness #asyncness #unsafety fn #ident #impl_generics (#inputs)
			#output
			#where_clause
			{
				#body_expr
			}
		};

		Ok (item_fn)
	}

	fn transform_item_const (&self, item_const: TraitItemConst)
	-> ImplItemConst
	{
		let Self {delegated_type, forwarded_trait, ..} = self;

		let TraitItemConst {ident, generics, ty, ..} = item_const;

		let (impl_generics, type_generics, where_clause) =
			generics . split_for_impl ();

		let item_const = parse_quote!
		{
			const #ident #impl_generics: #ty =
				<#delegated_type as #forwarded_trait>::#ident #type_generics
			#where_clause;
		};

		item_const
	}

	pub fn transform_trait_item (&mut self, item: TraitItem)
	-> Result <ImplItem>
	{
		match item
		{
			TraitItem::Const (item_const) => Ok
			(
				ImplItem::Const (self . transform_item_const (item_const))
			),
			TraitItem::Fn (item_fn) => Ok
			(
				ImplItem::Fn (self . transform_item_fn (item_fn)?)
			),
			TraitItem::Type (item_type) => Ok
			(
				ImplItem::Type (self . transform_item_type (item_type)?)
			),
			_ => Err
			(
				Error::new_spanned
				(
					item,
					"Forwarding trait items of this type is not supported"
				)
			)
		}
	}

	pub fn add_predicates
	(
		&self,
		predicates: &mut Punctuated <WherePredicate, Token! [,]>
	)
	{
		self . associated_type_transformers . add_predicates
		(
			predicates,
			&self . delegated_type,
			&self . forwarded_trait
		);

		self . independent_type_transformers . add_predicates (predicates);
	}
}
