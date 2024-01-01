pub trait Soapy: Sized {
    type SoaRaw: SoaRaw<Self>;
}

/// A low-level utility providing fundamental operations needed by `Soa<T>`
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
/// - 0 if no previous calls to [`SoaRaw::grow`] or [`SoaRaw::shrink`] have been
/// made, or
/// - the same value as was used for `new_capacity` in previous calls
/// to [`SoaRaw::grow`] and [`SoaRaw::shrink`]
pub trait SoaRaw<T>: Copy + Clone {
    /// For each field with type `F` in `T`, `Slices` has a field with type
    /// `&[F]`
    type Slices<'a>
    where
        Self: 'a;

    /// For each field with type `F` in `T`, `SlicesMut` has a field with type
    /// `&mut [F]`
    type SlicesMut<'a>
    where
        Self: 'a;

    /// Creates a `Self` with dangling pointers for all its fields and without
    /// allocating memory.
    fn dangling() -> Self;

    /// Constructs safe, immutable slices of the arrays managed by `Self`.
    fn slices(&self, len: usize) -> Self::Slices<'_>;

    /// Constructs safe, mutable slices of the arrays managed by `Self`.
    fn slices_mut(&mut self, len: usize) -> Self::SlicesMut<'_>;

    /// Grows the allocation with room for `old_capacity` elements to fit
    /// `new_capacity` elements and moves `length` number of array elements to
    /// their new locations.
    ///
    /// # Safety
    ///
    /// The caller must verify that
    ///
    /// - `size_of::<T>() > 0`
    /// - `new_capacity > old_capacity`
    /// - `length <= old_capacity`
    /// - `old_capacity == PREV_CAP`
    unsafe fn grow(&mut self, old_capacity: usize, new_capacity: usize, length: usize);

    /// Shrinks the allocation with room for `old_capacity` elements to fit
    /// `new_capacity` elements and moves `length` number of array elements to
    /// their new locations. Deallocates if new_capacity is 0.
    ///
    /// # Safety
    ///
    /// The caller must verify that
    ///
    /// - `size_of::<T>() > 0`
    /// - `new_capacity < old_capacity`
    /// - `length <= new_capacity`
    /// - `old_capacity == PREV_CAP`
    unsafe fn shrink(&mut self, old_capacity: usize, new_capacity: usize, length: usize);

    /// Deallocates the allocation with room for `capacity` elements.
    ///
    /// # Safety
    ///
    /// It is not valid to use `Self` after calling this method as the array
    /// pointers are not updated. The caller must verify that
    ///
    /// - `old_capacity == PREV_CAP`
    unsafe fn dealloc(&mut self, capacity: usize);

    /// Copies `count` elements from `src` index to `dst` index in each of the
    /// arrays.
    ///
    /// # Safety
    ///
    /// The caller must verify that
    ///
    /// - `src < PREV_CAP`
    /// - `dst < PREV_CAP`
    /// - `src + count <= PREV_CAP`
    /// - `dst + count <= PREV_CAP`
    unsafe fn copy(&mut self, src: usize, dst: usize, count: usize);

    /// Sets the element at `index` to `element`.
    ///
    /// # Safety
    ///
    /// The caller must verify that
    ///
    /// - `index < PREV_CAP`
    unsafe fn set(&mut self, index: usize, element: T);

    /// Gets the element at `index`.
    ///
    /// # Safety
    ///
    /// After calling `get`, the element at `index` should be treated as having
    /// been moved out of `Self` and into the caller. Therefore, it is no longer
    /// valid to reference this array element either by value or by reference.
    /// The caller must verify that
    ///
    /// - `index < PREV_CAP`
    unsafe fn get(&self, index: usize) -> T;
}
