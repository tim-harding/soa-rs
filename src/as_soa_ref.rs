use crate::Soars;

/// Similar to [`AsRef`], but for [`Soars::Ref`].
///
/// This is primarily used to provide convenient implementations of standard
/// traits for [`Slice`].
///
/// [`Slice`]: crate::Slice
pub trait AsSoaRef {
    /// The type to get a reference of.
    ///
    /// When using the [`Soars`] derive macro, this is the type that was derived
    /// from and which is the generic parameter of [`Slice`] and [`Soa`].
    ///
    /// [`Slice`]: crate::Slice
    /// [`Soa`]: crate::Soa
    type Item: Soars;

    /// Converts this type to an SoA reference of the associated type.
    fn as_soa_ref(&self) -> <Self::Item as Soars>::Ref<'_>;
}
