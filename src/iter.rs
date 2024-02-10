use crate::{Ref, SoaRaw, Soapy};
use std::marker::PhantomData;

/// Immutable [`Soa`] iterator.
///
/// This struct is created by the [`iter`] method.
///
/// [`Soa`]: crate::Soa
/// [`iter`]: crate::Soa::iter
pub struct Iter<'a, T>
where
    T: 'a + Soapy,
{
    pub(crate) raw: T::Raw,
    pub(crate) len: usize,
    pub(crate) _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: 'a + Soapy,
{
    type Item = Ref<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let out = Some(Ref(unsafe { self.raw.get_ref() }));
            self.raw = unsafe { self.raw.offset(1) };
            out
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: 'a + Soapy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(Ref(unsafe { self.raw.offset(self.len).get_ref() }))
        }
    }
}
