use crate::{SliceMut, SliceRef, Soapy};

pub trait SoaArray {
    type Item: Soapy;

    fn as_slice(&self) -> SliceRef<'_, Self::Item>;

    fn as_mut_slice(&mut self) -> SliceMut<'_, Self::Item>;
}
