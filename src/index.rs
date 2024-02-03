use crate::slice::Slice;
use soapy_shared::{SoaRaw, Soapy};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// A helper trait for indexing operations.
pub trait SoaIndex<T>
where
    T: Soapy,
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
    T: Soapy,
{
    type Output<'a> = T::Ref<'a> where T: 'a;

    type OutputMut<'a> = T::RefMut<'a>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        if self < slice.len {
            Some(unsafe { slice.raw.get_ref(self) })
        } else {
            None
        }
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        if self < slice.len {
            Some(unsafe { slice.raw.get_mut(self) })
        } else {
            None
        }
    }
}

impl<T> SoaIndex<T> for Range<usize>
where
    T: Soapy,
{
    type Output<'a> = T::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = T::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        if self.start <= slice.len && self.end <= slice.len {
            Some(unsafe { slice.raw.slices(self.start, self.end) })
        } else {
            None
        }
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        if self.start <= slice.len && self.end <= slice.len {
            Some(unsafe { slice.raw.slices_mut(self.start, self.end) })
        } else {
            None
        }
    }
}

impl<T> SoaIndex<T> for RangeFrom<usize>
where
    T: Soapy,
{
    type Output<'a> = T::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = T::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        (self.start..slice.len).get(slice)
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        (self.start..slice.len).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeFull
where
    T: Soapy,
{
    type Output<'a> = T::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = T::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        (0..slice.len).get(slice)
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        (0..slice.len).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeInclusive<usize>
where
    T: Soapy,
{
    type Output<'a> = T::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = T::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        (*self.start()..*self.end() + 1).get(slice)
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        (*self.start()..*self.end() + 1).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeTo<usize>
where
    T: Soapy,
{
    type Output<'a> = T::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = T::SlicesMut<'a>
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
    T: Soapy,
{
    type Output<'a> = T::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = T::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        (0..self.end + 1).get(slice)
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        (0..self.end + 1).get_mut(slice)
    }
}
