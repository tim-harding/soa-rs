use crate::{SoaRaw, Soapy};
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
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: 'a + Soapy,
{
    type Item = T::Ref<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let out = unsafe { self.raw.get_ref(self.start) };
            self.start += 1;
            Some(out)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }

    // TODO: Nightly-only try_fold implementation
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: 'a + Soapy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            self.end -= 1;
            Some(unsafe { self.raw.get_ref(self.end) })
        }
    }
}
