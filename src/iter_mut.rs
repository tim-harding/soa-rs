use crate::{soa_ref::RefMut, SoaRaw, Soapy};
use std::marker::PhantomData;

/// Mutable [`Soa`] iterator.
///
/// This struct is created by the [`iter_mut`] method.
///
/// [`Soa`]: crate::Soa
/// [`iter_mut`]: crate::Soa::iter_mut
pub struct IterMut<'a, T>
where
    T: 'a + Soapy,
{
    pub(crate) raw: T::Raw,
    pub(crate) len: usize,
    pub(crate) _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: 'a + Soapy,
{
    type Item = RefMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let out = Some(RefMut(unsafe { self.raw.get_mut() }));
            self.raw = unsafe { self.raw.offset(1) };
            out
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T>
where
    T: 'a + Soapy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(RefMut(unsafe { self.raw.offset(self.len).get_mut() }))
        }
    }
}
