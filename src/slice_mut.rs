use crate::{eq_impl, iter_raw::IterRaw, IterMut, Slice, SliceRef, Soa, Soars};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// An mutably borrowed [`Slice`].
///
/// A `SliceMut` is a thin wrapper over a [`Slice`] that applies the same
/// borrowing rules as a mutable reference. It is semantically equivalent to
/// `&mut Slice`.
pub struct SliceMut<'a, T>
where
    T: 'a + Soars,
{
    pub(crate) slice: Slice<T, ()>,
    pub(crate) len: usize,
    pub(crate) marker: PhantomData<&'a mut T>,
}

impl<'a, T> SliceMut<'a, T>
where
    T: Soars,
{
    /// Creates a new [`SliceMut`] from the given slice and length. Intended for
    /// use in proc macro code, not user code.
    ///
    /// # Safety
    ///
    /// The provided slice and its length must be compatible. Since the slice
    /// passed in has no intrinsic lifetime, care must be taken to ensure that
    /// the lifetime of [`SliceMut`] is valid.
    #[doc(hidden)]
    pub unsafe fn from_slice(slice: Slice<T, ()>, len: usize) -> Self {
        Self {
            slice,
            len,
            marker: PhantomData,
        }
    }
}

impl<'a, T> AsRef<Slice<T>> for SliceMut<'a, T>
where
    T: Soars,
{
    fn as_ref(&self) -> &Slice<T> {
        self.deref()
    }
}

impl<'a, T> AsMut<Slice<T>> for SliceMut<'a, T>
where
    T: Soars,
{
    fn as_mut(&mut self) -> &mut Slice<T> {
        self.deref_mut()
    }
}

impl<'a, T> Deref for SliceMut<'a, T>
where
    T: Soars,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.slice.as_unsized(self.len) }
    }
}

impl<'a, T> DerefMut for SliceMut<'a, T>
where
    T: Soars,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.slice.as_unsized_mut(self.len) }
    }
}

impl<'a, T> IntoIterator for SliceMut<'a, T>
where
    T: Soars,
{
    type Item = T::RefMut<'a>;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter_raw: IterRaw {
                slice: Slice::with_raw(self.raw()),
                len: self.len(),
                adapter: PhantomData,
            },
            _marker: PhantomData,
        }
    }
}

eq_impl::impl_for!(SliceMut<'_, T>);

impl<'a, T> Debug for SliceMut<'a, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a, T> PartialOrd for SliceMut<'a, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<'a, T> Ord for SliceMut<'a, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<'a, T> Hash for SliceMut<'a, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}
