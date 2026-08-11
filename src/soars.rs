use core::ptr::NonNull;

use crate::{AsSoaRef, SoaDeref, SoaRaw};

#[diagnostic::on_unimplemented(
    label = "SOA type",
    note = "The Soars trait is required for SOA compatibility"
)]
/// Provides [`Soa`] compatibility.
///
/// # Safety
///
/// [`Soars::Deref`] must be `#[repr(transparent)]` with [`Slice<Self::Raw>`].
/// This trait should be derived using the derive macro.
///
/// [`Slice<Self::Raw>`]: crate::Slice
/// [`Soa`]: crate::Soa
pub unsafe trait Soars: AsSoaRef<Item = Self> {
    /// Implements internal, unsafe, low-level routines used by [`Soa`]
    ///
    /// [`Soa`]: crate::Soa
    type Raw: SoaRaw<Item = Self>;

    /// [`Slice`] dereferences to this type to provide getters for the individual
    /// fields as slices.
    ///
    /// [`Slice`]: crate::Slice
    type Deref: ?Sized + SoaDeref<Item = Self>;

    /// A reference to an element of a [`Slice`].
    ///
    /// For each field with type `T`, this type has a field with type `&T`.
    ///
    /// [`Slice`]: crate::Slice
    type Ref<'a>: AsSoaRef<Item = Self>
    where
        Self: 'a;

    /// A reference to an element of a [`Slice`].
    ///
    /// For each field with type `T`, this type has a field with type `&mut T`.
    ///
    /// [`Slice`]: crate::Slice
    type RefMut<'a>: AsSoaRef<Item = Self>
    where
        Self: 'a;

    /// The slices that make up a [`Slice`].
    ///
    /// For each field with type `T`, this type has a field with type `&[T]`.
    ///
    /// [`Slice`]: crate::Slice
    type Slices<'a>
    where
        Self: 'a;

    /// The mutable slices that make up a [`Slice`].
    ///
    /// For each field with type `T`, this type has a field with type `&mut [T]`.
    ///
    /// [`Slice`]: crate::Slice
    type SlicesMut<'a>
    where
        Self: 'a;
}

#[diagnostic::on_unimplemented(
    label = "SOA packed type",
    note = "Enable with #[soa_derive(include(RefPacked))]"
)]
/// Enables compact packed shared element references.
///
/// Implemented automatically by the [`Soars`](crate::Soars) derive when
/// `#[soa_derive(include(RefPacked))]` is present. Required by
/// [`Slice::get_packed`](crate::Slice::get_packed).
///
/// # Safety
///
/// [`SoarsPacked::get_ref_packed`] must uphold the same validity requirements
/// documented on that method.
pub unsafe trait SoarsPacked: Soars {
    /// A compact reference to an element of a [`Slice`](crate::Slice).
    ///
    /// Stores a pointer to the SoA raw storage and a pre-validated index.
    /// Size is always 2 × pointer regardless of field count.
    /// Fields are accessed via methods that perform the pointer arithmetic
    /// on demand rather than materializing all field references at once.
    type RefPacked<'a>: Copy
    where
        Self: 'a;

    /// Creates a compact packed reference to the element at `idx`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    /// - `base` points to a valid, initialized [`Soars::Raw`] for lifetime `'a`
    /// - `idx` is a valid index (< length) and was validated before this call
    unsafe fn get_ref_packed<'a>(base: NonNull<Self::Raw>, idx: usize) -> Self::RefPacked<'a>;
}

#[diagnostic::on_unimplemented(
    label = "SOA packed mut type",
    note = "Enable with #[soa_derive(include(RefMutPacked))]"
)]
/// Enables compact packed mutable element references.
///
/// Implemented automatically by the [`Soars`](crate::Soars) derive when
/// `#[soa_derive(include(RefMutPacked))]` is present. Required by
/// [`Slice::get_packed_mut`](crate::Slice::get_packed_mut).
///
/// # Safety
///
/// [`SoarsMutPacked::get_ref_mut_packed`] must uphold the same validity
/// requirements documented on that method.
pub unsafe trait SoarsMutPacked: Soars {
    /// A compact mutable reference to an element of a [`Slice`](crate::Slice).
    ///
    /// Mutable counterpart of [`SoarsPacked::RefPacked`]. Not [`Copy`].
    /// Field accessors take `&mut self` to prevent simultaneous aliased
    /// mutable borrows.
    type RefMutPacked<'a>
    where
        Self: 'a;

    /// Creates a compact packed mutable reference to the element at `idx`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    /// - `base` points to a valid, initialized [`Soars::Raw`] for lifetime `'a`
    /// - `idx` is a valid index (< length) and was validated before this call
    /// - no other references to the same element exist
    unsafe fn get_ref_mut_packed<'a>(
        base: NonNull<Self::Raw>,
        idx: usize,
    ) -> Self::RefMutPacked<'a>;
}
