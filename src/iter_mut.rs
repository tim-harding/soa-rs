use crate::{
    iter_raw::{iter_with_raw, IterRaw, IterRawAdapter},
    soa_ref::RefMut,
    Slice, SoaRaw, Soapy,
};
use std::{iter::FusedIterator, marker::PhantomData};

/// Mutable [`Slice`] iterator.
///
/// This struct is created by the [`iter_mut`] method.
///
/// [`Slice`]: crate::Slice
/// [`iter_mut`]: crate::Slice::iter_mut
pub struct IterMut<'a, T>
where
    T: 'a + Soapy,
{
    pub(crate) iter_raw: IterRaw<T, Self>,
    pub(crate) _marker: PhantomData<&'a mut T>,
}

impl<'a, T> IterRawAdapter<T> for IterMut<'a, T>
where
    T: 'a + Soapy,
{
    type Item = RefMut<'a, T>;

    fn item_from_raw(raw: <T as Soapy>::Raw) -> Self::Item {
        RefMut(unsafe { raw.get_mut() })
    }
}

impl<'a, T> IterMut<'a, T>
where
    T: 'a + Soapy,
{
    /// Returns an immutable slice of all elements that have not been yielded
    /// yet.
    pub fn as_slice(&self) -> &Slice<T> {
        self.iter_raw.as_slice()
    }

    /// Returns a mutable slice of all elements that have not been yielded yet.
    pub fn as_mut_slice(&mut self) -> &mut Slice<T> {
        self.iter_raw.as_mut_slice()
    }
}

iter_with_raw!(IterMut<'a, T>, 'a);
