use crate::SoaRaw;

pub trait SoaArray {
    type Raw: SoaRaw;

    fn as_raw(&self) -> Self::Raw;
}
