use std::collections::HashMap;

use syn
::{
	Ident,
	Type,
	Path,
	Generics,
	WherePredicate,
	BoundLifetimes,
	Token,
	parse_quote
};
use syn::punctuated::Punctuated;
use syn::parse::Result;
use syn::fold::Fold;

use crate::syn::associated_type::AssociatedType;

use crate::fold::evaluator::get_associated_type_evaluator;

use crate::value_transformer::value_transformer::ValueTransformer;

pub struct AssociatedTypeTransformer
{
	pub lifetimes: Option <BoundLifetimes>,
	pub associated_type: AssociatedType,
	pub replacement_type: Type,
	pub value_transformer: ValueTransformer
}

impl AssociatedTypeTransformer
{
	pub fn get_replacement_type (&self, trait_def_generics: &Generics)
	-> Result <Type>
	{
		let mut associated_type_evaluator = get_associated_type_evaluator
		(
			&self . associated_type . generics,
			trait_def_generics
		)?;

		let replacement_type = associated_type_evaluator
			. fold_type (self . replacement_type . clone ());

		Ok (replacement_type)
	}
}

pub struct AssociatedTypeTransformers
{
	map: HashMap <Ident, AssociatedTypeTransformer>
}

impl AssociatedTypeTransformers
{
	pub fn new () -> Self
	{
		Self {map: HashMap::new ()}
	}

	pub fn insert
	(
		&mut self,
		associated_type_transformer: AssociatedTypeTransformer
	)
	{
		self . map . insert
		(
			associated_type_transformer . associated_type . ident . clone (),
			associated_type_transformer
		);
	}

	pub fn get_assigned_type
	(
		&self,
		associated_type_ident: &Ident,
		trait_def_generics: &Generics,
		delegated_type: &Type,
		forwarded_trait: &Path
	)
	-> Result <Type>
	{
		let ty = match self
			. map
			. get (associated_type_ident)
		{
			None => parse_quote!
			(
				<#delegated_type as #forwarded_trait>::#associated_type_ident #trait_def_generics
			),
			Some (associated_type_transformer) =>
				associated_type_transformer
					. get_replacement_type (trait_def_generics)?
		};

		Ok (ty)
	}

	pub fn get_transformation <'a, 'b>
	(
		&'a mut self,
		ty: &'b Type,
		delegated_type: &Type,
		forwarded_trait: &Path
	)
	-> Option <(&'b Type, Type, &'a mut ValueTransformer)>
	{
		if let Some (AssociatedType {ident, generics, ..}) =
			AssociatedType::match_type (ty)
		{
			self . map . get_mut (&ident) . map
			(
				|associated_type_transformer|
				(
					ty,
					parse_quote! (<#delegated_type as #forwarded_trait>::#ident #generics),
					&mut associated_type_transformer . value_transformer
				)
			)
		}
		else { None }
	}

	pub fn add_predicates
	(
		&self,
		predicates: &mut Punctuated <WherePredicate, Token! [,]>,
		delegated_type: &Type,
		forwarded_trait: &Path
	)
	{
		for associated_type_transformer in self . map . values ()
		{
			let AssociatedType {ident, generics, ..} =
				&associated_type_transformer . associated_type;

			associated_type_transformer . value_transformer . add_predicates
			(
				predicates,
				&associated_type_transformer . lifetimes,
				&associated_type_transformer . replacement_type,
				&parse_quote! (<#delegated_type as #forwarded_trait>::#ident #generics)
			);
		}
	}
}
