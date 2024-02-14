#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

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

mod soapy;
pub use soapy::Soapy;

mod with_ref;
pub use with_ref::WithRef;

mod soa_raw;
pub use soa_raw::SoaRaw;

mod soa_ref;
pub use soa_ref::{Ref, RefMut};

mod eq_impl;

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

// Making sure that violating borrow rules fails.
//
// # Simultaneous mutable and immutable borrows
//
// ## Okay
//
// ```
// use soapy::{Soa, Soapy, soa, SliceRef, SliceMut};
// #[derive(Soapy, PartialEq, Debug)]
// struct Foo(usize);
// let mut soa = soa![Foo(10), Foo(20)];
// let slice: SliceRef<_> = soa.as_slice();
// let slice_mut: SliceMut<_> = soa.as_mut_slice();
// ```
//
// ## Not okay
//
// ```compile_fail
// use soapy::{Soa, Soapy, soa, SliceRef, SliceMut};
// #[derive(Soapy, PartialEq, Debug)]
// struct Foo(usize);
// let mut soa = soa![Foo(10), Foo(20)];
// let slice: SliceRef<_> = soa.as_slice();
// let slice_mut: SliceMut<_> = soa.as_mut_slice();
// println!("{:?}", slice); // Added
// ```
//
// # Multiple mutable borrows
//
// ## Okay
//
// ```
// use soapy::{Soa, Soapy, soa, SliceMut};
// #[derive(Soapy, PartialEq, Debug)]
// struct Foo(usize);
// let mut soa = soa![Foo(10), Foo(20)];
// let mut slice_mut: SliceMut<_> = soa.as_mut_slice();
// let mut slice_mut_2: SliceMut<_> = slice_mut.idx_mut(..);
// slice_mut.f0_mut()[0] = 30;
// ```
//
// ## Not okay
//
// ```compile_fail
// use soapy::{Soa, Soapy, soa, SliceMut};
// #[derive(Soapy, PartialEq, Debug)]
// struct Foo(usize);
// let mut soa = soa![Foo(10), Foo(20)];
// let mut slice_mut: SliceMut<_> = soa.as_mut_slice();
// let mut slice_mut_2: SliceMut<_> = slice_mut.idx_mut(..);
// slice_mut.f0_mut()[0] = 30;
// slice_mut_2.f0_mut()[0] = 40; // Added
// ```
mod borrow_tests {}
