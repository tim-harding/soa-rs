use crate::slice_raw::SliceRaw;
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
    fn get(self, slice: &SliceRaw<T>) -> Option<Self::Output<'_>>;

    /// Returns the mutable output at this location, if in bounds.
    fn get_mut(self, slice: &mut SliceRaw<T>) -> Option<Self::OutputMut<'_>>;
}

impl<T> SoaIndex<T> for usize
where
    T: Soapy,
{
    type Output<'a> = T::Ref<'a> where T: 'a;

    type OutputMut<'a> = T::RefMut<'a>
    where
        T: 'a;

    fn get(self, slice: &SliceRaw<T>) -> Option<Self::Output<'_>> {
        if self < slice.0.len {
            Some(unsafe { slice.0.raw.get_ref(self) })
        } else {
            None
        }
    }

    fn get_mut(self, slice: &mut SliceRaw<T>) -> Option<Self::OutputMut<'_>> {
        if self < slice.0.len {
            Some(unsafe { slice.0.raw.get_mut(self) })
        } else {
            None
        }
    }
}

// TODO: Add back the impls for the range types
