use soapy_shared::{RawSoa, Soapy};
use std::marker::PhantomData;

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
    pub(crate) raw: T::RawSoa,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: Soapy,
{
    type Item = T::RefMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let out = unsafe { self.raw.get_mut(self.start) };
            self.start += 1;
            Some(out)
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
            Some(unsafe { self.raw.get_mut(self.end) })
        }
    }
}
