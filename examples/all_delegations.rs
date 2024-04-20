use std::convert::{AsRef, AsMut};

use forward_traits::{forwardable, forward_receiver, forward_traits};

#[forwardable]
trait Foo: Sized
{
	fn foo1 (self);
	fn foo2 (&self);
	fn foo3 (&mut self);
	fn foo4 (self: Box <Self>);
	fn foo5 (x: Option <Self>);
	fn foo6 (x: Result <Self, ()>);
	fn foo7 (x: (Self, Self));
	fn foo8 (x: [Self; 2]);

	fn foo9 () -> Self;
	fn foo10 () -> Box <Self>;
	fn foo11 () -> Option <Self>;
	fn foo12 () -> Result <Self, ()>;
	fn foo13 () -> (Self, Self);
	fn foo14 () -> [Self; 2];
}

#[forwardable]
trait Bar: Sized
{
	fn bar1 (self);
	fn bar2 (&self);
	fn bar3 (&mut self);
	fn bar4 (self: Box <Self>);
	fn bar5 (x: Option <Self>);
	fn bar6 (x: Result <Self, ()>);
	fn bar7 (x: (Self, Self));
	fn bar8 (x: [Self; 2]);
}

struct A;

impl Foo for A
{
	fn foo1 (self) {}
	fn foo2 (&self) {}
	fn foo3 (&mut self) {}
	fn foo4 (self: Box <Self>) {}
	fn foo5 (_x: Option <Self>) {}
	fn foo6 (_x: Result <Self, ()>) {}
	fn foo7 (_x: (Self, Self)) {}
	fn foo8 (_x: [Self; 2]) {}

	fn foo9 () -> Self { A }
	fn foo10 () -> Box <Self> { Box::new (A) }
	fn foo11 () -> Option <Self> { Some (A) }
	fn foo12 () -> Result <Self, ()> { Ok (A) }
	fn foo13 () -> (Self, Self) { (A, A) }
	fn foo14 () -> [Self; 2] { [A, A] }
}

impl Bar for A
{
	fn bar1 (self) {}
	fn bar2 (&self) {}
	fn bar3 (&mut self) {}
	fn bar4 (self: Box <Self>) {}
	fn bar5 (_x: Option <Self>) {}
	fn bar6 (_x: Result <Self, ()>) {}
	fn bar7 (_x: (Self, Self)) {}
	fn bar8 (_x: [Self; 2]) {}
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

forward_traits! (for B -> A impl Foo);
forward_traits! (for B . 0 impl Bar);

fn main ()
{
	let mut b = B::foo9 ();

	b . foo3 ();
	b . foo2 ();
	b . foo1 ();

	let b = B::foo10 ();
	b . foo4 ();
}
