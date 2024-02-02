// TODO: Can this be split into Slice and SliceInternal or something?

use crate::RawSoa;

pub trait Slice {
    type Raw: RawSoa;

    fn empty() -> Self;

    fn len(&self) -> usize;

    unsafe fn set_len(&mut self, length: usize);

    fn raw(&self) -> Self::Raw;
}
