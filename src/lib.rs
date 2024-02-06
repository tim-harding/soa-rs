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
pub use soa_ref::Ref;

mod eq_impl;

/// Derive macro for the [`Soapy`] trait.
///
/// [`Soapy`]: soapy_shared::Soapy
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

    ($x:expr $(,$xs:expr)*) => {
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
