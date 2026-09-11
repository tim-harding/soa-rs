use crate::SoarsPacked;
use core::{iter::FusedIterator, marker::PhantomData, ptr::NonNull};

/// Immutable packed [`Slice`] iterator.
///
/// This struct is created by the [`iter_packed`] method.
///
/// [`Slice`]: crate::Slice
/// [`iter_packed`]: crate::Slice::iter_packed
pub struct IterPacked<'a, T>
where
    T: 'a + SoarsPacked,
{
    pub(crate) base: NonNull<T::Raw>,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) _marker: PhantomData<&'a T>,
}

impl<T> Default for IterPacked<'_, T>
where
    T: SoarsPacked,
{
    fn default() -> Self {
        Self {
            base: NonNull::dangling(),
            start: 0,
            end: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for IterPacked<'_, T>
where
    T: SoarsPacked,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for IterPacked<'_, T> where T: SoarsPacked {}

impl<'a, T> Iterator for IterPacked<'a, T>
where
    T: SoarsPacked,
{
    type Item = T::RefPacked<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let idx = self.start;
            self.start += 1;
            // SAFETY: idx is in [start, end) of a slice whose raw lives for 'a
            Some(unsafe { T::get_ref_packed(self.base, idx) })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.end - self.start
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for IterPacked<'a, T>
where
    T: SoarsPacked,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            self.end -= 1;
            // SAFETY: end was in (start, end] of a slice whose raw lives for 'a
            Some(unsafe { T::get_ref_packed(self.base, self.end) })
        }
    }
}

impl<T> FusedIterator for IterPacked<'_, T> where T: SoarsPacked {}
impl<T> ExactSizeIterator for IterPacked<'_, T> where T: SoarsPacked {}
