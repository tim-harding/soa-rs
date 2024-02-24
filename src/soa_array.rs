use crate::{Slice, Soapy};

/// # Safety
///
/// This trait is unsafe because the soundness of the default impls of
/// `as_slice` and `as_slice_mut` depend on the correctness of the `len` method.
pub unsafe trait SoaArray {
    type Item: Soapy;

    unsafe fn as_raw(&self) -> <Self::Item as Soapy>::Raw;

    fn len(&self) -> usize;

    fn as_slice(&self) -> &Slice<Self::Item> {
        unsafe { Slice::with_raw(self.as_raw()).as_unsized(self.len()) }
    }

    fn as_mut_slice(&mut self) -> &mut Slice<Self::Item> {
        unsafe { Slice::with_raw(self.as_raw()).as_unsized_mut(self.len()) }
    }
}
