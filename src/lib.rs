/*!

This crate provides general mechanisms for implementing traits on types by
forwarding an implementation provided by another type.

Two different forwarding methods are provided: Forwarding traits implemented by
members, and forwarding traits implemented by types that the receiver type can
convert to.  These methods may be used in combination on the same receiver type.
This crate fully supports generic traits and struct types.

For more details about capabilities and limitations, see the documentation pages
for the individual macros.

# Basics

In order to forward a trait, some basic things are needed.

```rust
use forward_traits::{forwardable, forward_receiver, forward_traits_via_member};
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
struct A {}

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
# struct A {}
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
# struct A {}
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
# struct A {}
# impl FooTrait for A { type Bar = Self; fn foo (&self) -> &Self::Bar { self } const BAZ: u32 = 42; }
# #[forward_receiver]
# struct B (A);
# forward_traits_via_member! (B . 0, FooTrait);
assert_eq! (<B as FooTrait>::BAZ, 42);
```

# Re-Exporting Forwardable Traits

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

# Traits in Other Crates

Forwarding traits works with traits in other crates, so long as those trait
definitions are annotated with `#[forwardable]`.

If not, then annotations must be supplied separately.  When supplying
annotations in this way, the trait is imported (or re-exported if a visibility
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
	trait
	{
		type Item;
		type IntoIter;
		fn into_iter (self) -> Self::IntoIter;
	}
);

#[forward_receiver]
struct VecWrapper <T> (Vec <T>);

// Note that we are referring to the IntoIterator in the current scope.
forward_traits_via_member! (VecWrapper . 0, IntoIterator);

// Now we can call the trait method on the wrapper type.
VecWrapper (vec! (1, 2, 3)) . into_iter ();
```

*/

mod generics;

mod partial_eval;
mod mangle;

mod uncurry;

mod transform_use;

mod type_def_info;
mod trait_def_info;
mod forwarded_trait_info;

mod member;

mod transformer;
mod conversion_transformer;
mod member_transformer;

mod forwardable;
mod supply_trait_info;
mod forward_receiver;

mod forward_via_conversion;
mod forward_via_member;

use proc_macro::TokenStream;

/**

This attribute primarily annotates trait definitions with forwarding
information. Secondarily, it is also used to make sure that the forwarding info
is properly re-exported along with the traits that it belongs to.

# Mechanism

The way that this attribute works is by defining a macro which can be used to
uncurry the trait forwarding information into another macro.

Due to limitations of macro export rules, a mangled version of that macro's name
is also created and exported into the crate root.  While these names are mangled
so that they're unlikely to cause name collisions, annotating trait definitions
of the same name in two different modules of the same crate will _definitely_
cause a problem.  Please keep trait names within a single crate unique.

# Annotating Trait Definitions

Use on trait definitions is pretty simple.  Just apply the attribute.

```rust
# use forward_traits::forwardable;
#[forwardable]
trait Foo
{
	// ...
}
```

The only consideration is that any types used for method arguments or associated
constants should be named by their fully-qualified paths.  This will prevent
name-resolution errors from occurring in the macro-generated implementations.

# Annotating Re-Exports

When re-exporting a trait that has been annotated, the use statement that does
the re-export should also be annotated.

```rust
# use forward_traits::forwardable;
mod foo
{
	# use forward_traits::forwardable;
	#[forwardable]
	pub trait Foo
	{
		// ...
	}
}

#[forwardable]
pub use foo::Foo;
```

The forwarding information is located by modifying the path of the trait passed
into the forwarding macro.  If the forwarding information isn't re-exported
alongside the trait, it won't be properly located if a path to the un-annotated
re-export is used in the forwarding macro.

*/
#[proc_macro_attribute]
pub fn forwardable (attr: TokenStream, item: TokenStream) -> TokenStream
{
	forwardable::forwardable_impl (attr, item)
}

/**

This attribute annotates type definitions with forwarding information.

# Mechanism

The way that this attribute works is by defining a macro which can be used to
uncurry the type information into another macro.

Due to limitations of macro export rules, a mangled version of that macro's name
is also created and exported into the crate root.  While these names are mangled
so that they're unlikely to cause name collisions, annotating type definitions
of the same name in two different modules of the same crate will _definitely_
cause a problem.  Please keep type names within a single crate unique.

# Usage

Usage of this attribute is pretty simple.  Just apply it to type definitions.

```rust
# use forward_traits::forward_receiver;
#[forward_receiver]
struct Foo
{
	// ...
}
```

Both regular structs and tuple-structs are supported.

# Limitations

The only types that are supported are structs.  Forwarding methods for enums can
be done in some circumstances easily enough, but figuring out what type to use
as the base for the associated types and constants is not as straightforward.
Solving those sorts of problems is beyond the scope of this crate.

```rust,compile_fail
# use forward_traits::forward_receiver;
// Error: expected `struct`
#[forward_receiver]
enum Foo
{
	// ...
}
```

*/
#[proc_macro_attribute]
pub fn forward_receiver (attr: TokenStream, item: TokenStream) -> TokenStream
{
	forward_receiver::forward_receiver_impl (attr, item)
}

/**

This macro allows the user to supply forwarding information for a trait in an
external crate that they do not control.

# Usage

The macro takes two arguments.  The first is a path to the trait that we're
providing annotations for.  The second is the annotation information.

The annotation information is basically just a subset of the parts that make up
a full trait definition.

 * `pub` or `pub (restriction)` - An optional visibility specification.  This
 isn't strictly a part of the trait's info, but will determine the visibility of
 the generated macro and trait re-export that is generated as a side-effect of
 this macro.

 * `trait` - just the keyword `trait`.

 * `<'a, T, const N: usize, ...>` - generic parameters, as would be found after
 the type identifier in a normal trait definition.  Any default values will be
 ignored, and should not be provided.

 * `where T: 'a, ...` - (optional) a where clause, as would be found in the
 trait definition.

 * `{type Error; fn try_from (x: T) -> Result <Self, Self::Error>; ...}` - A
 block containing the definitions of the trait items.  Again, any default values
 (or implementations) will be ignored, and should not be provided.

All types included should be named by their fully-qualified paths whenever
applicable.

# Mechanism

The way that this attribute works is by defining a macro which can be used to
uncurry the trait forwarding information into another macro.

Due to limitations of macro export rules, a mangled version of that macro's name
is also created and exported into the crate root.  While these names are mangled
so that they're unlikely to cause name collisions, annotating trait definitions
of the same name in two different modules of the same crate will _definitely_
cause a problem.  Please keep trait names within a single crate unique.

# Example

```rust
# use forward_traits::{supply_forwarding_info_for_trait, forward_receiver, forward_traits_via_conversion};
supply_forwarding_info_for_trait!
(
	std::iter::FromIterator,
	pub (crate) trait <A>
	{
		fn from_iter <T> (iter: T) -> Self
		where T: IntoIterator <Item = A>;
	}
);
# #[forward_receiver]
# struct VecWrapper <T> (Vec <T>);
# impl <T> From <Vec <T>> for VecWrapper <T> { fn from (vec: Vec <T>) -> Self { Self (vec) } }
# forward_traits_via_conversion! (VecWrapper -> Vec <T>, FromIterator <T>);
```

*/
#[proc_macro]
pub fn supply_forwarding_info_for_trait (input: TokenStream) -> TokenStream
{
	supply_trait_info::supply_forwarding_info_for_trait_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn __forward_trait_via_conversion (input: TokenStream) -> TokenStream
{
	forward_via_conversion::__forward_trait_via_conversion_impl (input)
}

/**

This macro generates trait implementations based on conversions from the
implementing type to a delegated type which already implements the trait.

# Usage

The first argument to the macro is a specification of the base type and the
delegated type.  This specification has the form `BaseTypeIdent ->
path::to::DelegatedType`.

The remaining arguments are descriptions of traits that should be forwarded.
The trait generic arguments, if any, will be interpreted as if the generic
parameters from the base type are in scope.

If a trait would need additional generic arguments to be introduced in order to
correctly specify the trait's generic parameters, these arguments can be
provided by prefixing the trait path with a quantifier over those parameters.
The syntax is similar to that of
Higher-Ranked Trait Bounds, except that all forms of generic parameters are
supported.  These parameters will be introduced into the `impl` scope along with
the generic parameters of the receiver type, so make sure that their names don't
collide with the receiver type's generic parameters.

Additionally, if additional where predicates need to be provided on top of those
found in the type definition and trait definition (and besides those which could
be provided automatically by the forwarding macro), then those may also be
introduced by suffixing the trait path with a where clause.

Putting that all together, a description of a trait to be forwarded might look
like this: `for <'a> path::to::Trait <&'a [T]> where T: 'a`.

# Example

```rust
# use forward_traits::supply_forwarding_info_for_trait;
#
# supply_forwarding_info_for_trait!
# (
# 	std::iter::IntoIterator,
# 	trait
# 	{
# 		type Item;
# 		type IntoIter;
#
# 		fn into_iter (self) -> Self::IntoIter;
# 	}
# );
#
# supply_forwarding_info_for_trait!
# (
# 	std::convert::TryFrom,
# 	trait <T>
# 	{
# 		type Error;
#
# 		fn try_from (value: T) -> std::result::Result <Self, Self::Error>;
# 	}
# );
#
use forward_traits::{forward_receiver, forward_traits_via_conversion};

#[derive (Debug)]
#[forward_receiver]
struct Point
{
	x: f32,
	y: f32
}

impl From <Point> for [f32; 2]
{
	fn from (p: Point) -> [f32; 2]
	{
		[p . x, p . y]
	}
}

impl From <[f32; 2]> for Point
{
	fn from (a: [f32; 2]) -> Self
	{
		Self {x: a [0], y: a [1]}
	}
}

// Make sure that the traits we want are annotated.  In this case, we've
// annotated some std traits and imported them into the local scope.

forward_traits_via_conversion! (Point -> [f32; 2], for <'a> TryFrom <&'a [f32]>, IntoIterator);

// Now we can do weird stuff, life try to construct Point from slices.

Point::try_from ([1f32, 2f32] . as_slice ()) . unwrap () . into_iter ();
Point::try_from ([1f32] . as_slice ()) . unwrap_err ();
```

# Conversions

Up to 4 different conversions may be used.  If `BaseType` were to forward an
implementation by `DelegatedType`, those conversions would be:

 * `<BaseType as std::borrow::Borrow <DelegatedType>>::borrow ()` for function
 arguments of type `&Self`.

 * `<BaseType as std::borrow::BorrowMut <DelegatedType>>::borrow_mut ()` for
 function arguments of type `&mut Self`.

 * `<BaseType as std::convert::Into <DelegatedType>>::into ()` for function
 arguments of type `Self`.

 * `<BaseType as std::convert::From <DelegatedType>>::from ()` for a return type
 of `Self`.

Any conversion that is actually used to forward a trait implementation will need
to be implemented for the receiving type.  Any conversion that is not used does
not need to be implemented.

In practice, if a trait implementation would only use borrowing conversions, it
might make more sense to use a member forward instead, as that doesn't require
that the receiver type implement any conversion traits.

Note that forwarding via conversion is the only way to forward a trait that has
a method that returns `Self` in any form.

All arguments to a trait method that have a type of some form of self are
converted, not just the method receiver.  This also allows traits to be
forwarded that require/provide methods that don't take a receiver, but still
take arguments of the receiver type.

Self types in container types like `Result` and `Box` are also converted.

*/
#[proc_macro]
pub fn forward_traits_via_conversion (input: TokenStream) -> TokenStream
{
	forward_via_conversion::forward_traits_via_conversion_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn __forward_trait_via_member (input: TokenStream) -> TokenStream
{
	forward_via_member::__forward_trait_via_member_impl (input)
}

/**

This macro generates trait implementations for a type where a member of that
type implements the trait.

# Usage

The first argument to the macro is a specification of which member should
provide the implementation of the forwarded trait.  This specification takes the
form `BaseTypeIdent . member`, where `member` is either an identifier or an
index, as appropriate for the type.

The remaining arguments are descriptions of traits that should be forwarded.
The trait generic arguments, if any, will be interpreted as if the generic
parameters from the base type are in scope.

If a trait would need additional generic arguments to be introduced in order to
correctly specify the trait's generic parameters, these arguments can be
provided by prefixing the trait path with a quantifier over those parameters.
The syntax is similar to that of
Higher-Ranked Trait Bounds, except that all forms of generic parameters are
supported.  These parameters will be introduced into the `impl` scope along with
the generic parameters of the receiver type, so make sure that their names don't
collide with the receiver type's generic parameters.

Additionally, if additional where predicates need to be provided on top of those
found in the type definition and trait definition (and besides those which could
be provided automatically by the forwarding macro), then those may also be
introduced by suffixing the trait path with a where clause.

Putting that all together, a description of a trait to be forwarded might look
like this: `for <'a> path::to::Trait <&'a [T]> where T: 'a`.

# Example

```rust
# use forward_traits::supply_forwarding_info_for_trait;
#
# supply_forwarding_info_for_trait!
# (
# 	std::ops::Index,
# 	trait <Idx>
# 	{
# 		type Output;
#
# 		fn index (&self, index: Idx) -> &Self::Output;
# 	}
# );
#
# supply_forwarding_info_for_trait!
# (
# 	std::ops::IndexMut,
# 	trait <Idx>
# 	where Self: Index <Idx>
# 	{
# 		fn index_mut (&mut self, index: Idx) -> &mut Self::Output;
# 	}
# );
#
use forward_traits::{forward_receiver, forward_traits_via_member};

#[forward_receiver]
struct Foo
{
	header: [u8; 4],
	items: Vec <u8>
}

forward_traits_via_member! (Foo . items, Index <usize>, IndexMut <usize>);
```

# Conversions

Conversions are performed via member access.  Return types cannot be converted,
as member access has no inverse.

All arguments to a trait method that have a type of some form of self are
converted, not just the method receiver.  This also allows traits to be
forwarded that require/provide methods that don't take a receiver, but still
take arguments of the receiver type.

Self types in container types like `Result` and `Box` are also converted.

*/
#[proc_macro]
pub fn forward_traits_via_member (input: TokenStream) -> TokenStream
{
	forward_via_member::forward_traits_via_member_impl (input)
}
