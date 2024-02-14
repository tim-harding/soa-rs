use crate::{
    iter_raw::{iter_with_raw, IterRaw, IterRawAdapter},
    Ref, SliceRef, SoaRaw, Soapy,
};
use std::{iter::FusedIterator, marker::PhantomData};

/// Immutable [`Slice`] iterator.
///
/// This struct is created by the [`iter`] method.
///
/// [`Slice`]: crate::Slice
/// [`iter`]: crate::Slice::iter
pub struct Iter<'a, T>
where
    T: 'a + Soapy,
{
    pub(crate) iter_raw: IterRaw<T, Self>,
    pub(crate) _marker: PhantomData<&'a T>,
}

impl<'a, T> Iter<'a, T>
where
    T: 'a + Soapy,
{
    /// Returns an immutable slice of all elements that have not been yielded
    /// yet.
    pub fn as_slice(&self) -> SliceRef<'_, T> {
        self.iter_raw.as_slice()
    }
}

impl<'a, T> IterRawAdapter<T> for Iter<'a, T>
where
    T: Soapy,
{
    type Item = Ref<'a, T>;

    fn item_from_raw(raw: T::Raw) -> Self::Item {
        Ref(unsafe { raw.get_ref() })
    }
}

iter_with_raw!(Iter<'a, T>, 'a);
