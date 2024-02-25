use syn
::{
	Ident,
	Type,
	Expr,
	Path,
	PathArguments,
	GenericArgument,
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
	Token,
	parse_quote,
	parse2
};
use syn::punctuated::Punctuated;
use syn::parse::{Result, Error};
use quote::ToTokens;

fn is_self (ty: &Type) -> bool
{
	*ty == parse_quote! (Self)
}

fn is_ref_self (ty: &Type) -> bool
{
	if let Type::Reference (ty_ref) = ty
	{
		ty_ref . mutability . is_none () && is_self (&ty_ref . elem)
	}
	else { false }
}

fn is_ref_mut_self (ty: &Type) -> bool
{
	if let Type::Reference (ty_ref) = ty
	{
		ty_ref . mutability . is_some () && is_self (&ty_ref . elem)
	}
	else { false }
}

fn get_leaf (ty: &Type)
-> Option <(&Ident, Option <&Punctuated <GenericArgument, Token! [,]>>)>
{
	if let Type::Path (ty_path) = ty
	{
		if let Some (last_segment) = ty_path . path . segments . last ()
		{
			let ident = &last_segment . ident;
			let args = if let PathArguments::AngleBracketed (angle_args) =
				&last_segment . arguments
			{
				Some (&angle_args . args)
			}
			else { None };

			Some ((ident, args))
		}
		else
		{
			None
		}
	}
	else { None }
}

macro_rules! define_is_container
{
	($fn_name: ident, $type_str: expr) =>
	{
		fn $fn_name (ty: &Type) -> Option <&Type>
		{
			if let Some ((ident, Some (args))) = get_leaf (ty)
			{
				if ident == $type_str && args . len () == 2
				{
					if let Some (GenericArgument::Type (ty)) = args . first ()
					{
						return Some (ty);
					}
				}
			}

			None
		}
	}
}

define_is_container! (is_result, "Result");
define_is_container! (is_box, "Box");
define_is_container! (is_pin, "Pin");
define_is_container! (is_rc, "Rc");
define_is_container! (is_arc, "Arc");

// I could make a lot of these functions immutable and private by making them
// bare fns that take a trait item as the first argument.  Then the trait could
// simply provide an implementation of the top-level user-facing function.
// Dunno if that's what I actually want to do, though.

pub trait Transformer
{
	fn transform_input_self
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		input_type: &Type
	)
	-> Result <Expr>;

	fn transform_input_ref_self
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		input_type: &Type
	)
	-> Result <Expr>;

	fn transform_input_ref_mut_self
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		input_type: &Type
	)
	-> Result <Expr>;

	fn transform_input_result
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		inner_type: &Type
	)
	-> Result <(Expr, bool)>
	{
		let inner_input = parse_quote! (v);

		if let (inner_input, true) = self . transform_input
		(
			delegated_type,
			inner_input,
			inner_type
		)?
		{
			let input = parse_quote!
			(
				#input . map (|v| #inner_input)
			);

			return Ok ((input, true));
		}

		Ok ((input, false))
	}

	fn transform_input_box
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		inner_type: &Type
	)
	-> Result <(Expr, bool)>
	{
		let inner_input = parse_quote! (*#input);

		if let (inner_input, true) = self . transform_input
		(
			delegated_type,
			inner_input,
			inner_type
		)?
		{
			let input = parse_quote!
			(
				Box::new (#inner_input)
			);

			return Ok ((input, true));
		}

		Ok ((input, false))
	}

	fn transform_input
	(
		&mut self,
		delegated_type: &Type,
		input: Expr,
		input_type: &Type,
	)
	-> Result <(Expr, bool)>
	{
		if is_self (input_type)
		{
			return Ok
			((
				self . transform_input_self
				(
					delegated_type,
					input,
					input_type
				)?,
				true
			));
		}
		else if is_ref_self (input_type)
		{
			return Ok
			((
				self . transform_input_ref_self
				(
					delegated_type,
					input,
					input_type
				)?,
				true
			));
		}
		else if is_ref_mut_self (input_type)
		{
			return Ok
			((
				self . transform_input_ref_mut_self
				(
					delegated_type,
					input,
					input_type
				)?,
				true
			));
		}
		else if let Some (inner_type) = is_result (input_type)
		{
			return self . transform_input_result
			(
				delegated_type,
				input,
				inner_type
			);
		}
		else if let Some (inner_type) = is_box (input_type)
		{
			return self . transform_input_box
			(
				delegated_type,
				input,
				inner_type
			);
		}
		else if let Some (inner_type) = is_pin (input_type)
		{
			if let (_, true) = self . transform_input
			(
				delegated_type,
				input . clone (),
				inner_type
			)?
			{
				return Err
				(
					Error::new_spanned
					(
						input_type,
						"Forwarding methods that take pinned forms of self is unsupported"
					)
				)
			}
		}
		else if let Some (inner_type) = is_rc (input_type)
		{
			if let (_, true) = self . transform_input
			(
				delegated_type,
				input . clone (),
				inner_type
			)?
			{
				return Err
				(
					Error::new_spanned
					(
						input_type,
						"Forwarding methods that take reference counting pointers to self is unsupported"
					)
				);
			}
		}
		else if let Some (inner_type) = is_arc (input_type)
		{
			if let (_, true) = self . transform_input
			(
				delegated_type,
				input . clone (),
				inner_type
			)?
			{
				return Err
				(
					Error::new_spanned
					(
						input_type,
						"Forwarding methods that take reference counting pointers to self is unsupported"
					)
				);
			}
		}

		Ok ((input, false))
	}

	fn transform_output_self
	(
		&mut self,
		delegated_type: &Type,
		output: Expr,
		output_type: &Type
	)
	-> Result <Expr>;

	fn transform_output_result
	(
		&mut self,
		delegated_type: &Type,
		output: Expr,
		inner_type: &Type
	)
	-> Result <(Expr, bool)>
	{
		let inner_output = parse_quote! (v);

		if let (inner_output, true) = self . transform_output
		(
			delegated_type,
			inner_output,
			inner_type
		)?
		{
			let output = parse_quote!
			(
				#output . map (|v| #inner_output)
			);

			return Ok ((output, true));
		}

		Ok ((output, false))
	}

	fn transform_output_box
	(
		&mut self,
		delegated_type: &Type,
		output: Expr,
		inner_type: &Type
	)
	-> Result <(Expr, bool)>
	{
		let inner_output = parse_quote! (*#output);

		if let (inner_output, true) = self . transform_output
		(
			delegated_type,
			inner_output,
			inner_type
		)?
		{
			let output = parse_quote!
			(
				Box::new (#inner_output)
			);

			return Ok ((output, true));
		}

		Ok ((output, false))
	}

	fn transform_output
	(
		&mut self,
		delegated_type: &Type,
		output: Expr,
		output_type: &Type
	)
	-> Result <(Expr, bool)>
	{
		if is_self (output_type)
		{
			return Ok
			((
				self . transform_output_self
				(
					delegated_type,
					output,
					output_type
				)?,
				true
			));
		}
		else if is_ref_self (output_type)
		{
			return Err
			(
				Error::new_spanned
				(
					output_type,
					"Methods that return `&Self` cannot be forwarded"
				)
			);
		}
		else if is_ref_mut_self (output_type)
		{
			return Err
			(
				Error::new_spanned
				(
					output_type,
					"Methods that return `&mut Self` cannot be forwarded"
				)
			);
		}
		else if let Some (inner_type) = is_result (output_type)
		{
			return self . transform_output_result
			(
				delegated_type,
				output,
				inner_type
			);
		}
		else if let Some (inner_type) = is_box (output_type)
		{
			return self . transform_output_box
			(
				delegated_type,
				output,
				inner_type
			);
		}
		else if let Some (inner_type) = is_pin (output_type)
		{
			if let (_output, true) = self . transform_output
			(
				delegated_type,
				output . clone (),
				inner_type
			)?
			{
				return Err
				(
					Error::new_spanned
					(
						output_type,
						"Methods that return any pinned form of self cannot be forwarded"
					)
				);
			}
		}
		else if let Some (inner_type) = is_rc (output_type)
		{
			if let (_output, true) = self . transform_output
			(
				delegated_type,
				output . clone (),
				inner_type
			)?
			{
				return Err
				(
					Error::new_spanned
					(
						output_type,
						"Methods that return any reference counted pointers to self cannot be forwarded"
					)
				);
			}
		}
		else if let Some (inner_type) = is_arc (output_type)
		{
			if let (_output, true) = self . transform_output
			(
				delegated_type,
				output . clone (),
				inner_type
			)?
			{
				return Err
				(
					Error::new_spanned
					(
						output_type,
						"Methods that return any reference counted pointers to self cannot be forwarded"
					)
				);
			}
		}

		Ok ((output, false))
	}

	fn construct_arg (&mut self, delegated_type: &Type, input: &FnArg)
	-> Result <Expr>
	{
		match input
		{
			FnArg::Receiver (receiver) =>
			{
				let Receiver {self_token, ty, ..} = receiver;

				let arg = self . transform_input
				(
					delegated_type,
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
					delegated_type,
					parse2 (pat . to_token_stream ())?,
					ty . as_ref ()
				)?
					. 0;

				Ok (arg)
			}
		}
	}

	fn transform_item_type
	(
		&mut self,
		delegated_type: &Type,
		forwarded_trait: &Path,
		item_type: TraitItemType
	)
	-> Result <ImplItemType>
	{
		let TraitItemType {ident, generics, ..} = item_type;

		let (impl_generics, type_generics, where_clause) =
			generics . split_for_impl ();

		let item_type = parse_quote!
		{
			type #ident #impl_generics =
				<#delegated_type as #forwarded_trait>::#ident #type_generics
			#where_clause;
		};

		Ok (item_type)
	}

	fn transform_item_fn
	(
		&mut self,
		delegated_type: &Type,
		forwarded_trait: &Path,
		item_fn: TraitItemFn
	)
	-> Result <ImplItemFn>
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
			..
		}
			= item_fn;

		let mut args = Punctuated::<Expr, Token! [,]>::new ();
		for input in &inputs
		{
			args . push (self . construct_arg (delegated_type, input)?);
		}

		let call_expr = parse_quote!
		(
			<#delegated_type as #forwarded_trait>::#ident (#args)
		);

		let body_expr = if let ReturnType::Type (_, boxed_ty) = &output
		{
			self . transform_output (delegated_type, call_expr, boxed_ty . as_ref ())? . 0
		}
		else
		{
			call_expr
		};

		let (impl_generics, _, where_clause) = generics . split_for_impl ();

		let item_fn = parse_quote!
		{
			#constness #asyncness #unsafety fn #ident #impl_generics (#inputs)
			#output
			#where_clause
			{
				#body_expr
			}
		};

		Ok (item_fn)
	}

	fn transform_item_const
	(
		&mut self,
		delegated_type: &Type,
		forwarded_trait: &Path,
		item_const: TraitItemConst
	)
	-> ImplItemConst
	{
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

	fn transform_item
	(
		&mut self,
		delegated_type: &Type,
		forwarded_trait: &Path,
		item: TraitItem
	)
	-> Result <ImplItem>
	{
		match item
		{
			TraitItem::Const (item_const) => Ok
			(
				ImplItem::Const
				(
					self . transform_item_const
					(
						delegated_type,
						forwarded_trait,
						item_const
					)
				)
			),
			TraitItem::Fn (item_fn) => Ok
			(
				ImplItem::Fn
				(
					self . transform_item_fn
					(
						delegated_type,
						forwarded_trait,
						item_fn
					)?
				)
			),
			TraitItem::Type (item_type) => Ok
			(
				ImplItem::Type
				(
					self . transform_item_type
					(
						delegated_type,
						forwarded_trait,
						item_type
					)?
				)
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
}
