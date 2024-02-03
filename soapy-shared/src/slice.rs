use crate::SoaRaw;

pub struct SliceRaw<T>
where
    T: SoaRaw,
{
    pub len: usize,
    pub raw: T,
}
