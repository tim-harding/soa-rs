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
    /// The type to be borrowed.
    type Item;

    /// Calls the provided function with a reference to `T`
    fn with_ref<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Self::Item) -> R;

    /// Creates a clone of the borrowed value.
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

    /// Creates a copy of the borrowed value.
    fn copied(&self) -> Self::Item
    where
        Self::Item: Copy,
    {
        self.with_ref(|me| *me)
    }
}
