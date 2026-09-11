use crate::SoarsMutPacked;
use core::{iter::FusedIterator, marker::PhantomData, ptr::NonNull};

/// Mutable packed [`Slice`] iterator.
///
/// This struct is created by the [`iter_packed_mut`] method.
///
/// [`Slice`]: crate::Slice
/// [`iter_packed_mut`]: crate::Slice::iter_packed_mut
pub struct IterPackedMut<'a, T>
where
    T: 'a + SoarsMutPacked,
{
    pub(crate) base: NonNull<T::Raw>,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) _marker: PhantomData<&'a mut T>,
}

impl<T> Default for IterPackedMut<'_, T>
where
    T: SoarsMutPacked,
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

impl<'a, T> Iterator for IterPackedMut<'a, T>
where
    T: SoarsMutPacked,
{
    type Item = T::RefMutPacked<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let idx = self.start;
            self.start += 1;
            // SAFETY: idx is in [start, end) of a slice whose raw lives for 'a;
            // exclusive access is guaranteed by &'a mut T
            Some(unsafe { T::get_ref_mut_packed(self.base, idx) })
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

impl<'a, T> DoubleEndedIterator for IterPackedMut<'a, T>
where
    T: SoarsMutPacked,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            self.end -= 1;
            // SAFETY: end was in (start, end] of a slice whose raw lives for 'a;
            // exclusive access is guaranteed by &'a mut T
            Some(unsafe { T::get_ref_mut_packed(self.base, self.end) })
        }
    }
}

impl<T> FusedIterator for IterPackedMut<'_, T> where T: SoarsMutPacked {}
impl<T> ExactSizeIterator for IterPackedMut<'_, T> where T: SoarsMutPacked {}
