use std::fmt::Debug;
use std::ops::Add;

use forward_traits::{forwardable, forward_receiver, forward_traits};

struct Algebra {}

#[forwardable]
trait AlgAdd <T>
{
	type Output;

	fn add (self, x: T, y: T) -> Self::Output;
}

impl <T> AlgAdd <T> for Algebra
where T: Add <T, Output = T>
{
	type Output = T;

	fn add (self, x: T, y: T) -> Self::Output { x + y }
}

#[allow (dead_code)]
struct RefContainer <'a, T> (&'a T);

#[forwardable]
trait WeirdAsRef <T>
{
	type Borrowed <'a>
	where T: 'a;

	fn weird_as_ref <'a> (self, x: &'a T) -> Self::Borrowed <'a>;
}

impl <T> WeirdAsRef <T> for Algebra
{
	type Borrowed <'a> = RefContainer <'a, T>
	where T: 'a;

	fn weird_as_ref <'a> (self, x: &'a T) -> Self::Borrowed <'a>
	{
		RefContainer (x)
	}
}

#[derive (Copy, Clone, PartialEq, Debug)]
struct Wrap <T> (T);

impl <T> From <T> for Wrap <T>
{
	fn from (x: T) -> Self
	{
		Self (x)
	}
}

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
	for <T> WrapAlgebra -> Algebra [Wrap <T> . 0: T, Self::Output -> Wrap <T>]
	where Wrap <T>: Sized
	impl AlgAdd <Wrap <T>>
);

forward_traits!
(
	for <T> WrapAlgebra -> Algebra
	[
		Wrap <T> . 0: T,
		for <'a> Self::Borrowed <'a> -> Wrap <RefContainer <'a, T>>
	]
	impl WeirdAsRef <Wrap <T>>
);

fn main ()
{
	assert_eq!
	(
		WrapAlgebra {} . add (Wrap::<f32> (1.0), Wrap::<f32> (2.0)),
		Wrap::<f32> (3.0)
	);
}
