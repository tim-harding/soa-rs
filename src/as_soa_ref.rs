use crate::Soapy;

pub trait AsSoaRef {
    type Item: Soapy;

    fn as_soa_ref(&self) -> <Self::Item as Soapy>::Ref<'_>;
}
