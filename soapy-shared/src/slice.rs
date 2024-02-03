// TODO: Can this be split into Slice and SliceInternal or something?

use crate::RawSoa;

pub trait Slice {
    type Raw: RawSoa;
    type Deref;

    fn from_raw_parts(raw: Self::Raw, length: usize) -> Self;

    fn as_deref(&self) -> &Self::Deref;

    fn len(&self) -> usize;

    fn len_mut(&mut self) -> &mut usize;

    fn raw(&self) -> Self::Raw;

    fn raw_mut(&mut self) -> &mut Self::Raw;
}
