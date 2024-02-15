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
#[repr(transparent)]
pub struct SliceMut<'a, T>(pub(crate) Slice<T>, pub(crate) PhantomData<&'a mut T>)
where
    T: 'a + Soapy;

impl<'a, T> AsRef<Slice<T>> for SliceMut<'a, T>
where
    T: Soapy,
{
    fn as_ref(&self) -> &Slice<T> {
        &self.0
    }
}

impl<'a, T> AsMut<Slice<T>> for SliceMut<'a, T>
where
    T: Soapy,
{
    fn as_mut(&mut self) -> &mut Slice<T> {
        &mut self.0
    }
}

impl<'a, T> Deref for SliceMut<'a, T>
where
    T: Soapy,
{
    type Target = Slice<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for SliceMut<'a, T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
                slice: Slice {
                    raw: self.raw,
                    len: self.len,
                },
                adapter: PhantomData,
            },
            _marker: PhantomData,
        }
    }
}

eq_impl::impl_for!(SliceMut<'a, T>);

impl<'a, T> Debug for SliceMut<'a, T>
where
    T: Soapy + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a, T> PartialOrd for SliceMut<'a, T>
where
    T: Soapy + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'a, T> Ord for SliceMut<'a, T>
where
    T: Soapy + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<'a, T> Hash for SliceMut<'a, T>
where
    T: Soapy + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
