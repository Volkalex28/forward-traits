use forward_traits::supply_forwarding_info_for_trait;

supply_forwarding_info_for_trait!
(
	std::ops::Index,
	trait <Idx>
	{
		type Output;
		fn index (&self, index: Idx) -> &Self::Output;
	}
);

supply_forwarding_info_for_trait!
(
	std::ops::IndexMut,
	trait <Idx>
	where Self: Index <Idx>
	{
		fn index_mut (&mut self, index: Idx) -> &mut Self::Output;
	}
);

use forward_traits::{forward_receiver, forward_traits};

#[allow (dead_code)]
#[forward_receiver]
struct Foo
{
	header: [u8; 4],
	items: Vec <u8>
}

forward_traits! (for Foo . items impl Index <usize> + IndexMut <usize>);

fn main ()
{
	// Stuff
}
