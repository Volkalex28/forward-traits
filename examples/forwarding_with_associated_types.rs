use forward_traits::{forwardable, forward_receiver, forward_traits};

#[forwardable]
trait Convert <T>
{
	fn convert (x: T) -> Self;
}

impl Convert <usize> for usize
{
	fn convert (x: usize) -> Self { x }
}

#[forwardable]
trait Accumulatable: Convert <Self::Accumulator>
{
	type Accumulator;

	fn zero_accumulator () -> Self::Accumulator;
}

#[forwardable]
trait Acc <T>: Accumulatable
{
	fn accumulate (acc: &mut Self::Accumulator, x: T);
}

impl Accumulatable for usize
{
	type Accumulator = usize;

	fn zero_accumulator () -> Self::Accumulator { 0 }
}

impl Acc <usize> for usize
{
	fn accumulate (acc: &mut Self::Accumulator, x: usize) { *acc += x; }
}

#[forward_receiver]
#[derive (Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Wrap <T> (T);

impl <T> From <T> for Wrap <T>
where T: Accumulatable
{
	fn from (x: T) -> Self
	{
		Self (x)
	}
}

forward_traits! (for Wrap -> T impl Convert <T::Accumulator> where T: Accumulatable;);
forward_traits! (for Wrap . 0 impl Accumulatable + Acc <T>);

fn main ()
{
	let mut acc = Wrap::<usize>::zero_accumulator ();
	Wrap::<usize>::accumulate (&mut acc, 1);
	assert_eq! (Wrap::<usize>::convert (acc) . 0, 0usize);
}
