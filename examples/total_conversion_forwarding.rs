use std::convert::{AsRef, AsMut};

use forward_traits::{forwardable, forward_receiver, forward_traits_via_conversion};

#[forwardable]
trait Foo
{
	fn foo1 (self);
	fn foo2 (&self);
	fn foo3 (&mut self);
	fn foo4 () -> Self;
}

struct A {}

impl Foo for A
{
	fn foo1 (self) {}
	fn foo2 (&self) {}
	fn foo3 (&mut self) {}
	fn foo4 () -> Self { A {} }
}

#[forward_receiver]
struct B (A);

impl From <A> for B
{
	fn from (a: A) -> Self
	{
		B (a)
	}
}

impl From <B> for A
{
	fn from (b: B) -> Self
	{
		b . 0
	}
}

impl AsRef <A> for B
{
	fn as_ref (&self) -> &A
	{
		&self . 0
	}
}

impl AsMut <A> for B
{
	fn as_mut (&mut self) -> &mut A
	{
		&mut self . 0
	}
}

forward_traits_via_conversion! (B -> A, Foo);

fn main ()
{
	let mut b = B::foo4 ();

	b . foo3 ();
	b . foo2 ();
	b . foo1 ();
}
