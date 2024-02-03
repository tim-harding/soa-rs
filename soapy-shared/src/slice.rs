use crate::RawSoa;

pub struct SliceRaw<T>
where
    T: RawSoa,
{
    pub len: usize,
    pub raw: T,
}
