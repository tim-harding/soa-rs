#[derive(Copy, Clone)]
pub struct SliceData<R>
where
    R: Copy + Clone,
{
    pub len: usize,
    pub raw: R,
}
