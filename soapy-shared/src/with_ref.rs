use std::{
    cmp::Ordering,
    fmt::{self, Debug, Formatter},
};

/// Similar to [`AsRef`], except that instead of returning `&T`, it accepts
/// closure that takes `&T`.
///
/// This is required for getting a `&T` from a [`Soapy::Ref`] or
/// [`Soapy::RefMut`] because these types only contain references to each of
/// `T`'s fields. In order to get a reference to `T`, `T` must first be
/// constructed on the stack, but returning that reference is a lifetime
/// violation. Therefore, we must invert the control flow. Note that the same
/// idea for getting `&mut T` is less effective, as we would have to write all
/// the fields to `T` back to their actual storage locations in the `Soa` after
/// the closure, even if only some of the fields were modified.
///
/// This trait also provides various convenience methods that are equivalent
/// to common [`std`] trait methods. Ideally, we would like to write blanket
/// implementations for all types that implement [`WithRef`]. Unfortunately,
/// doing so is a violation of the orphan rule, so this is the next best thing.
/// All of these convenience methods forward to the implementations on `T`.
///
/// [`Soapy::Ref`]: crate::Soapy::Ref
/// [`Soapy::RefMut`]: crate::Soapy::RefMut
pub trait WithRef {
    type Item;

    /// Calls the provided function with a reference to `T`
    fn with_ref<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Item) -> R;

    /// Returns a clone of `Self::Item`.
    ///
    /// Prefer [`copied`] where possible.
    ///
    /// [`copied`]: WithRef::copied
    fn cloned(&self) -> Self::Item
    where
        Self::Item: Clone,
    {
        self.with_ref(|me| me.clone())
    }

    /// Returns a copy of `Self::Item`.
    ///
    /// Prefer this over [`cloned`] where possible.
    ///
    /// [`cloned`]: WithRef::cloned
    fn copied(&self) -> Self::Item
    where
        Self::Item: Copy,
    {
        self.with_ref(|me| *me)
    }

    /// Convenience for `Self::Item`'s [`Debug::fmt`] implementation.
    fn debug(&self, f: &mut Formatter<'_>) -> fmt::Result
    where
        Self::Item: Debug,
    {
        self.with_ref(|me| me.fmt(f))
    }

    /// Convenience for `Self::Item`'s [`PartialEq::eq`] implementation.
    fn partial_eq(&self, other: &impl WithRef<Item = Self::Item>) -> bool
    where
        Self::Item: PartialEq,
    {
        self.with_ref(|me| other.with_ref(|them| me == them))
    }

    /// Convenience for `Self::Item`'s [`PartialOrd::partial_cmp`] implementation.
    fn partial_ord(&self, other: &impl WithRef<Item = Self::Item>) -> Option<Ordering>
    where
        Self::Item: PartialOrd,
    {
        self.with_ref(|me| other.with_ref(|them| me.partial_cmp(them)))
    }

    /// Convenience for `Self::Item`'s [`Ord::cmp`] implementation.
    fn ord(&self, other: &impl WithRef<Item = Self::Item>) -> Ordering
    where
        Self::Item: Ord,
    {
        self.with_ref(|me| other.with_ref(|them| me.cmp(them)))
    }
}
