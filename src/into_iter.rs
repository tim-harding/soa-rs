use crate::{
    iter_raw::{iter_with_raw, IterRaw, IterRawAdapter},
    Slice, Soa, SoaRaw, Soapy,
};
use std::{fmt::Debug, iter::FusedIterator, mem::size_of};

/// An iterator that moves out of a [`Soa`].
///
/// This struct is created by the [`into_iter`] method, provided by the
/// [`IntoIterator`] trait.
///
/// [`Soa`]: crate::Soa
/// [`into_iter`]: crate::Soa::into_iter
pub struct IntoIter<T>
where
    T: Soapy,
{
    pub(crate) iter_raw: IterRaw<T, Self>,
    pub(crate) ptr: *mut u8,
    pub(crate) cap: usize,
}

impl<T> IterRawAdapter<T> for IntoIter<T>
where
    T: Soapy,
{
    type Item = T;

    fn item_from_raw(raw: T::Raw) -> Self::Item {
        unsafe { raw.get() }
    }
}

impl<T> Default for IntoIter<T>
where
    T: Soapy,
{
    fn default() -> Self {
        Soa::<T>::new().into_iter()
    }
}

impl<T> Debug for IntoIter<T>
where
    T: Soapy + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}

impl<T> IntoIter<T>
where
    T: Soapy,
{
    /// Returns an immutable slice of all elements that have not been yielded
    /// yet.
    pub fn as_slice(&self) -> &Slice<T> {
        &self.iter_raw.slice
    }

    /// Returns a mutable slice of all elements that have not been yielded yet.
    pub fn as_mut_slice(&mut self) -> &mut Slice<T> {
        &mut self.iter_raw.slice
    }
}

impl<T> Drop for IntoIter<T>
where
    T: Soapy,
{
    fn drop(&mut self) {
        for _ in self.by_ref() {}
        if size_of::<T>() > 0 && self.cap > 0 {
            unsafe { <T::Raw as SoaRaw>::from_parts(self.ptr, self.cap).dealloc(self.cap) }
        }
    }
}

iter_with_raw!(IntoIter<T>);
