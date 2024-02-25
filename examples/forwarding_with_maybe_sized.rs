use forward_traits::{forwardable, forward_receiver, forward_traits_via_member};

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

forward_traits_via_member! (B . 0, for <T> Foo <T>);

fn main ()
{
}
