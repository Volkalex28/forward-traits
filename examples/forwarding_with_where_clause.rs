use forward_traits
::{
	supply_forwarding_info_for_trait,
	forward_receiver,
	forward_traits
};

supply_forwarding_info_for_trait!
(
	std::clone::Clone,
	trait { fn clone (&self) -> Self; }
);

#[derive (Copy, Clone)]
struct A {}

#[forward_receiver]
struct B (A);

impl From <A> for B
{
	fn from (a: A) -> Self
	{
		B (a)
	}
}

impl AsRef <A> for B
{
	fn as_ref (&self) -> &A
	{
		&self . 0
	}
}

forward_traits! (for B -> A impl Clone where A: Copy;);

fn main ()
{
	let _ = B (A {}) . clone ();
}
