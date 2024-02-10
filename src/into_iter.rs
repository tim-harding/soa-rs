use crate::{Slice, SliceMut, SliceRef, SoaRaw, Soapy};
use std::{iter::FusedIterator, marker::PhantomData, mem::size_of};

// TODO: Nightly-only try_fold implementation

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
    pub(crate) ptr: *mut u8,
    pub(crate) raw: T::Raw,
    pub(crate) cap: usize,
    pub(crate) len: usize,
}

impl<T> IntoIter<T>
where
    T: Soapy,
{
    pub fn as_slice(&self) -> SliceRef<'_, T> {
        SliceRef(
            unsafe { Slice::from_raw_parts(self.raw, self.len) },
            PhantomData,
        )
    }

    pub fn as_mut_slice(&mut self) -> SliceMut<'_, T> {
        SliceMut(
            unsafe { Slice::from_raw_parts(self.raw, self.len) },
            PhantomData,
        )
    }
}

impl<T> Iterator for IntoIter<T>
where
    T: Soapy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let out = Some(unsafe { self.raw.get() });
            self.raw = unsafe { self.raw.offset(1) };
            out
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T>
where
    T: Soapy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.raw.offset(self.len).get() })
        }
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

impl<T> FusedIterator for IntoIter<T> where T: Soapy {}
impl<T> ExactSizeIterator for IntoIter<T> where T: Soapy {}
