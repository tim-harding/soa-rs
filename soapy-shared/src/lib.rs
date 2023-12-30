mod soa;
pub use soa::Soa;

pub trait Soapy {
    type SoaRaw: SoaRaw;
}

pub trait SoaRaw {
    type Item: Sized;
    type Fields<'a>
    where
        Self: 'a;
    type FieldsMut<'a>
    where
        Self: 'a;

    fn new() -> Self;
    fn fields(&self, len: usize) -> Self::Fields<'_>;
    fn fields_mut<'a>(&'a mut self, len: usize) -> Self::FieldsMut<'a>;
    unsafe fn realloc(&mut self, old_capacity: usize, new_capacity: usize);
    unsafe fn dealloc(&mut self, capacity: usize);
    unsafe fn copy(&mut self, src: usize, dst: usize, count: usize);
    unsafe fn set(&mut self, index: usize, element: Self::Item);
    unsafe fn get(&self, index: usize) -> Self::Item;
}
