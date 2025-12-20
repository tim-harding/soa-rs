use crate::AsSoaRef;

/// Construct an owned value by cloning fields from an SoA reference.
///
/// This trait allows constructing an owned instance of a type by cloning
/// all fields from an SoA element.
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
/// let owned = Point::from_soa_ref(&point_ref);
/// assert_eq!(owned, Point { x: 1, y: 2 });
/// ```
pub trait FromSoaRef {
    /// Constructs `Self` by cloning all fields from the SoA reference.
    fn from_soa_ref<R>(item: &R) -> Self
    where
        R: AsSoaRef<Item = Self>;
}
