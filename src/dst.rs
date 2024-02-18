use std::ops::{Deref, DerefMut};

/// A dependently-sized type. This can be used to treat the same type `T` as
/// sized or unsized depending on the generic parameter `D`. In particular, this
/// is useful when exposing `&mut` references to internal data to prevent
/// unsoundness due to [`swap`].
///
/// [`swap`]: std::mem::swap
pub struct Dst<T, D: ?Sized = [()]>(pub(crate) T, pub(crate) D);

impl<T> Dst<T, ()> {
    /// Converts to an unsized variant.
    ///
    /// # Safety
    ///
    /// - `length` must be valid for the underlying type `T`.
    /// - The lifetime of the returned reference is unconstrained. Ensure that
    /// the right lifetimes are applied.
    pub(crate) unsafe fn as_unsized<'a>(&self, len: usize) -> &'a Dst<T, [()]> {
        &*(std::ptr::slice_from_raw_parts(self, len) as *const Dst<T>)
    }

    /// Converts to an mutable unsized variant.
    ///
    /// # Safety
    ///
    /// - `length` must be valid for the underlying type `T`.
    /// - The lifetime of the returned reference is unconstrained. Ensure that
    /// the right lifetimes are applied.
    pub(crate) unsafe fn as_unsized_mut<'a>(&mut self, len: usize) -> &'a mut Dst<T, [()]> {
        &mut *(std::ptr::slice_from_raw_parts_mut(self, len) as *mut Dst<T>)
    }
}

impl<T> Dst<T, [()]> {
    /// Converts from an unsized variant to sized variant
    ///
    /// # Safety
    ///
    /// Since this returns an owned value, it implicitly extends the lifetime &
    /// in an unbounded way. The caller must ensure proper lifetimes with, for
    /// example, [`PhantomData`].
    ///
    /// [`PhantomData`]: std::marker::PhantomData
    pub(crate) const unsafe fn as_sized(&self) -> Dst<T, ()>
    where
        T: Copy,
    {
        let ptr = self as *const _ as *const Dst<T, ()>;
        unsafe { *ptr }
    }

    pub(crate) const fn len(&self) -> usize {
        self.1.len()
    }
}

unsafe impl<T, D: ?Sized> Send for Dst<T, D> where T: Send {}
unsafe impl<T, D: ?Sized> Sync for Dst<T, D> where T: Sync {}

impl<T> Clone for Dst<T, ()>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), ())
    }
}

impl<T> Copy for Dst<T, ()> where T: Copy {}

impl<T, D> Deref for Dst<T, D> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, D> DerefMut for Dst<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
