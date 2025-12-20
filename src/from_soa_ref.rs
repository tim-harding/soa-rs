use crate::AsSoaRef;

pub trait FromSoaRef {
    fn from_soa_ref<R>(item: &R) -> Self
    where
        R: AsSoaRef<Item = Self>;
}
