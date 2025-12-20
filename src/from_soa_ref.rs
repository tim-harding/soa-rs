use crate::Soars;

/// Construct an owned value by cloning fields from an SoA reference.
///
/// This trait allows constructing an owned instance of a type by cloning
/// all fields from an SoA element.
pub trait FromSoaRef: Soars {
    /// Constructs `Self` by cloning all fields from the SoA reference.
    ///
    /// # Example
    ///
    /// ```
    /// # use soa_rs::{Soars, FromSoaRef, soa};
    /// #[derive(Soars, FromSoaRef, Debug, PartialEq, Clone)]
    /// #[soa_derive(Debug)]
    /// struct Point {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let soa = soa![Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
    /// let point_ref = soa.idx(0);
    /// let owned = Point::from_soa_ref(point_ref);
    /// assert_eq!(owned, Point { x: 1, y: 2 });
    /// ```
    fn from_soa_ref(item: <Self as Soars>::Ref<'_>) -> Self;
}

/// Analogous to [`clone`] or [`to_owned`], but for SoA array elements.
///
/// The opposite of [`FromSoaRef`].
///
/// [`to_owned`]: std::borrow::ToOwned::to_owned
/// [`clone`]: std::clone::Clone::clone
pub trait SoaRefToOwned<T>
where
    T: FromSoaRef,
{
    /// Construct an owned value from an SoA element.
    ///
    /// # Example
    ///
    /// ```
    /// # use soa_rs::{Soars, FromSoaRef, soa, SoaRefToOwned};
    /// #[derive(Soars, FromSoaRef, Debug, PartialEq, Clone)]
    /// #[soa_derive(Debug)]
    /// struct Point(f32, f32);
    ///
    /// let soa = soa![Point(1.0, 2.0), Point(3.0, 4.0)];
    /// let el = soa.idx(1);
    /// let owned = el.soa_ref_to_owned();
    /// assert_eq!(owned, Point(3.0, 4.0));
    /// ```
    fn soa_ref_to_owned(self) -> T;
}

/// Reflexive auto-implementation. Whenever [`FromSoaRef`] is implemented, so to
/// is this one going in the opposite direction. This is analogous to the
/// relationship between [`From`] and [`Into`].
impl<S, D> SoaRefToOwned<D> for S
where
    D: FromSoaRef,
    S: <D as Soars>::Ref,
{
    fn soa_ref_to_owned(self) -> D {
        D::from_soa_ref(self)
    }
}
