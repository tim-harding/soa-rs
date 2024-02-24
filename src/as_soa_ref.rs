pub trait AsSoaRef {
    type Item<'a>
    where
        Self: 'a;

    fn as_soa_ref(&self) -> Self::Item<'_>;
}
