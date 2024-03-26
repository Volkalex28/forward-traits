use syn::{Token, bracketed};
use syn::token::Bracket;
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result};
use quote::ToTokens;

use crate::transform_info::TransformInfo;

pub struct AdditionalTransformInfos
{
	bracket_token: Option <Bracket>,
	transform_infos: Punctuated <TransformInfo, Token! [,]>,
}

impl Default for AdditionalTransformInfos
{
	fn default () -> Self
	{
		Self { bracket_token: None, transform_infos: Punctuated::new () }
	}
}

impl Parse for AdditionalTransformInfos
{
	fn parse (input: ParseStream) -> Result <Self>
	{
		if input . peek (Bracket)
		{
			let content;
			let bracket_token = Some (bracketed! (content in input));
			let transform_infos = Punctuated::parse_terminated (&content)?;

			Ok (Self {bracket_token, transform_infos})
		}
		else
		{
			Ok (Self::default ())
		}
	}
}

impl ToTokens for AdditionalTransformInfos
{
	fn to_tokens (&self, tokens: &mut proc_macro2::TokenStream)
	{
		if self . transform_infos . is_empty () { return; }

		self
			. bracket_token
			. unwrap_or_default ()
			. surround
			(
				tokens,
				|tokens| self . transform_infos . to_tokens (tokens)
			);
	}
}

impl IntoIterator for AdditionalTransformInfos
{
	type Item = TransformInfo;
	type IntoIter = <Punctuated <TransformInfo, Token! [,]> as IntoIterator>::IntoIter;

	fn into_iter (self) -> Self::IntoIter
	{
		self . transform_infos . into_iter ()
	}
}

impl <'a> IntoIterator for &'a AdditionalTransformInfos
{
	type Item = &'a TransformInfo;
	type IntoIter = <&'a Punctuated <TransformInfo, Token! [,]> as IntoIterator>::IntoIter;

	fn into_iter (self) -> Self::IntoIter
	{
		(&self . transform_infos) . into_iter ()
	}
}

impl <'a> IntoIterator for &'a mut AdditionalTransformInfos
{
	type Item = &'a mut TransformInfo;
	type IntoIter = <&'a mut Punctuated <TransformInfo, Token! [,]> as IntoIterator>::IntoIter;

	fn into_iter (self) -> Self::IntoIter
	{
		(&mut self . transform_infos) . into_iter ()
	}
}
