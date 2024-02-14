use crate::{Slice, SliceRef, SoaRaw, Soapy};
use std::marker::PhantomData;

/// An iterator over a [`Slice`] in (non-overlapping) chunks of `chunk_size`
/// elements.
///
/// When the slice len is not evenly divided by the chunk size, the last up to
/// `chunk_size-1` elements will be omitted but can be retrieved from the
/// [`remainder`] function from the iterator.
///
/// This struct is created by the [`chunks_exact`] method.
///
/// [`remainder`]: ChunksExact::remainder
/// [`chunks_exact`]: Slice::chunks_exact
pub struct ChunksExact<'a, T>
where
    T: 'a + Soapy,
{
    pub(crate) slice: Slice<T>,
    pub(crate) chunk_size: usize,
    pub(crate) _marker: PhantomData<&'a T>,
}

impl<'a, T> ChunksExact<'a, T>
where
    T: Soapy,
{
    /// Returns the remainder of the original slice that has not been yielded by
    /// the iterator.
    pub fn remainder(&self) -> SliceRef<'a, T> {
        SliceRef(self.slice, PhantomData)
    }
}

impl<'a, T> Iterator for ChunksExact<'a, T>
where
    T: Soapy,
{
    type Item = SliceRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.len < self.chunk_size {
            None
        } else {
            let out = Slice {
                len: self.chunk_size,
                raw: self.slice.raw,
            };
            self.slice.len -= self.chunk_size;
            self.slice.raw = unsafe { self.slice.raw.offset(self.chunk_size) };
            Some(SliceRef(out, PhantomData))
        }
    }
}
