use crate::{SliceMut, SliceRef, Soapy};

/// A compile-time, fixed-size SoA array.
///
/// What [`Slice`] is to `[T]`, SoA arrays are to `[T; N]`. They are useful
/// whenever you have `const` data you want to structure in SoA form.
///
/// When deriving [`Soapy`] for some type `Foo`, a struct `FooArray` is also
/// created which implements this trait. The primary way to create a SoA array
/// is to use the `FooArray::from_array` method for that type, which cannot be
/// included in this trait because it is `const`.
///
/// [`Slice`]: crate::Slice
pub trait SoaArray {
    /// The type that the SoA array stores.
    ///
    /// When using the [`Soapy`] derive macro, this is the type that was derived
    /// from and which is the generic parameter of [`Slice`] and [`Soa`].
    ///
    /// [`Slice`]: crate::Slice
    /// [`Soa`]: crate::Soa
    type Item: Soapy;

    /// Returns a [`Slice`] containing the entire array.
    ///
    /// [`Slice`]: crate::Slice
    fn as_slice(&self) -> SliceRef<'_, Self::Item>;

    /// Returns a mutable [`Slice`] containing the entire array.
    ///
    /// [`Slice`]: crate::Slice
    fn as_mut_slice(&mut self) -> SliceMut<'_, Self::Item>;
}
