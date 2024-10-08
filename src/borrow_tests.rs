/// ```
/// use soa_rs::{Soa, Soars, soa, Slice};
/// #[derive(Soars, PartialEq, Debug)]
/// #[soa_derive(Debug, PartialEq)]
/// struct Foo(usize);
/// let mut soa = soa![Foo(10), Foo(20)];
/// let slice: &Slice<_> = soa.as_ref();
/// let slice_mut: &mut Slice<_> = soa.as_mut();
/// ```
mod simultaneous_mutable_and_immutable {
    /// ```compile_fail,E0502
    /// use soa_rs::{Soa, Soars, soa, Slice};
    /// #[derive(Soars, PartialEq, Debug)]
    /// #[soa_derive(Debug, PartialEq)]
    /// struct Foo(usize);
    /// let mut soa = soa![Foo(10), Foo(20)];
    /// let slice: &Slice<_> = soa.as_ref();
    /// let slice_mut: &mut Slice<_> = soa.as_mut();
    /// println!("{:?}", slice); // Added
    /// ```
    mod fail {}
}

/// ```
/// use soa_rs::{Soa, Soars, soa, Slice};
/// #[derive(Soars, PartialEq, Debug)]
/// #[soa_derive(Debug, PartialEq)]
/// struct Foo(usize);
/// let mut soa = soa![Foo(10), Foo(20)];
/// let slice: &Slice<_> = soa.as_ref();
/// let slice_mut: &mut Slice<_> = soa.as_mut();
/// slice_mut.f0_mut()[0] = 30;
/// ```
mod multiple_mutable_borrows {
    /// ```compile_fail,E0499
    /// use soa_rs::{Soa, Soars, soa, Slice};
    /// #[derive(Soars, PartialEq, Debug)]
    /// #[soa_derive(Debug, PartialEq)]
    /// struct Foo(usize);
    /// let mut soa = soa![Foo(10), Foo(20)];
    /// let slice: &Slice<_> = soa.as_ref();
    /// let slice_mut: &mut Slice<_> = soa.as_mut();
    /// let slice_mut_2: &mut Slice<_> = soa.as_mut(); // Added
    /// slice_mut.f0_mut()[0] = 30;
    /// slice_mut_2.f0_mut()[0] = 40; // Added
    /// ```
    mod fail {}
}

/// Regression test for <https://github.com/tim-harding/soa_rs/issues/2>
///
/// ```
/// use soa_rs::{Soa, Soars};
///
/// #[derive(Soars)]
/// #[soa_derive(Debug, PartialEq)]
/// struct Foo(u8);
///
/// let mut x = Soa::<Foo>::new();
/// x.push(Foo(0));
/// let mut y = Soa::<Foo>::new();
/// ```
mod swap_slices_by_mut_ref {
    /// ```compile_fail,E0277
    /// use soa_rs::{Soa, Soars};
    /// use std::ops::DerefMut; // Added
    ///
    /// #[derive(Soars)]
    /// #[soa_derive(Debug, PartialEq)]
    /// struct Foo(u8);
    ///
    /// let mut x = Soa::<Foo>::new();
    /// x.push(Foo(0));
    /// let mut y = Soa::<Foo>::new();
    /// std::mem::swap(x.deref_mut(), y.deref_mut()); // Added
    /// ```
    mod deref_mut {}

    /// ```compile_fail,E0277
    /// use soa_rs::{Soa, Soars, Slice}; // Changed
    ///
    /// #[derive(Soars)]
    /// #[soa_derive(Debug, PartialEq)]
    /// struct Foo(u8);
    ///
    /// let mut x = Soa::<Foo>::new();
    /// x.push(Foo(0));
    /// let mut y = Soa::<Foo>::new();
    /// std::mem::swap::<Slice<_>>(x.as_mut(), y.as_mut()); // Added
    /// ```
    mod as_mut {}
}
