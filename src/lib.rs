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
use forward_traits::{forwardable, forward_receiver, forward_traits};
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

Lastly, we need to specify that we want to forward a trait.  In this case, we
want to forward a trait implemented by a member, so we write:

```rust
# use forward_traits::{forwardable, forward_receiver, forward_traits};
# #[forwardable]
# trait FooTrait { type Bar; fn foo (&self) -> &Self::Bar; const BAZ: u32; }
# struct A {}
# impl FooTrait for A { type Bar = Self; fn foo (&self) -> &Self::Bar { self } const BAZ: u32 = 42; }
# #[forward_receiver]
# struct B (A);
forward_traits! (for B . 0 impl FooTrait);
```

And now we can see that the trait is properly forwarded.

```rust
# use forward_traits::{forwardable, forward_receiver, forward_traits};
# #[forwardable]
# trait FooTrait { type Bar; fn foo (&self) -> &Self::Bar; const BAZ: u32; }
# struct A {}
# impl FooTrait for A { type Bar = Self; fn foo (&self) -> &Self::Bar { self } const BAZ: u32 = 42; }
# #[forward_receiver]
# struct B (A);
# forward_traits! (for B . 0 impl FooTrait);
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
	forward_traits
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
forward_traits! (for VecWrapper . 0 impl IntoIterator);

// Now we can call the trait method on the wrapper type.
VecWrapper (vec! (1, 2, 3)) . into_iter ();
```

*/

mod uncurry;

mod generics;
mod syn;
mod fold;

mod value_transformer;
mod type_transformer;
mod transformer;

mod macros;

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
	macros::forwardable::forwardable_impl (attr, item)
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
	macros::forward_receiver::forward_receiver_impl (attr, item)
}

/**

This macro allows the user to supply forwarding information for a trait in an
external crate that they do not control.

# Usage

The macro takes two arguments.  The first is a path to the trait that we're
providing annotations for.  The second is the annotation information.

The annotation information is basically just a subset of the parts that make up
a full trait definition.

 * `pub` or `pub (restriction)` - (optional) A visibility specification.  This
   isn't strictly a part of the trait's info, but will determine the visibility
   of the generated macro and trait re-export that is generated as a side-effect
   of this macro.

 * `trait` - just the keyword `trait`.

 * `<'a, T, const N: usize, ...>` - (optional) generic parameters, as would be
   found after the type identifier in a normal trait definition.  Any default
   values will be ignored, and should not be provided.

 * `where T: 'a, ...` - (optional) a where clause, as would be found in the
   trait definition.

 * `{type Error; fn try_from (x: T) -> Result <Self, Self::Error>; ...}` - A
   block containing the definitions of the trait items.  Again, any default
   values (or implementations) will be ignored, and should not be provided.

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
# use forward_traits::{supply_forwarding_info_for_trait, forward_receiver, forward_traits};
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
# forward_traits! (for VecWrapper -> Vec <T> impl FromIterator <T>);
```

*/
#[proc_macro]
pub fn supply_forwarding_info_for_trait (input: TokenStream) -> TokenStream
{
	macros::supply_trait_info::supply_forwarding_info_for_trait_impl (input)
}

#[doc (hidden)]
#[proc_macro]
pub fn __forward_trait (input: TokenStream) -> TokenStream
{
	macros::forward_traits::__forward_trait_impl (input)
}

/**

The namesake of the crate, this macro generates trait implementations by
delegating the trait items to another type.

# Usage

The syntax for the argument to this macro is a little complicated.

## Base Type Transformation

The first part of the syntax is a specification of the base type and how to
transform it into the delegated type.  There are two such transformations
available.

 * Conversion: `Ident -> Type`.  The type named by the ident (it is expected to
   be in the same scope as the macro invocation) is transformed into the
   delegated type via conversion traits.  For arguments, the following types are
   converted using the specified traits.

   * `Self`: `std::convert::Into <DelegatedType>`
   * `&Self`: `std::convert::AsRef <DelegatedType>`
   * `&mut Self`: `std::convert::AsMut <DelegatedType>`

   Besides these specific types, `Result` and `Box` are also transformed if
   their contents are a transformable type.  All arguments of a convertible type
   are converted, not just the receiver.

   The return value may be converted as well, if it is a form of `Self` type.
   This uses the following trait.

   * `-> Self`: `std::convert::From <DelegatedType>`

   Like with arguments, `Result` and `Box` forms are also transformed.

   The conversion traits that are actually used need to be implemented for the
   base type.  Any conversion traits that are not used are not required.

 * Member access: `Ident . Ident|Index`.  The first ident is the same as with
   conversion.  The `Ident|Index` names a member of the struct to delegate to.
   An ident is required in the case of a struct with named fields, and an index
   is required in the case of a tuple struct.

   `Self`, `&Self`, and `&mut Self` typed arguments are tranformed via member
   access.  Like with conversion, `Result` and `Box` forms of transformable
   types are also transformed.  Member delegation cannot transform return
   values.

## Additional Transformations

After the base type transformation, we might want to list some other type
transformations to be applied.  The set of conversions available is the same as
for the base type transformation.  These specifications have a slightly
different syntax, as they do not rely on type annotations like the base type
transformation does.

 * Conversion: `Type -> Type`.  Instead of an ident in the first position, we
   have a full type.  The behavior is otherwise the same.

 * Member access: `Type . Ident|Index : Type`.  Not only do we have a type in
   the first position instead of an ident, but the type of the member must also
   be provided.

These transformations do not assume that the type on the left is in the same
scope as the macro invocation.  The transformations applied, and the rules that
must be followed for the types on the left side of these transformation
specifications are otherwise the same as for the base type transformation.

Note that the base type transformation only transforms forms of type `Self`.  If
the base type is explicitly written out rather than being referred to with the
`Self` type, then it won't be transformed by the base type transformation.

The base type _can_ be used on the left-hand side of an additional
transformation to specify that explicit forms of the base type should be
transformed as well.

## Forwarded Traits

Lastly, we have the actual traits to forward.

In the simplest form, this is just the path to the trait, plus a set of generic
arguments (which will be interpreted as if the base type generics are in scope).

However, additional generic paramters may be required in order to provide the
trait with all of its generic arguments.  These may be supplied by prefixing the
path to the trait with a `for <GenericParam, ...>` construction.  Keep in mind
that any generic parameters provided as part of the transformation specification
will also be in scope.

Normally, the where clause is constructed by combining the where clause of on
the type definition, the where clause provided with the transformation
specifications, and the where clause on the trait definition, and then finally
adding some additional traits necessary for the forwarding.  Some of the generic
parameters or arguments may end up requiring some additional where predicates
beyond the computed set, however.  These additional predicates may be provided
by appending a where clause to the forwarded trait specification.  If a where
clause is appended, it must also be followed by a semi-colon (`;`).

If all of these syntactic elements are required in the same specification, you
can end up with something looking like this:

```rust,ignore
for <GenericParam, ...> path::to::Trait <GenericArgument, ...> where WherePredicate, ...;
```

## Putting It All Together

There are some additional bits of glue that are required in-between the parts.

Before the base type transformation specification goes the `for` keyword.  If
there are any generic parameters which should be introduced for all trait
implementations beyond those introduced by the receiver type definition, they
may be listed in angle brackets in-between the `for` keyword and the base type
transformation specification.

Additional transformations are listed, comma-separated, inside of square brackets
(`[]`).  This angle-bracketed list comes immediately after the base type
transformation.  The angle-bracketed list may be omitted in its entirety if no
additional transformations are to be specified.

If any additional where predicates should be introduced for all trait
implementations beyond those introduced by the receiver type definition, a where
clause may be provided after the additional transformations.

The `impl` keyword comes next, marking the beginning of the list of forwarded
traits.

Finally, the forwarded traits are listed, separated by plus tokens (`+`).

The overall structure, including all optional parts, looks like this.

```rust,ignore
forward_traits!
(
	for <GenericParam, ...> BaseTransformation [AdditionalTransformation, ...]
	where WherePredicate, ...
	impl ForwardedTrait + ...
);
```

# Examples

Here we need to introduce a lifetime for one of our forwarded trait's generic
arguments.

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
use forward_traits::{forward_receiver, forward_traits};

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

forward_traits! (for Point -> [f32; 2] impl for <'a> TryFrom <&'a [f32]> + IntoIterator);

// Now we can do weird stuff, life try to construct Point from slices.

Point::try_from ([1f32, 2f32] . as_slice ()) . unwrap () . into_iter ();
Point::try_from ([1f32] . as_slice ()) . unwrap_err ();
```

Here we're just doing some boring member delegation.

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
use forward_traits::{forward_receiver, forward_traits};

#[forward_receiver]
struct Foo
{
	header: [u8; 4],
	items: Vec <u8>
}

forward_traits! (for Foo . items impl Index <usize> + IndexMut <usize>);
```

Here we're transforming more than one type at once, and we need to introduce a
generic parameter in order to specify these additional transformations.

```rust
use forward_traits::{forwardable, forward_receiver, forward_traits};

struct Algebra {}

#[forwardable]
trait Foo <T>
{
	fn foo (self, x: T);
}

impl <T> Foo <T> for Algebra
{
	fn foo (self, x: T) {}
}

struct Wrap <T> (T);

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
	for <T> WrapAlgebra -> Algebra [Wrap <T> . 0: T]
	impl Foo <Wrap <T>>
);

WrapAlgebra {} . foo (Wrap::<f32> (1.0))
```
*/
#[proc_macro]
pub fn forward_traits (input: TokenStream) -> TokenStream
{
	macros::forward_traits::forward_traits_impl (input)
}
