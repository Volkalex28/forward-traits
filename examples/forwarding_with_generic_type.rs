use forward_traits
::{
	supply_forwarding_info_for_trait,
	forward_receiver,
	forward_traits_via_member
};

supply_forwarding_info_for_trait!
(
	std::iter::IntoIterator,
	trait
	{
		type Item;
		type IntoIter;

		fn into_iter (self) -> Self::IntoIter;
	}
);

#[forward_receiver]
struct VecWrapper <T> (Vec <T>);

forward_traits_via_member! (VecWrapper . 0, IntoIterator);

fn main ()
{
	VecWrapper (vec! (1, 2, 3)) . into_iter ();
}
