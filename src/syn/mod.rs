pub mod member;
pub mod associated_type;
pub mod from_type;
pub mod transformable_types;

pub mod trait_def_info;
pub mod type_def_info;
pub mod forwarded_trait_info;

pub mod additional_type_transformers;

pub mod kw
{
	syn::custom_keyword! (Box);
	syn::custom_keyword! (Option);
	syn::custom_keyword! (Result);
}
