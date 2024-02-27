use crate::Soapy;

/// Similar to [`AsRef`], but for [`Soapy::Ref`].
///
/// This is primarily used to provide convenient implementations of standard
/// traits for [`Slice`].
///
/// [`Slice`]: crate::Slice
pub trait AsSoaRef {
    /// The type to get a reference of.
    ///
    /// When using the [`Soapy`] derive macro, this is the type that was derived
    /// from and which is the generic parameter of [`Slice`] and [`Soa`].
    ///
    /// [`Slice`]: crate::Slice
    /// [`Soa`]: crate::Soa
    type Item: Soapy;

    /// Converts this type to an SoA reference of the associated type.
    fn as_soa_ref(&self) -> <Self::Item as Soapy>::Ref<'_>;
}
