use forward_traits::{forwardable, forward_receiver, forward_traits};

#[forwardable]
trait Foo <A, B = A>
{
	fn foo (&self, x: A, y: B) -> Option <B>;
}

struct Delegated
{
}

impl Foo <u32> for Delegated
{
	fn foo (&self, x: u32, y: u32) -> Option <u32>
	{
		Some (x + y)
	}
}

#[forward_receiver]
struct Receiver
{
	d: Delegated
}

forward_traits! (for Receiver . d impl Foo <u32>);

fn main ()
{
	assert_eq! (Receiver {d: Delegated {}} . foo (1, 2), Some (3));
}
