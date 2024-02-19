/*!

This crate provides general mechanisms for implementing traits on types by
forwarding an implementation provided by another type.

Two different forwarding methods are provided: Forwarding traits implemented by
members, and forwarding traits imlemented by types that the receiver type can
convert to.  These methods may be used in combination on the same receiver type.
This crate fully supports generic traits and struct types.

For more details about capabilities and limitations, see the documentation pages
for the inidividual macros.

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
	trait_info () () [] {type Item; type IntoIter} {fn into_iter (self) -> Self::IntoIter} {}
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

# Usage

## Trait Definitions

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

## Re-Exports

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
alongside the trait, it won't be properly located if a path to the un-annocated
re-export is used in the forwarding macro.

# Limitations

Traits that use special container types as receivers (such as `Box <T>`, or `Rc
<T>`) are not supported.  This is because the desired behavior for such a
forward is unclear.

```rust,compile_fail
# use forward_traits::forwardable;
#[forwardable]
trait Foo
{
	// Error: Containerized receivers are not supported
	fn foo (self: Box <Self>);
}
```

*/
#[proc_macro_attribute]
pub fn forwardable (attr: TokenStream, item: TokenStream) -> TokenStream
{
	info::forwardable_impl (attr, item)
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

Useage of this attribute is pretty simple.  Just apply it to type definitions.

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
	info::forward_receiver_impl (attr, item)
}

/**

This macro allows the user to supply forwarding information for a trait in an
external crate that they do not control.

# Usage

The macro takes two arguments.  The first is a path to the trait that we're
providing annotations for.  The second is the annotation information.

The annotation information consists of the following parts, in order:

 * `trait_info` - just the keyword `trait_info`.

 * `(R, S, T, ...)` - a parenthesized list of the names of the generic
 parameters of the trait, separated by commas.

 * `(Self, ...)` - a parenthesized list of the default values for those generic
 arguments which have them, separated by commas.

 * `[Self: Clone, ...]` - a bracketed list of all trait and lifetime bounds at
 the trait level, separated by commas.  This includes bounds that would have
 been written into the generics parameters in the trait definition, except for
 type equality bounds on the generic parameters.

 * `{type Foo; type Bar <'a> where Self: 'a; ...}` - a braced list of the
 declarations of all associated types in the trait, included the where clauses,
 but excluding any super-trait bounds or default values, separated by semicolons

 * `{fn foo (&self) -> Self::Foo; ...}` - a braced list of the signatures of all
 methods in the trait, including the where clauses, and including any methods
 that have default implementations, separated by semicolons.

 * `{FOO: u32; BAR: T}` - a braced list of the declarations of all associated
 constants in the trait, including there where clauses, but excluding any
 default values, separated by semicolons.

All types included should be named by their fully-qualified paths whenever
applicable.

# Examples

```rust
# use forward_traits::{supply_forwarding_info_for_trait, forward_receiver, forward_traits_via_conversion};
supply_forwarding_info_for_trait!
(
	std::iter::FromIterator,
	trait_info
		(A)
		()
		[Self: Sized]
		{}
		{fn from_iter <T> (iter: T) -> Self where T: IntoIterator <Item = A>}
		{}
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
	info::supply_forwarding_info_for_trait_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn __forward_trait_via_conversion (input: TokenStream) -> TokenStream
{
	forward::__forward_trait_via_conversion_impl (input)
}

/**

*/
#[proc_macro]
pub fn forward_traits_via_conversion (input: TokenStream) -> TokenStream
{
	forward::forward_traits_via_conversion_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn __forward_trait_via_member (input: TokenStream) -> TokenStream
{
	forward::__forward_trait_via_member_impl (input)
}

/**

*/
#[proc_macro]
pub fn forward_traits_via_member (input: TokenStream) -> TokenStream
{
	forward::forward_traits_via_member_impl (input)
}
