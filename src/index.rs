use std::{marker::PhantomData, ops::RangeFull};

use crate::{slice_mut::SliceMut, SliceRef};
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
    fn get<'a>(self, slice: &'a SliceRef<T>) -> Option<Self::Output<'a>>;

    /// Returns the mutable output at this location, if in bounds.
    fn get_mut<'a>(self, slice: &'a mut SliceMut<T>) -> Option<Self::OutputMut<'a>>;
}

impl<T> SoaIndex<T> for usize
where
    T: Soapy,
{
    type Output<'a> = T::Ref<'a> where T: 'a;

    type OutputMut<'a> = T::RefMut<'a>
    where
        T: 'a;

    fn get<'a>(self, slice: &'a SliceRef<T>) -> Option<Self::Output<'a>> {
        if self < slice.len() {
            Some(unsafe { slice.0 .0.raw.get_ref(self) })
        } else {
            None
        }
    }

    fn get_mut<'a>(self, slice: &'a mut SliceMut<T>) -> Option<Self::OutputMut<'a>> {
        if self < slice.len() {
            Some(unsafe { slice.0 .0.raw.get_mut(self) })
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

    fn get<'a>(self, slice: &'a SliceRef<T>) -> Option<Self::Output<'a>> {
        Some(*slice)
    }

    fn get_mut<'a>(self, slice: &'a mut SliceMut<T>) -> Option<Self::OutputMut<'a>> {
        // TODO: Verify that the input slice cannot be accessed while this value lives
        Some(SliceMut(slice.0, PhantomData))
    }
}
