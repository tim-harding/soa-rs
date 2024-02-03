use std::{
    marker::PhantomData,
    ops::{RangeFull, RangeTo},
};

use crate::{slice_mut::SliceMut, Slice, SliceRef};
use soapy_shared::{SoaRaw, Soapy};

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
            Some(unsafe { slice.0.raw.get_ref(self) })
        } else {
            None
        }
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        if self < slice.len() {
            Some(unsafe { slice.0.raw.get_mut(self) })
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
        Some(SliceMut(Slice(slice.0), PhantomData))
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
        Some(SliceRef(
            Slice(soapy_shared::SliceData {
                len: self.end,
                raw: slice.0.raw,
            }),
            PhantomData,
        ))
    }

    fn get_mut<'a>(self, slice: &'a mut Slice<T>) -> Option<Self::OutputMut<'a>> {
        Some(SliceMut(
            Slice(soapy_shared::SliceData {
                len: self.end,
                raw: slice.0.raw,
            }),
            PhantomData,
        ))
    }
}
