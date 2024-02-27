use crate::{Slice, SliceMut, SliceRef, SoaRaw, Soars};
use std::{
    marker::PhantomData,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

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
    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>>;

    /// Returns the mutable output at this location, if in bounds.
    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>>;
}

impl<T> SoaIndex<T> for usize
where
    T: Soars,
{
    type Output<'a> = T::Ref<'a> where T: 'a;

    type OutputMut<'a> = T::RefMut<'a>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        if self < slice.len() {
            Some(unsafe { slice.raw().offset(self).get_ref() })
        } else {
            None
        }
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        if self < slice.len() {
            Some(unsafe { slice.raw().offset(self).get_mut() })
        } else {
            None
        }
    }
}

impl<T> SoaIndex<T> for RangeFull
where
    T: Soars,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        Some(SliceRef {
            slice: unsafe { slice.as_sized() },
            len: slice.len(),
            marker: PhantomData,
        })
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        Some(SliceMut {
            slice: unsafe { slice.as_sized() },
            len: slice.len(),
            marker: PhantomData,
        })
    }
}

impl<T> SoaIndex<T> for Range<usize>
where
    T: Soars,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        let len = self.len();
        (len + self.start <= slice.len()).then(|| SliceRef {
            slice: Slice::with_raw(unsafe { slice.raw().offset(self.start) }),
            len,
            marker: PhantomData,
        })
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        self.get(slice).map(|s| SliceMut {
            slice: unsafe { s.as_sized() },
            len: s.len(),
            marker: PhantomData,
        })
    }
}

impl<T> SoaIndex<T> for RangeTo<usize>
where
    T: Soars,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        (0..self.end).get(slice)
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        (0..self.end).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeToInclusive<usize>
where
    T: Soars,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        (0..self.end + 1).get(slice)
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        (0..self.end + 1).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeFrom<usize>
where
    T: Soars,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        (self.start..slice.len()).get(slice)
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        (self.start..slice.len()).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeInclusive<usize>
where
    T: Soars,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        (*self.start()..*self.end() + 1).get(slice)
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        (*self.start()..*self.end() + 1).get_mut(slice)
    }
}
