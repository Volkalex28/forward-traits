/*!
This crate provides general mechanisms for implementing traits on types by
forwarding an implementation provided by another type.

Two different forwarding methods are provided: Forwarding traits implemented by
members, and forwarding traits imlemented by types that the receiver type can
convert to.  These methods may be used in combination on the same receiver type.
This crate fully supports generic traits and struct types.

For more details about capabilities and limitations, see the documentation pages
for the inidividual macros.

## Basics

In order to forward a trait, some basic things are needed.

```rust
use forward_traits::{forwardable, forward_receiver, forward_trait_via_member};
```

We need a trait definition which is annotated with information which is used to
generate forwarded implementations.  This is done by applying the
`#[forwardable]` attribute to the definition.

```rust
# use forward_traits::forwardable;
#[forwardable]
trait FooTrait
{
	type Bar;

	fn foo (&self) -> &Self::Bar;

	const BAZ: u32;
}
```

Then we need a type which initially implements this trait.

```rust
# trait FooTrait { type Bar; fn foo (&self) -> &Self::Bar; const BAZ: u32; }
struct A {};

impl FooTrait for A
{
	type Bar = Self;

	fn foo (&self) -> &Self::Bar { self }

	const BAZ: u32 = 42;
}
```

Next, we need a type for which we want to implement this trait by forwarding the
implementation found on the initially implementing type.  There are a few
different ways to define such a type, but here we will demonstrate the newtype
idiom.  This type needs to be annotated with the `#[forward_receiver]`
attribute.

```rust
# use forward_traits::forward_receiver;
# struct A {};
#[forward_receiver]
struct B (A);
```

Lastly, we need to specify that we want to forward a trait using one of the
forwarding macros. In this case, we want to forward a trait implemented by a
member, so we write:

```rust
# use forward_traits::{forwardable, forward_receiver, forward_traits_via_member};
# #[forwardable]
# trait FooTrait { type Bar; fn foo (&self) -> &Self::Bar; const BAZ: u32; }
# struct A {};
# impl FooTrait for A { type Bar = Self; fn foo (&self) -> &Self::Bar { self } const BAZ: u32 = 42; }
# #[forward_receiver]
# struct B (A);
forward_traits_via_member! (B . 0, FooTrait);
```

And now we can see that the trait is properly forwarded.

```rust
# use forward_traits::{forwardable, forward_receiver, forward_traits_via_member};
# #[forwardable]
# trait FooTrait { type Bar; fn foo (&self) -> &Self::Bar; const BAZ: u32; }
# struct A {};
# impl FooTrait for A { type Bar = Self; fn foo (&self) -> &Self::Bar { self } const BAZ: u32 = 42; }
# #[forward_receiver]
# struct B (A);
# forward_traits_via_member! (B . 0, FooTrait);
assert_eq! (<B as FooTrait>::BAZ, 42);
```

## Re-exporting Forwardable Traits

When re-exporting forwardable traits, the `#[forwardable]` attribute should be
applied to the use statement as well.  Note that the attribute will interpret
every item in the use tree as a trait that should be forwardable.  If you want
to re-export items that aren't forwardable traits from the same module(s),
you'll need to separate those re-exports out into another use statement;

```rust
use forward_traits::forwardable;

mod inner
{
	use forward_traits::forwardable;

	#[forwardable]
	pub trait Foo {}

	pub struct Bar {}
}

#[forwardable]
pub use inner::Foo;

pub use inner::Bar;
```

## Traits in Other Crates

Forwarding traits works with traits in other crates, so long as those trait
definitions are annotated with `#[forwardable]`.

If not, then annotations must be supplied separately.  When supplying
annotations in this way, the trait is imported (or-rexported if a visibilty
modifier is supplied) at the location of the annotation macro.  When forwarding
this trait, you must refer to this import/re-export (or a re-export thereof).

```rust
use forward_traits
::{
	supply_forwarding_info_for_trait,
	forward_receiver,
	forward_traits_via_member
};

// This has the side-effect of importing IntoIterator into the current scope.
supply_forwarding_info_for_trait!
(
	std::iter::IntoIterator,
	trait_info () () [] {Item, IntoIter} {fn into_iter (self) -> Self::IntoIter} {}
);

#[forward_receiver]
struct VecWrapper <T> (Vec <T>);

// Note that we are referring to the IntoIterator in the current scope.
forward_traits_via_member! (VecWrapper . 0, IntoIterator);

// Now we can call the trait method on the wrapper type.
VecWrapper (vec! (1, 2, 3)) . into_iter ();
```
*/

mod syntax;
mod info;
mod forward;
mod uncurry;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn forwardable (attr: TokenStream, item: TokenStream) -> TokenStream
{
	info::forwardable_impl (attr, item)
}

#[proc_macro_attribute]
pub fn forward_receiver (attr: TokenStream, item: TokenStream) -> TokenStream
{
	info::forward_receiver_impl (attr, item)
}

#[proc_macro]
pub fn supply_forwarding_info_for_trait (input: TokenStream) -> TokenStream
{
	info::supply_forwarding_info_for_trait_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn forward_trait_via_conversion (input: TokenStream) -> TokenStream
{
	forward::forward_trait_via_conversion_impl (input)
}

#[proc_macro]
pub fn forward_traits_via_conversion (input: TokenStream) -> TokenStream
{
	forward::forward_traits_via_conversion_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn forward_trait_via_member (input: TokenStream) -> TokenStream
{
	forward::forward_trait_via_member_impl (input)
}

#[proc_macro]
pub fn forward_traits_via_member (input: TokenStream) -> TokenStream
{
	forward::forward_traits_via_member_impl (input)
}
