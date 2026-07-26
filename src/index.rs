use crate::{Slice, SliceMut, SliceRef, SoaRaw, Soars};
use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// A helper trait for indexing operations.
pub trait SoaIndex<T>
where
    T: Soars,
{
    /// The output type returned by non-`mut` methods.
    type Output<'a>
    where
        T: 'a;

    /// The output type returned by `mut` methods.
    type OutputMut<'a>
    where
        T: 'a;

    /// Returns the output at this location, if in bounds.
    fn get<'a>(self, slice: SliceRef<'a, T>) -> Option<Self::Output<'a>>;

    /// Returns the mutable output at this location, if in bounds.
    fn get_mut<'a>(self, slice: SliceMut<'a, T>) -> Option<Self::OutputMut<'a>>;
}

impl<T> SoaIndex<T> for usize
where
    T: Soars,
{
    type Output<'a>
        = T::Ref<'a>
    where
        T: 'a;

    type OutputMut<'a>
        = T::RefMut<'a>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slice: SliceRef<'a, T>) -> Option<Self::Output<'a>> {
        // SAFETY: Offsetting by less than the length leaves
        // at least one item to read
        (self < slice.len()).then(|| unsafe { slice.raw().offset(self).get_ref() })
    }

    #[inline]
    fn get_mut<'a>(self, slice: SliceMut<'a, T>) -> Option<Self::OutputMut<'a>> {
        // SAFETY: Offsetting by less than the length leaves
        // at least one item to read
        (self < slice.len()).then(|| unsafe { slice.raw().offset(self).get_mut() })
    }
}

impl<T> SoaIndex<T> for RangeFull
where
    T: Soars,
{
    type Output<'a>
        = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a>
        = SliceMut<'a, T>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slice: SliceRef<'a, T>) -> Option<Self::Output<'a>> {
        Some(slice)
    }

    #[inline]
    fn get_mut<'a>(self, slice: SliceMut<'a, T>) -> Option<Self::OutputMut<'a>> {
        Some(slice)
    }
}

impl<T> SoaIndex<T> for Range<usize>
where
    T: Soars,
{
    type Output<'a>
        = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a>
        = SliceMut<'a, T>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slice: SliceRef<'a, T>) -> Option<Self::Output<'a>> {
        let len = self.len();
        (len + self.start <= slice.len()).then(|| {
            // SAFETY:
            // - The above bounds check ensures we won't be able to access the
            //   slice out of bounds.
            // - The returned lifetime is that of the given slice.
            let raw = unsafe { slice.raw().offset(self.start) };
            unsafe { SliceRef::from_slice(Slice::with_raw(raw), len) }
        })
    }

    #[inline]
    fn get_mut<'a>(self, slice: SliceMut<'a, T>) -> Option<Self::OutputMut<'a>> {
        let len = self.len();
        (len + self.start <= slice.len()).then(|| {
            // SAFETY:
            // - The above bounds check ensures we won't be able to access the
            //   slice out of bounds.
            // - The returned lifetime is that of the given slice, which is
            //   consumed to prevent aliasing.
            let raw = unsafe { slice.raw().offset(self.start) };
            unsafe { SliceMut::from_slice(Slice::with_raw(raw), len) }
        })
    }
}

impl<T> SoaIndex<T> for RangeTo<usize>
where
    T: Soars,
{
    type Output<'a>
        = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a>
        = SliceMut<'a, T>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slice: SliceRef<'a, T>) -> Option<Self::Output<'a>> {
        (0..self.end).get(slice)
    }

    #[inline]
    fn get_mut<'a>(self, slice: SliceMut<'a, T>) -> Option<Self::OutputMut<'a>> {
        (0..self.end).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeToInclusive<usize>
where
    T: Soars,
{
    type Output<'a>
        = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a>
        = SliceMut<'a, T>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slice: SliceRef<'a, T>) -> Option<Self::Output<'a>> {
        (0..self.end + 1).get(slice)
    }

    #[inline]
    fn get_mut<'a>(self, slice: SliceMut<'a, T>) -> Option<Self::OutputMut<'a>> {
        (0..self.end + 1).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeFrom<usize>
where
    T: Soars,
{
    type Output<'a>
        = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a>
        = SliceMut<'a, T>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slice: SliceRef<'a, T>) -> Option<Self::Output<'a>> {
        (self.start..slice.len()).get(slice)
    }

    #[inline]
    fn get_mut<'a>(self, slice: SliceMut<'a, T>) -> Option<Self::OutputMut<'a>> {
        (self.start..slice.len()).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeInclusive<usize>
where
    T: Soars,
{
    type Output<'a>
        = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a>
        = SliceMut<'a, T>
    where
        T: 'a;

    #[inline]
    fn get<'a>(self, slice: SliceRef<'a, T>) -> Option<Self::Output<'a>> {
        (*self.start()..*self.end() + 1).get(slice)
    }

    #[inline]
    fn get_mut<'a>(self, slice: SliceMut<'a, T>) -> Option<Self::OutputMut<'a>> {
        (*self.start()..*self.end() + 1).get_mut(slice)
    }
}
