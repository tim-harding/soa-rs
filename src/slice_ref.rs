use crate::{iter_raw::IterRaw, AsSlice, Iter, Slice, Soars};
use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

/// An immutably borrowed [`Slice`].
///
/// A `SliceRef` is a thin wrapper over a [`Slice`] that applies the same
/// borrowing rules as an immutable reference. It is semantically equivalent to
/// `&Slice`.
pub struct SliceRef<'a, T>
where
    T: 'a + Soars,
{
    pub(crate) slice: Slice<T, ()>,
    pub(crate) len: usize,
    pub(crate) marker: PhantomData<&'a T>,
}

impl<T> SliceRef<'_, T>
where
    T: Soars,
{
    /// Creates a new [`SliceRef`] from the given slice and length. Intended for
    /// use in proc macro code, not user code.
    ///
    /// # Safety
    ///
    /// The provided slice and its length must be compatible. Since the slice
    /// passed in has no intrinsic lifetime, care must be taken to ensure that
    /// the lifetime of [`SliceRef`] is valid.
    #[doc(hidden)]
    pub unsafe fn from_slice(slice: Slice<T, ()>, len: usize) -> Self {
        Self {
            slice,
            len,
            marker: PhantomData,
        }
    }
}

impl<T> Clone for SliceRef<'_, T>
where
    T: Soars,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for SliceRef<'a, T> where T: 'a + Soars {}

impl<T> AsRef<Slice<T>> for SliceRef<'_, T>
where
    T: Soars,
{
    fn as_ref(&self) -> &Slice<T> {
        // SAFETY:
        // - len is valid
        // - The returned lifetime is bound to self
        unsafe { self.slice.as_unsized(self.len) }
    }
}

impl<T> Deref for SliceRef<'_, T>
where
    T: Soars,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, T> IntoIterator for SliceRef<'a, T>
where
    T: Soars,
{
    type Item = T::Ref<'a>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter_raw: IterRaw {
                slice: Slice::with_raw(self.raw()),
                len: self.len(),
                adapter: PhantomData,
            },
            _marker: PhantomData,
        }
    }
}

impl<T> Debug for SliceRef<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T> PartialOrd for SliceRef<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T> Ord for SliceRef<'_, T>
where
    T: Soars + Ord,
    for<'b> T::Ref<'b>: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T> Hash for SliceRef<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<T, R> PartialEq<R> for SliceRef<'_, T>
where
    T: Soars,
    R: AsSlice<Item = T> + ?Sized,
    for<'a> T::Ref<'a>: PartialEq,
{
    fn eq(&self, other: &R) -> bool {
        self.as_ref() == other.as_slice().as_ref()
    }
}

impl<T> Eq for SliceRef<'_, T>
where
    T: Soars,
    for<'a> T::Ref<'a>: Eq,
{
}

impl<T> AsSlice for SliceRef<'_, T>
where
    T: Soars,
{
    type Item = T;

    fn as_slice(&self) -> SliceRef<'_, Self::Item> {
        *self
    }
}
