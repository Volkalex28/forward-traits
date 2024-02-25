use forward_traits::{forwardable, forward_receiver, forward_traits_via_member};

#[forwardable]
trait FooTrait
{
	type Bar;

	fn foo (&self) -> &Self::Bar;

	const BAZ: u32;
}

struct A {}

impl FooTrait for A
{
	type Bar = Self;

	fn foo (&self) -> &Self::Bar { self }

	const BAZ: u32 = 42;
}

#[forward_receiver]
struct B (A);

forward_traits_via_member! (B . 0, FooTrait);

fn main ()
{
	assert_eq! (<B as FooTrait>::BAZ, 42);
}
