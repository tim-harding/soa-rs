pub trait Soapy: Sized {
    type SoaRaw: SoaRaw<Self>;
}

pub trait SoaRaw<T> {
    type Fields<'a>
    where
        Self: 'a;

    type FieldsMut<'a>
    where
        Self: 'a;

    fn new() -> Self;
    fn fields(&self, len: usize) -> Self::Fields<'_>;
    fn fields_mut<'a>(&'a mut self, len: usize) -> Self::FieldsMut<'a>;
    unsafe fn grow(&mut self, old_capacity: usize, new_capacity: usize, length: usize);
    unsafe fn shrink(&mut self, old_capacity: usize, new_capacity: usize, length: usize);
    unsafe fn dealloc(&mut self, capacity: usize);
    unsafe fn copy(&mut self, src: usize, dst: usize, count: usize);
    unsafe fn set(&mut self, index: usize, element: T);
    unsafe fn get(&self, index: usize) -> T;
}
