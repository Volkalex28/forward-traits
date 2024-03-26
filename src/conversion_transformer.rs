use syn::{Type, Expr, WherePredicate, Token, parse_quote};
use syn::punctuated::Punctuated;
use syn::Result;

pub struct ConversionTransformer
{
	use_into: bool,
	use_borrow: bool,
	use_borrow_mut: bool,
	use_from: bool,
}

impl ConversionTransformer
{
	pub fn new () -> Self
	{
		Self
		{
			use_into: false,
			use_borrow: false,
			use_borrow_mut: false,
			use_from: false,
		}
	}
}

impl ConversionTransformer
{
	pub fn transform_input
	(
		&mut self,
		input: Expr,
		from_type: &Type,
		to_type: &Type
	)
	-> Result <Expr>
	{
		self . use_into = true;

		let input = parse_quote!
		(
			<#from_type as std::convert::Into <#to_type>>::into (#input)
		);

		Ok (input)
	}

	pub fn transform_input_ref
	(
		&mut self,
		input: Expr,
		from_type: &Type,
		to_type: &Type
	)
	-> Result <Expr>
	{
		self . use_borrow = true;

		let input = parse_quote!
		(
			<#from_type as std::convert::AsRef <#to_type>>::as_ref (#input)
		);

		Ok (input)
	}

	pub fn transform_input_ref_mut
	(
		&mut self,
		input: Expr,
		from_type: &Type,
		to_type: &Type
	)
	-> Result <Expr>
	{
		self . use_borrow_mut = true;

		let input = parse_quote!
		(
			<#from_type as std::convert::AsMut <#to_type>>::as_mut (#input)
		);

		Ok (input)
	}

	pub fn transform_output
	(
		&mut self,
		output: Expr,
		from_type: &Type,
		to_type: &Type
	)
	-> Result <Expr>
	{
		self . use_from = true;

		let output = parse_quote!
		(
			<#from_type as std::convert::From <#to_type>>::from (#output)
		);

		Ok (output)
	}

	pub fn add_predicates
	(
		&self,
		predicates: &mut Punctuated <WherePredicate, Token! [,]>,
		from_type: &Type,
		to_type: &Type
	)
	{
		if self . use_into
		{
			predicates . push
			(
				parse_quote! (#from_type: std::convert::Into <#to_type>)
			);
		}

		if self . use_borrow
		{
			predicates . push
			(
				parse_quote! (#from_type: std::convert::AsRef <#to_type>)
			);
		}

		if self . use_borrow_mut
		{
			predicates . push
			(
				parse_quote! (#from_type: std::convert::AsMut <#to_type>)
			);
		}

		if self . use_from
		{
			predicates . push
			(
				parse_quote! (#from_type: std::convert::From <#to_type>)
			);
		}
	}
}
