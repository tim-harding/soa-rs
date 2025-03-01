use crate::{
    Slice, SoaRaw, Soars,
    iter_raw::{IterRaw, IterRawAdapter, iter_with_raw},
};
use std::{
    fmt::{self, Debug, Formatter},
    iter::FusedIterator,
    marker::PhantomData,
};

/// Immutable [`Slice`] iterator.
///
/// This struct is created by the [`iter`] method.
///
/// [`Slice`]: crate::Slice
/// [`iter`]: crate::Slice::iter
pub struct Iter<'a, T>
where
    T: 'a + Soars,
{
    pub(crate) iter_raw: IterRaw<T, Self>,
    pub(crate) _marker: PhantomData<&'a T>,
}

impl<T> Debug for Iter<'_, T>
where
    T: Soars,
    for<'b> T::Ref<'b>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}

impl<T> Default for Iter<'_, T>
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

impl<T> Clone for Iter<'_, T>
where
    T: Soars,
{
    fn clone(&self) -> Self {
        Self {
            iter_raw: self.iter_raw,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iter<'a, T>
where
    T: Soars,
{
    /// Returns an immutable slice of all elements that have not been yielded
    /// yet.
    pub fn as_slice(&self) -> &'a Slice<T> {
        // SAFETY: The returned lifetime is bound to Self
        unsafe { self.iter_raw.as_slice() }
    }
}

impl<'a, T> IterRawAdapter<T> for Iter<'a, T>
where
    T: Soars,
{
    type Item = T::Ref<'a>;

    unsafe fn item_from_raw(raw: T::Raw) -> Self::Item {
        unsafe { raw.get_ref() }
    }
}

iter_with_raw!(Iter<'a, T>, 'a);
