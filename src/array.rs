use crate::SoaRaw;

// TODO: AsSlice becomes possible if soapy_shared is merged with soapy

pub trait Array {
    type Raw: SoaRaw;

    unsafe fn as_raw(&self) -> Self::Raw;
}
