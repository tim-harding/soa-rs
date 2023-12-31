// TODO: ZSTs

use soapy_shared::{SoaRaw, Soapy};

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
        self.maybe_grow();
        unsafe {
            self.raw.set(self.len, element);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe { self.raw.get(self.len) })
        }
    }

    fn maybe_grow(&mut self) {
        if self.len < self.capacity {
            return;
        }
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
