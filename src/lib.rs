mod syntax;
mod info;
mod forward;
mod uncurry;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn forwardable (attr: TokenStream, item: TokenStream) -> TokenStream
{
	info::forwardable_impl (attr, item)
}

#[proc_macro_attribute]
pub fn forward_receiver (attr: TokenStream, item: TokenStream) -> TokenStream
{
	info::forward_receiver_impl (attr, item)
}

#[doc (hidden)]
#[proc_macro]
pub fn forward_trait_via_conversion_core (input: TokenStream) -> TokenStream
{
	forward::forward_trait_via_conversion_core_impl (input)
}

#[proc_macro]
pub fn forward_trait_via_conversion (input: TokenStream) -> TokenStream
{
	forward::forward_trait_via_conversion_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn forward_trait_via_member_core (input: TokenStream) -> TokenStream
{
	forward::forward_trait_via_member_core_impl (input)
}

#[proc_macro]
pub fn forward_trait_via_member (input: TokenStream) -> TokenStream
{
	forward::forward_trait_via_member_impl (input)
}
