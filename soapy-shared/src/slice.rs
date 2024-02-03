#[derive(Copy, Clone)]
pub struct SliceData<T>
where
    T: Copy + Clone,
{
    pub len: usize,
    pub raw: T,
}
