use forward_traits
::{
	supply_forwarding_info_for_trait,
	forward_receiver,
	forward_traits
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

forward_traits! (for VecWrapper . 0 impl IntoIterator);

fn main ()
{
	VecWrapper (vec! (1, 2, 3)) . into_iter ();
}
