//! # Soars
//!
//! Soars makes it simple to work with the structure-of-arrays memory layout.
//! What [`Vec`] is to array-of-structures, [`Soa`] is to structure-of-arrays.
//!
//! # Examples
//!
//! First, derive [`Soars`] for your type:
//! ```
//! # use soa_rs::{soa, Soars};
//! #[derive(Soars, Debug, Clone, Copy, PartialEq)]
//! #[soa_derive(Debug, PartialEq)]
//! struct Example {
//!     foo: u8,
//!     bar: u16,
//! }
//! ```
//!
//! You can create an [`Soa`] explicitly:
//! ```
//! # use soa_rs::{soa, Soars, Soa};
//! # #[derive(Soars, Debug, Clone, Copy, PartialEq)]
//! # #[soa_derive(Debug, PartialEq)]
//! # struct Example {
//! #     foo: u8,
//! #     bar: u16,
//! # }
//! let mut soa: Soa<Example> = Soa::new();
//! soa.push(Example { foo: 1, bar: 2 });
//! ```
//!
//! ...or by using the [`soa!`] macro.
//! ```
//! # use soa_rs::{soa, Soars};
//! # #[derive(Soars, Debug, Clone, Copy, PartialEq)]
//! # #[soa_derive(Debug, PartialEq)]
//! # struct Example {
//! #     foo: u8,
//! #     bar: u16,
//! # }
//! let mut soa = soa![Example { foo: 1, bar: 2 }, Example { foo: 3, bar: 4 }];
//! ```
//!
//! An SoA can be indexed and sliced just like a `&[T]`. Use `idx` in lieu of
//! the index operator.
//! ```
//! # use soa_rs::{soa, Soars, AsSlice};
//! # #[derive(Soars, Debug, Clone, Copy, PartialEq)]
//! # #[soa_derive(Debug, PartialEq)]
//! # struct Example {
//! #     foo: u8,
//! #     bar: u16,
//! # }
//! let mut soa = soa![
//!     Example { foo: 1, bar: 2 },
//!     Example { foo: 3, bar: 4 },
//!     Example { foo: 5, bar: 6 },
//!     Example { foo: 7, bar: 8 }
//! ];
//! assert_eq!(soa.idx(3), ExampleRef { foo: &7, bar: &8 });
//! assert_eq!(
//!     soa.idx(1..3),
//!     soa![
//!         Example { foo: 3, bar: 4 },
//!         Example { foo: 5, bar: 6 },
//!     ],
//! );
//! ```
//!
//! The usual [`Vec`] APIs work normally.
//! ```
//! # use soa_rs::{soa, Soars, Soa};
//! # #[derive(Soars, Debug, Clone, Copy, PartialEq)]
//! # #[soa_derive(Debug, PartialEq)]
//! # struct Example {
//! #     foo: u8,
//! #     bar: u16,
//! # }
//! let mut soa = Soa::<Example>::new();
//! soa.push(Example { foo: 1, bar: 2 });
//! soa.push(Example { foo: 3, bar: 4 });
//! soa.insert(0, Example { foo: 5, bar: 6 });
//! assert_eq!(soa.pop(), Some(Example { foo: 3, bar: 4 }));
//! for mut el in &mut soa {
//!     *el.bar += 10;
//! }
//! assert_eq!(soa.bar(), [16, 12]);
//! ```
//!
//! # Field getters
//!
//! You can access the fields as slices.
//! ```
//! # use soa_rs::{soa, Soars};
//! # #[derive(Soars, Debug, Clone, Copy, PartialEq)]
//! # #[soa_derive(Debug, PartialEq)]
//! # struct Example {
//! #     foo: u8,
//! #     bar: u16,
//! # }
//! let mut soa = soa![
//!     Example { foo: 1, bar: 2 },
//!     Example { foo: 3, bar: 4 },
//! ];
//! assert_eq!(soa.foo(), [1, 3]);
//! ```
//!
//! Postpend `_mut` for mutable slices.
//! ```
//! # use soa_rs::{soa, Soars};
//! # #[derive(Soars, Debug, Clone, Copy, PartialEq)]
//! # #[soa_derive(Debug, PartialEq)]
//! # struct Example {
//! #     foo: u8,
//! #     bar: u16,
//! # }
//! # let mut soa = soa![
//! #     Example { foo: 1, bar: 2 },
//! #     Example { foo: 3, bar: 4 },
//! # ];
//! for foo in soa.foo_mut() {
//!     *foo += 10;
//! }
//! assert_eq!(soa.foo(), [11, 13]);
//! ```
//!
//! For tuple structs, prepend the field number with `f`:
//! ```
//! # use soa_rs::{soa, Soars};
//! #[derive(Soars)]
//! # #[soa_derive(Debug, PartialEq)]
//! struct Example(u8);
//! let soa = soa![Example(5), Example(10)];
//! assert_eq!(soa.f0(), [5, 10]);
//! ```
//!
//! # Serde
//!
//! [`serde`](https://serde.rs/) support is enabled by the `serde` feature
//! flag.
//!
//! ```ignore
//! #[derive(Soars, serde::Deserialize)]
//! #[soa_derive(include(Ref), serde::Serialize)]
//! struct Test(u32);
//! ```
//!
//! [`Soars`]: soa_rs_derive::Soars
#![warn(missing_docs)]
#![no_std]

/// `::alloc` is not available at the top level the way that `::std` and
/// `::core` are. We don't want to do the `extern` inside the macro because
/// multiple invocations will cause name conflicts. Therefore, we re-export it
/// from the library for consumption instead.
#[doc(hidden)]
pub extern crate alloc as __alloc;

mod soa;
pub use soa::Soa;

mod index;
pub use index::SoaIndex;

mod into_iter;
pub use into_iter::IntoIter;

mod iter;
pub use iter::Iter;

mod iter_mut;
pub use iter_mut::IterMut;

mod slice;
pub use slice::Slice;

mod slice_mut;
pub use slice_mut::SliceMut;

mod slice_ref;
pub use slice_ref::SliceRef;

mod soa_deref;
pub use soa_deref::SoaDeref;

mod soars;
pub use soars::Soars;

mod from_soa_ref;
pub use from_soa_ref::{FromSoaRef, SoaRefToOwned};

mod soa_raw;
#[doc(hidden)]
pub use soa_raw::SoaRaw;

mod chunks_exact;
pub use chunks_exact::ChunksExact;

mod iter_raw;

mod as_slice;
pub use as_slice::{AsMutSlice, AsSlice};

mod as_soa_ref;
pub use as_soa_ref::AsSoaRef;

#[cfg(feature = "serde")]
mod serde;

/// Derive macro for the [`Soars`] trait.
///
/// Deriving Soars for some struct `Foo` will create the following additional
/// structs:
///
/// | Struct         | Field type | Use                                          |
/// |----------------|------------|----------------------------------------------|
/// | `FooSoaRaw`    | `*mut T`   | Low-level, unsafe memory handling for SoA    |
/// | `FooRef`       | `&T`       | SoA element reference                        |
/// | `FooRefMut`    | `&mut T`   | Mutable SoA element reference                |
/// | `FooSlices`    | `&[T]`     | SoA fields                                   |
/// | `FooSlicesMut` | `&mut [T]` | Mutable SoA fields                           |
/// | `FooArray`     | `[T; N]`   | `const`-compatible SoA                       |
/// | `FooDeref`     |            | SoA [`Deref`] target, provides slice getters |
///
/// The [`Soars`] trait implementation for `Foo` references these as associated
/// types. [`AsSoaRef`] is also implemented for `Foo`, `FooRef`, and `FooRefMut`.
///
/// # Arrays
///
/// The `FooArray` type is only generated when the `#[soa_array]` attribute is
/// added to the struct. Only structs without interior mutability support this
/// attribute for the time being, due to [this
/// issue](https://github.com/rust-lang/rust/issues/80384). SOA array types are
/// stack-allocated like normal arrays and are `const`-initializable.
///
/// # Derive for generated types
///
/// The `soa_derive` attribute can be used to derive traits for the generated
/// types. `Copy` and `Clone` are added automatically for `FooRef` and
/// `FooSlices`. In the following example, we have the following trait
/// implementations:
///
/// | Struct         | `Copy`/`Clone` | `Debug`/`PartialEq` | `Eq` | `PartialOrd` |
/// |----------------|----------------|---------------------|------|--------------|
/// | `FooRef`       | ✅             | ✅                  | ✅   |              |
/// | `FooRefMut`    |                | ✅                  | ✅   | ✅           |
/// | `FooSlices`    | ✅             | ✅                  |      |              |
/// | `FooSlicesMut` |                | ✅                  |      | ✅           |
/// | `FooArray`     |                | ✅                  |      | ✅           |
///
/// ```
/// # use soa_rs::{Soars};
/// #[derive(Soars)]
/// #[soa_derive(Debug, PartialEq)]
/// #[soa_derive(include(Ref, RefMut), Eq)]
/// #[soa_derive(exclude(Ref, Slices), PartialOrd)]
/// struct Foo(u8);
/// ```
///
/// # Alignment
///
/// Individual fields can be tagged with the `align` attribute to raise their
/// alignment. The slice for that field will start at a multiple of the
/// requested alignment if it is greater than or equal to the alignment of the
/// field's type. This can be useful for vector operations.
///
/// ```
/// # use soa_rs::{Soars};
/// # #[derive(Soars)]
/// # #[soa_derive(Debug, PartialEq)]
/// struct Foo(#[align(8)] u8);
/// ```
///
/// [`Deref`]: core::ops::Deref
pub use soa_rs_derive::Soars;

/// Derive macro for the [`FromSoaRef`] trait.
///
/// This macro generates an implementation that constructs an owned value
/// by cloning all fields from an SoA reference.
///
/// # Example
///
/// ```
/// # use soa_rs::{Soars, FromSoaRef, soa};
/// #[derive(Soars, FromSoaRef, Debug, PartialEq, Clone)]
/// #[soa_derive(Debug)]
/// struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2), Foo(3, 4)];
/// let point_ref = soa.idx(1);
/// let owned = Foo::from_soa_ref(point_ref);
/// assert_eq!(owned, Foo(3, 4));
/// ```
pub use soa_rs_derive::FromSoaRef;

/// Creates a [`Soa`] containing the arguments.
///
/// `soa!` allows [`Soa`]s to be defined with the same syntax as array
/// expressions. There are two forms of this macro:
///
/// - Create a [`Soa`] containing a given list of elements:
/// ```
/// # use soa_rs::{Soars, soa};
/// # #[derive(Soars, Debug, PartialEq, Copy, Clone)]
/// # #[soa_derive(Debug, PartialEq, PartialOrd)]
/// # struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2), Foo(3, 4)];
/// assert_eq!(soa, soa![Foo(1, 2), Foo(3, 4)]);
/// ```
///
/// - Create a [`Soa`] from a given element and size:
///
/// ```
/// # use soa_rs::{Soars, soa};
/// # #[derive(Soars, Debug, PartialEq, Copy, Clone)]
/// # #[soa_derive(Debug, PartialEq)]
/// # struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2); 2];
/// assert_eq!(soa, soa![Foo(1, 2), Foo(1, 2)]);
/// ```
#[macro_export]
macro_rules! soa {
    () => {
        $crate::Soa::new()
    };

    ($x:expr $(,$xs:expr)* $(,)?) => {
        {
            let mut out = $crate::Soa::with($x);
            $(
            out.push($xs);
            )*
            out
        }
    };

    ($elem:expr; 0) => {
        soa![]
    };

    ($elem:expr; 1) => {
        $crate::Soa::with($elem)
    };

    ($elem:expr; $n:expr) => {
        {
            let elem = $elem;
            let mut out = $crate::Soa::with(elem.clone());

            let mut i = 2;
            while i < $n {
                out.push(elem.clone());
            }

            out.push(elem);
            out
        }
    };
}

#[doc = include_str!("../README.md")]
mod readme_tests {}

mod borrow_tests;
