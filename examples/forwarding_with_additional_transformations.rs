use forward_traits::{forwardable, forward_receiver, forward_traits};

struct Algebra {}

#[forwardable]
trait Foo <T>
{
	fn foo (self, x: T);
}

impl <T> Foo <T> for Algebra
{
	fn foo (self, _x: T) {}
}

struct Wrap <T> (T);

#[forward_receiver]
struct WrapAlgebra {}

impl Into <Algebra> for WrapAlgebra
{
	fn into (self) -> Algebra
	{
		Algebra {}
	}
}

forward_traits!
(
	for <T> WrapAlgebra -> Algebra [Wrap <T> . 0: T]
	where Wrap <T>: Sized
	impl Foo <T>
);

fn main ()
{
	WrapAlgebra {} . foo (Wrap::<f32> (1.0))
}
