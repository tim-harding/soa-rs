use crate::{
    Slice, SliceMut, SoaRaw, Soars,
    iter_raw::{IterRaw, IterRawAdapter, iter_with_raw},
};
use std::{
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
    marker::PhantomData,
};

/// Mutable [`Slice`] iterator.
///
/// This struct is created by the [`iter_mut`] method.
///
/// [`Slice`]: crate::Slice
/// [`iter_mut`]: crate::Slice::iter_mut
pub struct IterMut<'a, T>
where
    T: 'a + Soars,
{
    pub(crate) iter_raw: IterRaw<T, Self>,
    pub(crate) _marker: PhantomData<&'a mut T>,
}

impl<T> Debug for IterMut<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}

impl<T> Default for IterMut<'_, T>
where
    T: Soars,
{
    fn default() -> Self {
        Self {
            iter_raw: IterRaw {
                slice: Slice::empty(),
                len: 0,
                adapter: PhantomData,
            },
            _marker: PhantomData,
        }
    }
}

impl<'a, T> IterRawAdapter<T> for IterMut<'a, T>
where
    T: Soars,
{
    type Item = T::RefMut<'a>;

    unsafe fn item_from_raw(raw: <T as Soars>::Raw) -> Self::Item {
        unsafe { raw.get_mut() }
    }
}

impl<'a, T> IterMut<'a, T>
where
    T: Soars,
{
    /// Returns an immutable slice of all elements that have not been yielded
    /// yet.
    pub fn as_slice(&self) -> &Slice<T> {
        unsafe { self.iter_raw.slice.as_unsized(self.iter_raw.len) }
    }

    /// Returns a mutable slice of all elements that have not been yielded yet.
    pub fn as_mut_slice(&mut self) -> &mut Slice<T> {
        unsafe { self.iter_raw.slice.as_unsized_mut(self.iter_raw.len) }
    }

    /// Returns a mutable slice of all elements that have not been yielded yet.
    ///
    /// To avoid creating `&mut` references that alias, this is forced to
    /// consume the iterator.
    pub fn into_slice(self) -> SliceMut<'a, T> {
        SliceMut {
            slice: self.iter_raw.slice,
            len: self.iter_raw.len,
            marker: PhantomData,
        }
    }
}

iter_with_raw!(IterMut<'a, T>, 'a);
