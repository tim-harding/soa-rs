use crate::{SliceRef, Soapy};

pub trait SoaArray {
    type Item: Soapy;

    fn as_slice(&self) -> SliceRef<'_, Self::Item>;

    fn as_slices(&self) -> <Self::Item as Soapy>::Slices<'_>;

    fn as_mut_slices(&mut self) -> <Self::Item as Soapy>::SlicesMut<'_>;
}
