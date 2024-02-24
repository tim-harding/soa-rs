use crate::Soapy;

/// A low-level utility providing fundamental operations needed by [`Soa`].
///
/// In particular, it manages an allocation and a set of pointers into
/// the allocation. Each of the pointers corresponds to a field of the type `T`
/// and is treated as an array of values of that field's type.
///
/// # Safety
///
/// Use of this type is inherently unsafe and should be restricted to the
/// implementation of `Soa`. There is no guarantee of contract stability between
/// versions. Further, this type will **neither** deallocate its memory **nor**
/// drop its contents when it is dropped. Special care must be taken to avoid
/// unsound use.
///
/// In the method documentation, it is established that `PREV_CAP` is
///
/// - 0 if no previous calls to [`SoaRaw::realloc_grow`] or [`SoaRaw::realloc_shrink`] have been
/// made, or
/// - the same value as was used for `new_capacity` in previous calls
/// to [`SoaRaw::realloc_grow`] and [`SoaRaw::realloc_shrink`]
///
/// [`Soa`]: crate::Soa
#[doc(hidden)]
pub unsafe trait SoaRaw: Copy + Clone {
    /// The type of element the SoA will contain.
    ///
    /// This is also the type for which the trait implementation is derived when
    /// using the derive macro.
    type Item: Soapy;

    /// Creates a [`SoaRaw`] with dangling pointers for all its fields and without
    /// allocating memory.
    fn dangling() -> Self;

    /// Construct a new [`SoaRaw`] with the given pointer and capacity.
    ///
    /// # Safety
    ///
    /// The pointer should come from a previous call to [`into_parts`] with
    /// `PREV_CAP == capacity`.
    ///
    /// [`into_parts`]: SoaRaw::into_parts
    unsafe fn from_parts(ptr: *mut u8, capacity: usize) -> Self;

    /// Decomposes a [`SoaRaw`] into its raw components.
    ///
    /// Returns the raw pointer to the underlying data. The same pointer should
    /// be used as the first argument to [`from_parts`].
    ///
    /// [`from_parts`]: SoaRaw::from_parts
    fn into_parts(self) -> *mut u8;

    /// Allocates room for `capacity` elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    ///
    /// - `size_of::<T>() > 0`
    /// - `capacity > 0`
    /// - `PREV_CAP == 0` (Otherwise use [`SoaRaw::realloc_grow`])
    #[must_use]
    unsafe fn alloc(capacity: usize) -> Self;

    /// Grows the allocation with room for `old_capacity` elements to fit
    /// `new_capacity` elements and moves `length` number of array elements to
    /// their new locations.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    ///
    /// - `size_of::<T>() > 0`
    /// - `new_capacity > old_capacity`
    /// - `length <= old_capacity`
    /// - `old_capacity > 0` (Otherwise use [`SoaRaw::alloc`])
    #[must_use]
    unsafe fn realloc_grow(
        &mut self,
        old_capacity: usize,
        new_capacity: usize,
        length: usize,
    ) -> Self;

    /// Shrinks the allocation with room for `old_capacity` elements to fit
    /// `new_capacity` elements and moves `length` number of array elements to
    /// their new locations.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    ///
    /// - `size_of::<T>() > 0`
    /// - `new_capacity < old_capacity`
    /// - `length <= new_capacity`
    /// - `old_capacity > 0` (Otherwise use [`SoaRaw::dealloc`])
    #[must_use]
    unsafe fn realloc_shrink(
        &mut self,
        old_capacity: usize,
        new_capacity: usize,
        length: usize,
    ) -> Self;

    /// Deallocates the allocation with room for `capacity` elements. The state
    /// after calling this method is equivalent to [`SoaRaw::dangling`].
    ///
    /// # Safety
    ///
    /// [`SoaRaw`] no longer valid after calling this function. The caller must ensure that
    ///
    /// - `size_of::<T>() > 0`
    /// - `old_capacity > 0`
    unsafe fn dealloc(self, old_capacity: usize);

    /// Copies `count` elements from `src` index to `dst` index in each of the
    /// arrays.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    ///
    /// - `src < PREV_CAP`
    /// - `dst < PREV_CAP`
    /// - `src + count <= PREV_CAP`
    /// - `dst + count <= PREV_CAP`
    unsafe fn copy_to(self, dst: Self, count: usize);

    /// Sets the element at `index` to `element`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    ///
    /// - `index < PREV_CAP`
    unsafe fn set(self, element: Self::Item);

    /// Gets the element at `index`.
    ///
    /// # Safety
    ///
    /// After calling this method, the element at `index` should be treated as
    /// having been moved out of [`SoaRaw`] and into the caller. Therefore, it
    /// is no longer valid to reference this array element either by value or by
    /// reference. The caller must ensure that
    ///
    /// - `index < PREV_CAP`
    unsafe fn get(self) -> Self::Item;

    /// Gets a reference to the element at `index`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    ///
    /// - `index < PREV_CAP`
    unsafe fn get_ref<'a>(self) -> <Self::Item as Soapy>::Ref<'a>;

    /// Gets a mutable reference to the element at `index`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    ///
    /// - `index < PREV_CAP`
    unsafe fn get_mut<'a>(self) -> <Self::Item as Soapy>::RefMut<'a>;

    /// Create a new [`SoaRaw`] starting at index `count`.
    ///
    /// This is similar to indexing by [`RangeFrom`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that
    ///
    /// - `count <= length`
    ///
    /// [`RangeFrom`]: std::ops::RangeFrom
    #[must_use]
    unsafe fn offset(self, count: usize) -> Self;

    unsafe fn slices<'a>(self, len: usize) -> <Self::Item as Soapy>::Slices<'a>;

    unsafe fn slices_mut<'a>(self, len: usize) -> <Self::Item as Soapy>::SlicesMut<'a>;
}
