/// ```
/// use soapy::{Soa, Soapy, soa, Slice};
/// #[derive(Soapy, PartialEq, Debug)]
/// #[extra_impl(Debug, PartialEq)]
/// struct Foo(usize);
/// let mut soa = soa![Foo(10), Foo(20)];
/// let slice: &Slice<_> = soa.as_slice();
/// let slice_mut: &mut Slice<_> = soa.as_mut_slice();
/// ```
mod simultaneous_mutable_and_immutable {
    /// ```compile_fail
    /// use soapy::{Soa, Soapy, soa, Slice};
    /// #[derive(Soapy, PartialEq, Debug)]
    /// #[extra_impl(Debug, PartialEq)]
    /// struct Foo(usize);
    /// let mut soa = soa![Foo(10), Foo(20)];
    /// let slice: &Slice<_> = soa.as_slice();
    /// let slice_mut: &mut Slice<_> = soa.as_mut_slice();
    /// println!("{:?}", slice); // Added
    /// ```
    mod fail {}
}

/// ```
/// use soapy::{Soa, Soapy, soa, Slice};
/// #[derive(Soapy, PartialEq, Debug)]
/// #[extra_impl(Debug, PartialEq)]
/// struct Foo(usize);
/// let mut soa = soa![Foo(10), Foo(20)];
/// let slice: &Slice<_> = soa.as_slice();
/// let slice_mut: &mut Slice<_> = soa.as_mut_slice();
/// slice_mut.f0_mut()[0] = 30;
/// ```
mod multiple_mutable_borrows {
    /// ```compile_fail
    /// use soapy::{Soa, Soapy, soa, Slice};
    /// #[derive(Soapy, PartialEq, Debug)]
    /// #[extra_impl(Debug, PartialEq)]
    /// struct Foo(usize);
    /// let mut soa = soa![Foo(10), Foo(20)];
    /// let slice: &Slice<_> = soa.as_slice();
    /// let slice_mut: &mut Slice<_> = soa.as_mut_slice();
    /// slice_mut.f0_mut()[0] = 30;
    /// slice_mut_2.f0_mut()[0] = 40; // Added
    /// ```
    mod fail {}
}

/// Regression test for <https://github.com/tim-harding/soapy/issues/2>
///
/// ```
/// use soapy::{Soa, Soapy};
///
/// #[derive(Soapy)]
/// #[extra_impl(Debug, PartialEq)]
/// struct Foo(u8);
///
/// let mut x = Soa::<Foo>::new();
/// x.push(Foo(0));
/// let mut y = Soa::<Foo>::new();
/// ```
mod swap_slices_by_mut_ref {
    /// ```compile_fail
    /// use soapy::{Soa, Soapy};
    ///
    /// #[derive(Soapy)]
    /// #[extra_impl(Debug, PartialEq)]
    /// struct Foo(u8);
    ///
    /// let mut x = Soa::<Foo>::new();
    /// x.push(Foo(0));
    /// let mut y = Soa::<Foo>::new();
    /// std::mem::swap(x.deref_mut(), y.deref_mut());
    /// ```
    mod deref_mut {}

    /// ```compile_fail
    /// use soapy::{Soa, Soapy};
    ///
    /// #[derive(Soapy)]
    /// #[extra_impl(Debug, PartialEq)]
    /// struct Foo(u8);
    ///
    /// let mut x = Soa::<Foo>::new();
    /// x.push(Foo(0));
    /// let mut y = Soa::<Foo>::new();
    /// std::mem::swap(x.as_mut(), y.as_mut()); // Changed
    /// ```
    mod as_mut {}

    /// ```compile_fail
    /// use soapy::{Soa, Soapy};
    ///
    /// #[derive(Soapy)]
    /// #[extra_impl(Debug, PartialEq)]
    /// struct Foo(u8);
    ///
    /// let mut x = Soa::<Foo>::new();
    /// x.push(Foo(0));
    /// let mut y = Soa::<Foo>::new();
    /// std::mem::swap(x.as_mut_slice(), y.as_mut_slice()); // Changed
    /// ```
    mod as_mut_slice {}
}
