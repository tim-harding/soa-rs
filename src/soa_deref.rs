use crate::{Slice, Soapy};

/// [`Slice`] dereferences to this type to provide getters for the individual
/// fields as slices.
///
/// See [`Soapy::Deref`]
pub trait SoaDeref {
    /// The [`Slice`] generic parameter
    type Item: Soapy;

    /// Creates a new deref target from the given slice
    fn from_slice(slice: &Slice<Self::Item>) -> &Self;

    /// Creates a new mutable deref target from the given slice
    fn from_slice_mut(slice: &mut Slice<Self::Item>) -> &mut Self;
}
