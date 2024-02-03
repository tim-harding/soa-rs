use crate::slice_raw::SliceRaw;
use soapy_shared::Soapy;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub struct SliceMut<'a, T: 'a>(SliceRaw<T>, PhantomData<&'a mut T>)
where
    T: Soapy;

impl<'a, T: 'a> Deref for SliceMut<'a, T>
where
    T: Soapy,
{
    type Target = SliceRaw<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: 'a> DerefMut for SliceMut<'a, T>
where
    T: Soapy,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
