use forward_traits::{forwardable, forward_receiver, forward_traits};

#[forwardable]
trait Foo <T>
where T: ?Sized
{
}

struct A ();

impl <T> Foo <T> for A
{
}

#[forward_receiver]
struct B (A);

forward_traits! (for B . 0 impl for <T> Foo <T>);

fn main ()
{
}
