use core::{marker::PhantomData, ptr::NonNull};

/// Opaque compact pointer: raw SoA storage plus a validated element index.
///
/// Private storage for generated `FooRefPacked` types. Fields are private to
/// this crate, so forging invalid raw pointer and an out-of-bounds index requires `unsafe`.
pub struct PackedPointer<'a, Raw> {
    raw: NonNull<Raw>,
    idx: usize,
    _marker: PhantomData<&'a Raw>,
}

impl<Raw> Copy for PackedPointer<'_, Raw> {}

impl<Raw> Clone for PackedPointer<'_, Raw> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

/// Opaque compact mutable pointer: raw SoA storage plus a validated element index.
///
/// Private storage for generated `FooRefMutPacked` types. Not [`Copy`], matching
/// `&mut`-like exclusivity. Fields are private to this crate, so forging an
/// out-of-bounds index requires `unsafe`.
pub struct PackedMutPointer<'a, Raw> {
    raw: NonNull<Raw>,
    idx: usize,
    _marker: PhantomData<&'a mut Raw>,
}

macro_rules! impl_packed_pointer {
    ($name:ident, $safety:expr) => {
        impl<'a, Raw> $name<'a, Raw> {
            /// Creates a packed pointer without checking that `raw`/`idx` are valid.
            ///
            /// # Safety
            ///
            #[doc = $safety]
            #[inline]
            pub unsafe fn new_unchecked(raw: NonNull<Raw>, idx: usize) -> Self {
                Self {
                    raw,
                    idx,
                    _marker: PhantomData,
                }
            }

            /// Returns the element index stored in this packed pointer.
            #[inline]
            pub fn index(&self) -> usize {
                self.idx
            }

            /// Returns the raw SoA storage pointer.
            #[inline]
            pub fn raw(&self) -> NonNull<Raw> {
                self.raw
            }
        }
    };
}

impl_packed_pointer!(
    PackedPointer,
    "The caller must ensure that `raw` points to valid SoA raw storage for lifetime `'a` and that `idx` is in bounds for that storage."
);
impl_packed_pointer!(
    PackedMutPointer,
    "The caller must ensure that `raw` points to valid SoA raw storage for lifetime `'a`, that `idx` is in bounds for that storage, and that no other references to the same element exist."
);
