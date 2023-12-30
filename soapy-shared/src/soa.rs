// TODO: ZSTs

use crate::{SoaRaw, Soapy};

pub struct Soa<T>
where
    T: Soapy,
{
    len: usize,
    cap: usize,
    raw: T::SoaRaw,
}

impl<T> Soa<T>
where
    T: Soapy,
{
    pub fn new() -> Self {
        Self {
            len: 0,
            cap: 0,
            raw: T::SoaRaw::new(),
        }
    }
}
