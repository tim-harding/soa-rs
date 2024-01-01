pub trait Soapy: Sized {
    type SoaRaw: SoaRaw<Self>;
}

pub trait SoaRaw<T>: Copy + Clone {
    type Slices<'a>
    where
        Self: 'a;

    type SlicesMut<'a>
    where
        Self: 'a;

    fn new() -> Self;
    fn slices(&self, len: usize) -> Self::Slices<'_>;
    fn slices_mut(&mut self, len: usize) -> Self::SlicesMut<'_>;
    unsafe fn grow(&mut self, old_capacity: usize, new_capacity: usize, length: usize);
    unsafe fn shrink(&mut self, old_capacity: usize, new_capacity: usize, length: usize);
    unsafe fn dealloc(&mut self, capacity: usize);
    unsafe fn copy(&mut self, src: usize, dst: usize, count: usize);
    unsafe fn set(&mut self, index: usize, element: T);
    unsafe fn get(&self, index: usize) -> T;
}
