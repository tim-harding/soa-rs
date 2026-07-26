use crate::{Slice, SliceRef, SoaRaw, Soars};
use core::marker::PhantomData;

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
    T: 'a + Soars,
{
    slice: Slice<T, ()>,
    remainder: SliceRef<'a, T>,
    parts_remaining: usize,
    chunk_size: usize,
}

impl<'a, T> ChunksExact<'a, T>
where
    T: Soars,
{
    pub(crate) fn new(slice: SliceRef<'a, T>, chunk_size: usize) -> Self {
        let len = slice.len();
        let rem_len = len % chunk_size;
        let fst_len = len - rem_len;
        Self {
            remainder: slice.idx(fst_len..),
            slice: slice.slice,
            parts_remaining: fst_len / chunk_size,
            chunk_size,
        }
    }

    /// Returns the remainder of the original slice that has not been yielded by
    /// the iterator.
    pub fn remainder(&self) -> SliceRef<'a, T> {
        self.remainder
    }
}

impl<'a, T> Iterator for ChunksExact<'a, T>
where
    T: Soars,
{
    type Item = SliceRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.parts_remaining == 0 {
            None
        } else {
            let out = SliceRef {
                slice: self.slice,
                len: self.chunk_size,
                marker: PhantomData,
            };
            self.parts_remaining -= 1;
            // SAFETY: We had a remaining part, so we have at least chunk_size items
            self.slice.raw = unsafe { self.slice.raw().offset(self.chunk_size) };
            Some(out)
        }
    }
}
