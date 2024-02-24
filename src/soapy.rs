use crate::{SoaArray, SoaDeref, SoaRaw, WithRef};

/// Provides [`Soa`] compatibility.
///
/// # Safety
///
/// [`Soapy::Deref`] mut be `#[repr(transparent)]` with [`Slice<Self::Raw>`].
/// This trait should be derived using the derive macro.
///
/// [`Slice<Self::Raw>`]: crate::Slice
/// [`Soa`]: crate::Soa
pub unsafe trait Soapy: WithRef {
    /// Implements internal, unsafe, low-level routines used by [`Soa`]
    ///
    /// [`Soa`]: crate::Soa
    type Raw: SoaRaw<Item = Self>;

    /// [`Slice`] dereferences to this type to provide getters for the individual
    /// fields as slices.
    ///
    /// [`Slice`]: crate::Slice
    type Deref: ?Sized + SoaDeref<Item = Self>;

    /// For each field with type `F` in `T`, `Ref` has a field with type
    /// `&F`
    type Ref<'a>: WithRef<Item = Self>
    where
        Self: 'a;

    /// For each field with type `F` in `T`, `RefMut` has a field with type
    /// `&mut F`
    type RefMut<'a>: WithRef<Item = Self>
    where
        Self: 'a;

    type Array<const N: usize>: SoaArray<Item = Self>;

    type Slices<'a>
    where
        Self: 'a;

    type SlicesMut<'a>
    where
        Self: 'a;
}
