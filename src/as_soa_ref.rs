pub trait AsSoaRef<'r, 'i> {
    type Item<'s>
    where
        Self: 's + 'r;

    fn as_soa_ref(&'r self) -> Self::Item<'i>;
}
