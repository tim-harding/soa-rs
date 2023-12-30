// TODO: ZSTs

use crate::{SoaRaw, Soapy};

pub struct Soa<T>
where
    T: Soapy,
{
    len: usize,
    capacity: usize,
    raw: T::SoaRaw,
}

impl<T> Soa<T>
where
    T: Soapy,
{
    pub fn new() -> Self {
        Self {
            len: 0,
            capacity: 0,
            raw: T::SoaRaw::new(),
        }
    }

    pub fn push(&mut self, element: T) {
        if self.len == self.capacity {
            self.grow();
        }
        unsafe {
            self.raw.set(self.len, element);
        }
    }

    fn grow(&mut self) {
        let new_capacity = match self.capacity {
            0 => 4,
            cap => cap * 2,
        };

        unsafe {
            self.raw.grow(self.capacity, new_capacity, self.len);
        }

        self.capacity = new_capacity;
    }
}
