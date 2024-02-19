//! # Soapy
//!
//! Soapy makes it simple to work with the structure-of-arrays memory layout.
//! What [`Vec`] is to array-of-structures, [`Soa`] is to structure-of-arrays.
//!
//! # Examples
//!
//! First, derive [`Soapy`] for your type:
//! ```
//! # use soapy::{soa, Soapy};
//! #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
//! struct Example {
//!     foo: u8,
//!     bar: u16,
//! }
//! ```
//!
//! You can create an [`Soa`] explicitly:
//! ```
//! # use soapy::{soa, Soapy, Soa};
//! # #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
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
//! # use soapy::{soa, Soapy};
//! # #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
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
//! # use soapy::{soa, Soapy};
//! # #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
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
//! assert_eq!(soa.idx(3), Example { foo: 7, bar: 8 });
//! assert_eq!(soa.idx(1..3), [Example { foo: 3, bar: 4 }, Example { foo: 5, bar: 6 }]);
//! ```
//!
//! The usual [`Vec`] APIs work normally.
//! ```
//! # use soapy::{soa, Soapy, Soa};
//! # #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
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
//! # use soapy::{soa, Soapy};
//! # #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
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
//! # use soapy::{soa, Soapy};
//! # #[derive(Soapy, Debug, Clone, Copy, PartialEq)]
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
//! # use soapy::{soa, Soapy};
//! #[derive(Soapy)]
//! struct Example(u8);
//! let soa = soa![Example(5), Example(10)];
//! assert_eq!(soa.f0(), [5, 10]);
//! ```
#![warn(missing_docs)]

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

mod soapy;
pub use soapy::Soapy;

mod with_ref;
pub use with_ref::WithRef;

mod soa_raw;
#[doc(hidden)]
pub use soa_raw::SoaRaw;

mod soa_ref;
pub use soa_ref::{Ref, RefMut};

mod chunks_exact;
pub use chunks_exact::ChunksExact;

mod eq_impl;
mod iter_raw;

/// Derive macro for the [`Soapy`] trait.
///
/// Deriving Soapy for some struct `Foo` will create the following additional
/// structs:
///
/// | Struct      | Field type | Use                                          |
/// |-------------|------------|----------------------------------------------|
/// | `FooSoaRaw` | `*mut F`   | Low-level, unsafe memory handling for SoA    |
/// | `FooRef`    | `&F`       | Immutable SoA element reference              |
/// | `FooRefMut` | `&mut F`   | Mutable SoA element reference                |
/// | `FooDeref`  |            | SoA [`Deref`] target, provides slice getters |
///
/// The [`Soapy`] trait implementation for `Foo` references these as associated
/// types. [`WithRef`] is also implemented for `Foo`, `FooRef`, and `FooRefMut`.
///
/// # Alignment
///
/// Individual fields can be tagged with the `align` attribute to raise their
/// alignment. The slice for that field will start at a multiple of the
/// requested alignment if it is greater than or equal to the alignment of the
/// field's type. This can be useful for vector operations.
///
/// ```
/// # use soapy::{Soapy};
/// #[derive(Soapy)]
/// struct Foo(#[align(8)] u8);
/// ```
///
/// [`Deref`]: std::ops::Deref
pub use soapy_derive::Soapy;

/// Creates a [`Soa`] containing the arguments.
///
/// `soa!` allows [`Soa`]s to be defined with the same syntax as array
/// expressions. There are two forms of this macro:
///
/// - Create a [`Soa`] containing a given list of elements:
/// ```
/// # use soapy::{Soapy, soa};
/// # #[derive(Soapy, Debug, PartialEq, Copy, Clone)]
/// # struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2), Foo(3, 4)];
/// assert_eq!(soa, [Foo(1, 2), Foo(3, 4)]);
/// ```
///
/// - Create a [`Soa`] from a given element and size:
///
/// ```
/// # use soapy::{Soapy, soa};
/// # #[derive(Soapy, Debug, PartialEq, Copy, Clone)]
/// # struct Foo(u8, u16);
/// let soa = soa![Foo(1, 2); 2];
/// assert_eq!(soa, [Foo(1, 2); 2]);
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
            let mut out = $crate::Soa::with($elem);
            for _ in 1..$n {
                let first = out.first();
                // SAFETY: We already added the first element in Soa::with
                let first = unsafe { first.unwrap_unchecked() };
                let clone = $crate::WithRef::with_ref(&first, |first| first.clone());
                out.push(clone);
            }
            out
        }
    };
}

/// Making sure that violating borrow rules fails.
///
/// # Simultaneous mutable and immutable borrows
///
/// ## Okay
///
/// ```
/// use soapy::{Soa, Soapy, soa, Slice};
/// #[derive(Soapy, PartialEq, Debug)]
/// struct Foo(usize);
/// let mut soa = soa![Foo(10), Foo(20)];
/// let slice: &Slice<_> = soa.as_slice();
/// let slice_mut: &mut Slice<_> = soa.as_mut_slice();
/// ```
///
/// ## Not okay
///
/// ```compile_fail
/// use soapy::{Soa, Soapy, soa, Slice};
/// #[derive(Soapy, PartialEq, Debug)]
/// struct Foo(usize);
/// let mut soa = soa![Foo(10), Foo(20)];
/// let slice: &Slice<_> = soa.as_slice();
/// let slice_mut: &mut Slice<_> = soa.as_mut_slice();
/// println!("{:?}", slice); // Added
/// ```
///
/// # Multiple mutable borrows
///
/// ## Okay
///
/// ```
/// use soapy::{Soa, Soapy, soa, Slice};
/// #[derive(Soapy, PartialEq, Debug)]
/// struct Foo(usize);
/// let mut soa = soa![Foo(10), Foo(20)];
/// let slice: &Slice<_> = soa.as_slice();
/// let slice_mut: &mut Slice<_> = soa.as_mut_slice();
/// slice_mut.f0_mut()[0] = 30;
/// ```
///
/// ## Not okay
///
/// ```compile_fail
/// use soapy::{Soa, Soapy, soa, Slice};
/// #[derive(Soapy, PartialEq, Debug)]
/// struct Foo(usize);
/// let mut soa = soa![Foo(10), Foo(20)];
/// let slice: &Slice<_> = soa.as_slice();
/// let slice_mut: &mut Slice<_> = soa.as_mut_slice();
/// slice_mut.f0_mut()[0] = 30;
/// slice_mut_2.f0_mut()[0] = 40; // Added
/// ```
///
/// # Swapping slices by mut reference
/// Regression test for https://github.com/tim-harding/soapy/issues/2
///
/// ## Okay
///
/// ```
/// use soapy::{Soa, Soapy};
///
/// #[derive(Soapy)]
/// struct Foo(u8);
///
/// let mut x = Soa::<Foo>::new();
/// x.push(Foo(0));
/// let mut y = Soa::<Foo>::new();
/// ```
///
/// ## Not Okay
///
/// ```no_compile
/// use soapy::{Soa, Soapy};
///
/// #[derive(Soapy)]
/// struct Foo(u8);
///
/// let mut x = Soa::<Foo>::new();
/// x.push(Foo(0));
/// let mut y = Soa::<Foo>::new();
/// std::mem::swap(x.as_mut_slice(), y.as_mut_slice());
/// ```
///
/// ```no_compile
/// use soapy::{Soa, Soapy};
///
/// #[derive(Soapy)]
/// struct Foo(u8);
///
/// let mut x = Soa::<Foo>::new();
/// x.push(Foo(0));
/// let mut y = Soa::<Foo>::new();
/// std::mem::swap(x.deref_mut(), y.deref_mut());
/// ```
///
/// ```no_compile
/// use soapy::{Soa, Soapy};
///
/// #[derive(Soapy)]
/// struct Foo(u8);
///
/// let mut x = Soa::<Foo>::new();
/// x.push(Foo(0));
/// let mut y = Soa::<Foo>::new();
/// std::mem::swap(x.as_mut(), y.as_mut());
/// ```
mod borrow_tests {}

#[doc = include_str!("../README.md")]
mod readme_tests {}
