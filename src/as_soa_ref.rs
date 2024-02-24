pub trait AsSoaRef {
    type Ref<'a>
    where
        Self: 'a;

    fn as_soa_ref(&self) -> Self::Ref<'_>;
}
