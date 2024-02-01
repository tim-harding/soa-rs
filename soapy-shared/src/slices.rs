pub trait Slices {
    type Item<'a>
    where
        Self: 'a;

    fn iter(&self) -> impl Iterator<Item = Self::Item<'_>>;
}
