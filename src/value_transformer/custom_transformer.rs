use syn::{parse_quote, Block, Expr, Ident};
use syn::parse::Result;

pub struct CustomTransformer
{
	ident: Ident,
    block: Block,
}

impl CustomTransformer
{
	pub fn new 
    (
        ident: Ident,
        block: Block
    )
    -> Self
	{
		Self { ident, block }
	}
}

impl CustomTransformer
{
    fn transform
	(
		&mut self,
		input: Expr
	)
	-> Result <Expr> {
        let ident = &self . ident;
		let block = &self . block;

        let input = parse_quote!
        (
            {
                let #ident = #input;

                #[allow(unused_braces)]
                #block
            } 
        );

		Ok (input)
    }

	pub fn transform_input
	(
		&mut self,
		input: Expr
	)
	-> Result <Expr>
	{
		self . transform (input)
	}

	pub fn transform_input_ref
	(
		&mut self,
		input: Expr
	)
	-> Result <Expr>
	{
		self . transform (input)
	}

	pub fn transform_input_ref_mut
	(
		&mut self,
		input: Expr
	)
	-> Result <Expr>
	{
		self . transform (input)
	}
}