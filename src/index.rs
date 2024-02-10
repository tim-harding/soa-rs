use std::{
    marker::PhantomData,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use crate::{slice_mut::SliceMut, soa_ref::RefMut, Ref, Slice, SliceRef, SoaRaw, Soapy};

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
    type Output<'a> = Ref<'a, T> where T: 'a;

    type OutputMut<'a> = RefMut<'a, T>
    where
        T: 'a;

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        if self < slice.len() {
            Some(Ref(unsafe { slice.raw.offset(self).get_ref() }))
        } else {
            None
        }
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
        if self < slice.len() {
            Some(RefMut(unsafe { slice.raw.offset(self).get_mut() }))
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

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
        Some(SliceRef(*slice, PhantomData))
    }

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
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

    fn get(self, slice: &Slice<T>) -> Option<Self::Output<'_>> {
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

    fn get_mut(self, slice: &mut Slice<T>) -> Option<Self::OutputMut<'_>> {
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
    T: Soapy,
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
    T: Soapy,
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
