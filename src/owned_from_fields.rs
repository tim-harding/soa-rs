use crate::AsSoaRef;

pub trait OwnedFromFields {
    fn owned_from_fields<R>(item: R) -> Self
    where
        R: AsSoaRef<Item = Self>;
}
