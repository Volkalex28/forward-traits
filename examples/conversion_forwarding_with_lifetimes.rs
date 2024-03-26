use forward_traits
::{
	supply_forwarding_info_for_trait,
	forward_receiver,
	forward_traits
};

supply_forwarding_info_for_trait!
(
	std::convert::TryFrom,
	trait <T>
	{
		type Error;
		fn try_from (value: T) -> std::result::Result <Self, Self::Error>;
	}
);

#[allow (dead_code)]
#[forward_receiver]
#[derive (Debug)]
struct Point
{
	x: f32,
	y: f32
}

impl From <[f32; 2]> for Point
{
	fn from (a: [f32; 2]) -> Self
	{
		Self {x: a [0], y: a [1]}
	}
}

forward_traits! (for Point -> [f32; 2] impl for <'a> TryFrom <&'a [f32]>);

fn main ()
{
	Point::try_from ([1f32, 2f32] . as_slice ()) . unwrap ();
	Point::try_from ([1f32] . as_slice ()) . unwrap_err ();
}
