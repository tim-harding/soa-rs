use std::{
    marker::PhantomData,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use crate::{slice_mut::SliceMut, Slice, SliceRef, SoaRaw, Soapy};

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
    fn get<'a>(self, slice: &'a Slice<T>) -> Option<Self::Output<'a>>;

    /// Returns the mutable output at this location, if in bounds.
    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>>;
}

impl<T> SoaIndex<T> for usize
where
    T: Soapy,
{
    type Output<'a> = T::Ref<'a> where T: 'a;

    type OutputMut<'a> = T::RefMut<'a>
    where
        T: 'a;

    fn get<'a>(self, slice: &'a Slice<T>) -> Option<Self::Output<'a>> {
        if self < slice.len() {
            Some(unsafe { slice.raw.get_ref(self) })
        } else {
            None
        }
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        if self < slice.len() {
            Some(unsafe { slice.raw.get_mut(self) })
        } else {
            None
        }
    }
}

impl<T> SoaIndex<T> for RangeFull
where
    T: Soapy,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get<'a>(self, slice: &'a Slice<T>) -> Option<Self::Output<'a>> {
        Some(SliceRef(*slice, PhantomData))
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        Some(SliceMut(*slice, PhantomData))
    }
}

impl<T> SoaIndex<T> for Range<usize>
where
    T: Soapy,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get<'a>(self, slice: &'a Slice<T>) -> Option<Self::Output<'a>> {
        let len = self.len();
        (len + self.start <= slice.len()).then(|| {
            SliceRef(
                Slice {
                    len,
                    raw: unsafe { slice.raw.offset(self.start) },
                },
                PhantomData,
            )
        })
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        self.get(slice).map(|s| SliceMut(s.0, PhantomData))
    }
}

impl<T> SoaIndex<T> for RangeTo<usize>
where
    T: Soapy,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get<'a>(self, slice: &'a Slice<T>) -> Option<Self::Output<'a>> {
        (0..self.end).get(slice)
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        (0..self.end).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeToInclusive<usize>
where
    T: Soapy,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get<'a>(self, slice: &'a Slice<T>) -> Option<Self::Output<'a>> {
        (0..self.end + 1).get(slice)
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        (0..self.end + 1).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeFrom<usize>
where
    T: Soapy,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get<'a>(self, slice: &'a Slice<T>) -> Option<Self::Output<'a>> {
        (self.start..slice.len()).get(slice)
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        (self.start..slice.len()).get_mut(slice)
    }
}

impl<T> SoaIndex<T> for RangeInclusive<usize>
where
    T: Soapy,
{
    type Output<'a> = SliceRef<'a, T>
    where
        T: 'a;

    type OutputMut<'a> = SliceMut<'a, T>
    where
        T: 'a;

    fn get<'a>(self, slice: &'a Slice<T>) -> Option<Self::Output<'a>> {
        (*self.start()..*self.end() + 1).get(slice)
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        (*self.start()..*self.end() + 1).get_mut(slice)
    }
}
