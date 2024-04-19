use syn::{Token, bracketed};
use syn::token::Bracket;
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result};
use quote::ToTokens;

use crate::type_transformer::additional_type_transformer::AdditionalTypeTransformer;

pub struct AdditionalTypeTransformers
{
	bracket_token: Option <Bracket>,
	type_transformers: Punctuated <AdditionalTypeTransformer, Token! [,]>
}

impl Default for AdditionalTypeTransformers
{
	fn default () -> Self
	{
		Self { bracket_token: None, type_transformers: Punctuated::new () }
	}
}

impl Parse for AdditionalTypeTransformers
{
	fn parse (input: ParseStream) -> Result <Self>
	{
		if input . peek (Bracket)
		{
			let content;
			let bracket_token = Some (bracketed! (content in input));
			let type_transformers = Punctuated::parse_terminated (&content)?;

			Ok (Self {bracket_token, type_transformers})
		}
		else
		{
			Ok (Self::default ())
		}
	}
}

impl ToTokens for AdditionalTypeTransformers
{
	fn to_tokens (&self, tokens: &mut proc_macro2::TokenStream)
	{
		if self . type_transformers . is_empty () { return; }

		self
			. bracket_token
			. unwrap_or_default ()
			. surround
			(
				tokens,
				|tokens| self . type_transformers . to_tokens (tokens)
			);
	}
}

impl IntoIterator for AdditionalTypeTransformers
{
	type Item = AdditionalTypeTransformer;
	type IntoIter = <Punctuated <AdditionalTypeTransformer, Token! [,]> as IntoIterator>::IntoIter;

	fn into_iter (self) -> Self::IntoIter
	{
		self . type_transformers . into_iter ()
	}
}

impl <'a> IntoIterator for &'a AdditionalTypeTransformers
{
	type Item = &'a AdditionalTypeTransformer;
	type IntoIter = <&'a Punctuated <AdditionalTypeTransformer, Token! [,]> as IntoIterator>::IntoIter;

	fn into_iter (self) -> Self::IntoIter
	{
		(&self . type_transformers) . into_iter ()
	}
}

impl <'a> IntoIterator for &'a mut AdditionalTypeTransformers
{
	type Item = &'a mut AdditionalTypeTransformer;
	type IntoIter = <&'a mut Punctuated <AdditionalTypeTransformer, Token! [,]> as IntoIterator>::IntoIter;

	fn into_iter (self) -> Self::IntoIter
	{
		(&mut self . type_transformers) . into_iter ()
	}
}
