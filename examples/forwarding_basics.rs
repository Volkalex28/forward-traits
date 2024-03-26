use forward_traits::{forwardable, forward_receiver, forward_traits};

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

forward_traits! (for B . 0 impl FooTrait);

fn main ()
{
	assert_eq! (<B as FooTrait>::BAZ, 42);
}
