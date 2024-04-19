use std::collections::HashMap;

use syn::{Type, WherePredicate, BoundLifetimes, Token};
use syn::punctuated::Punctuated;

use crate::fold::type_transformer::TypeTransformer;

use crate::value_transformer::value_transformer::ValueTransformer;

pub struct IndependentTypeTransformer
{
	pub lifetimes: Option <BoundLifetimes>,
	pub from_type: Type,
	pub to_type: Type,
	pub value_transformer: ValueTransformer
}

pub struct IndependentTypeTransformers
{
	map: HashMap <Type, IndependentTypeTransformer>
}

impl IndependentTypeTransformers
{
	pub fn new () -> Self
	{
		Self {map: HashMap::new ()}
	}

	pub fn insert
	(
		&mut self,
		independent_type_transformer: IndependentTypeTransformer
	)
	{
		self . map . insert
		(
			independent_type_transformer . from_type . clone (),
			independent_type_transformer
		);
	}

	// Of the fold variety.
	pub fn get_type_transformer (&self)
	-> TypeTransformer <impl FnMut (&Type) -> Option <Type> + '_>
	{
		TypeTransformer::new
		(
			|ty| self
				. map
				. get (ty)
				. map
				(
					|independent_type_transformer|
					independent_type_transformer . to_type . clone ()
				)
		)
	}

	pub fn get_transformation <'a, 'b> (&'a mut self, ty: &'b Type)
	-> Option <(&'b Type, Type, &'a mut ValueTransformer)>
	{
		self
			. map
			. get_mut (ty)
			. map
			(
				|type_transformer|
				(
					ty,
					type_transformer . to_type . clone (),
					&mut type_transformer . value_transformer
				)
			)
	}

	pub fn add_predicates
	(
		&self,
		predicates: &mut Punctuated <WherePredicate, Token! [,]>
	)
	{
		for independent_type_transformer in self . map . values ()
		{
			independent_type_transformer . value_transformer . add_predicates
			(
				predicates,
				&independent_type_transformer . lifetimes,
				&independent_type_transformer . from_type,
				&independent_type_transformer . to_type
			);
		}
	}
}
