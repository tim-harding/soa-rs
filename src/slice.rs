use crate::slice_raw::SliceRaw;
use soapy_shared::Soapy;
use std::{marker::PhantomData, ops::Deref};

#[derive(Copy, Clone)]
pub struct Slice<'a, T: 'a>(SliceRaw<T>, PhantomData<&'a T>)
where
    T: Soapy;

impl<'a, T: 'a> Deref for Slice<'a, T>
where
    T: Soapy,
{
    type Target = SliceRaw<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
