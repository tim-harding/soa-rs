use crate::{eq_impl, iter_raw::IterRaw, soa_ref::RefMut, IterMut, Slice, SliceRef, Soa, Soapy};
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
    T: 'a + Soapy,
{
    pub(crate) slice: Slice<T, ()>,
    pub(crate) len: usize,
    pub(crate) marker: PhantomData<&'a mut T>,
}

impl<'a, T> AsRef<Slice<T>> for SliceMut<'a, T>
where
    T: Soapy,
{
    fn as_ref(&self) -> &Slice<T> {
        self.deref()
    }
}

impl<'a, T> AsMut<Slice<T>> for SliceMut<'a, T>
where
    T: Soapy,
{
    fn as_mut(&mut self) -> &mut Slice<T> {
        self.deref_mut()
    }
}

impl<'a, T> Deref for SliceMut<'a, T>
where
    T: Soapy,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.slice.as_unsized(self.len) }
    }
}

impl<'a, T> DerefMut for SliceMut<'a, T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.slice.as_unsized_mut(self.len) }
    }
}

impl<'a, T> IntoIterator for SliceMut<'a, T>
where
    T: Soapy,
{
    type Item = RefMut<'a, T>;
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
    T: Soapy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a, T> PartialOrd for SliceMut<'a, T>
where
    T: Soapy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<'a, T> Ord for SliceMut<'a, T>
where
    T: Soapy + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<'a, T> Hash for SliceMut<'a, T>
where
    T: Soapy + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}
