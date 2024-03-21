use crate::{SliceMut, SliceRef, Soars};

/// Similar to `AsRef<Slice>`, but returns a value type rather than an
/// reference.
pub trait AsSlice {
    /// The type that the slice contains.
    type Item: Soars;

    /// Returns a [`SliceRef`] containing the entire array.
    fn as_slice(&self) -> SliceRef<'_, Self::Item>;
}

/// Similar to `AsMut<Slice>`, but returns a value type rather than a mutable
/// reference.
pub trait AsMutSlice: AsSlice {
    /// Returns a [`SliceMut`] containing the entire array.
    fn as_mut_slice(&mut self) -> SliceMut<'_, Self::Item>;
}
