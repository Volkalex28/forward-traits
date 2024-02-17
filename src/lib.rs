mod syntax;
mod info;
mod forward;
mod uncurry;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn trait_info (attr: TokenStream, item: TokenStream) -> TokenStream
{
	info::trait_info_impl (attr, item)
}

#[proc_macro_attribute]
pub fn type_info (attr: TokenStream, item: TokenStream) -> TokenStream
{
	info::type_info_impl (attr, item)
}

#[proc_macro]
pub fn forward_conversion_trait_core (input: TokenStream) -> TokenStream
{
	forward::forward_conversion_trait_core_impl (input)
}

#[proc_macro]
pub fn forward_conversion_trait (input: TokenStream) -> TokenStream
{
	forward::forward_conversion_trait_impl (input)
}

#[proc_macro]
pub fn forward_member_trait_core (input: TokenStream) -> TokenStream
{
	forward::forward_member_trait_core_impl (input)
}

#[proc_macro]
pub fn forward_member_trait (input: TokenStream) -> TokenStream
{
	forward::forward_member_trait_impl (input)
}
