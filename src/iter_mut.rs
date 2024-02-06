use std::marker::PhantomData;

use crate::{soa_ref::RefMut, SoaRaw, Soapy};

/// Mutable [`Soa`] iterator.
///
/// This struct is created by the [`iter_mut`] method.
///
/// [`Soa`]: crate::Soa
/// [`iter_mut`]: crate::Soa::iter_mut
pub struct IterMut<'a, T>
where
    T: Soapy,
{
    pub(crate) raw: T::Raw,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Soapy,
{
    type Item = RefMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let out = unsafe { self.raw.get_mut(self.start) };
            self.start += 1;
            Some(RefMut(out))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: Soapy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            self.end -= 1;
            Some(RefMut(unsafe { self.raw.get_mut(self.end) }))
        }
    }
}
