use soapy_shared::{RawSoa, Soapy};
use std::mem::size_of;

pub struct Drain<T>
where
    T: Soapy,
{
    pub(crate) raw: T::RawSoa,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl<T> Iterator for Drain<T>
where
    T: Soapy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let out = unsafe { self.raw.get(self.start) };
            self.start += 1;
            Some(out)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for Drain<T>
where
    T: Soapy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            self.end -= 1;
            Some(unsafe { self.raw.get(self.end) })
        }
    }
}

impl<T> Drop for Drain<T>
where
    T: Soapy,
{
    fn drop(&mut self) {
        while let Some(_) = self.next() {}
    }
}
