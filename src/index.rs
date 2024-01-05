use crate::Soa;
use soapy_shared::{RawSoa, Soapy};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

pub trait SoaIndex<T>
where
    T: Soapy,
{
    type Output<'a>
    where
        T: 'a;
    type OutputMut<'a>
    where
        T: 'a;

    fn get(self, soa: &Soa<T>) -> Option<Self::Output<'_>>;
    fn get_mut(self, soa: &mut Soa<T>) -> Option<Self::OutputMut<'_>>;
}

impl<T> SoaIndex<T> for usize
where
    T: Soapy,
{
    type Output<'a> = <T::RawSoa as RawSoa<T>>::ItemRef<'a> where T: 'a;

    type OutputMut<'a> = <T::RawSoa as RawSoa<T>>::ItemRefMut<'a>
    where
        T: 'a;

    fn get(self, soa: &Soa<T>) -> Option<Self::Output<'_>> {
        if self < soa.len {
            Some(unsafe { soa.raw.get_ref(self) })
        } else {
            None
        }
    }

    fn get_mut(self, soa: &mut Soa<T>) -> Option<Self::OutputMut<'_>> {
        if self < soa.len {
            Some(unsafe { soa.raw.get_mut(self) })
        } else {
            None
        }
    }
}

impl<T> SoaIndex<T> for Range<usize>
where
    T: Soapy,
{
    type Output<'a> = <T::RawSoa as RawSoa<T>>::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = <T::RawSoa as RawSoa<T>>::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, soa: &Soa<T>) -> Option<Self::Output<'_>> {
        if self.start <= soa.len && self.end <= soa.len {
            Some(unsafe { soa.raw.slices(self.start, self.end) })
        } else {
            None
        }
    }

    fn get_mut(self, soa: &mut Soa<T>) -> Option<Self::OutputMut<'_>> {
        if self.start <= soa.len && self.end <= soa.len {
            Some(unsafe { soa.raw.slices_mut(self.start, self.end) })
        } else {
            None
        }
    }
}

impl<T> SoaIndex<T> for RangeFrom<usize>
where
    T: Soapy,
{
    type Output<'a> = <T::RawSoa as RawSoa<T>>::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = <T::RawSoa as RawSoa<T>>::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, soa: &Soa<T>) -> Option<Self::Output<'_>> {
        (self.start..soa.len).get(soa)
    }

    fn get_mut(self, soa: &mut Soa<T>) -> Option<Self::OutputMut<'_>> {
        (self.start..soa.len).get_mut(soa)
    }
}

impl<T> SoaIndex<T> for RangeFull
where
    T: Soapy,
{
    type Output<'a> = <T::RawSoa as RawSoa<T>>::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = <T::RawSoa as RawSoa<T>>::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, soa: &Soa<T>) -> Option<Self::Output<'_>> {
        (0..soa.len).get(soa)
    }

    fn get_mut(self, soa: &mut Soa<T>) -> Option<Self::OutputMut<'_>> {
        (0..soa.len).get_mut(soa)
    }
}

impl<T> SoaIndex<T> for RangeInclusive<usize>
where
    T: Soapy,
{
    type Output<'a> = <T::RawSoa as RawSoa<T>>::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = <T::RawSoa as RawSoa<T>>::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, soa: &Soa<T>) -> Option<Self::Output<'_>> {
        (*self.start()..*self.end() + 1).get(soa)
    }

    fn get_mut(self, soa: &mut Soa<T>) -> Option<Self::OutputMut<'_>> {
        (*self.start()..*self.end() + 1).get_mut(soa)
    }
}

impl<T> SoaIndex<T> for RangeTo<usize>
where
    T: Soapy,
{
    type Output<'a> = <T::RawSoa as RawSoa<T>>::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = <T::RawSoa as RawSoa<T>>::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, soa: &Soa<T>) -> Option<Self::Output<'_>> {
        (0..self.end).get(soa)
    }

    fn get_mut(self, soa: &mut Soa<T>) -> Option<Self::OutputMut<'_>> {
        (0..self.end).get_mut(soa)
    }
}

impl<T> SoaIndex<T> for RangeToInclusive<usize>
where
    T: Soapy,
{
    type Output<'a> = <T::RawSoa as RawSoa<T>>::Slices<'a>
    where
        T: 'a;

    type OutputMut<'a> = <T::RawSoa as RawSoa<T>>::SlicesMut<'a>
    where
        T: 'a;

    fn get(self, soa: &Soa<T>) -> Option<Self::Output<'_>> {
        (0..self.end + 1).get(soa)
    }

    fn get_mut(self, soa: &mut Soa<T>) -> Option<Self::OutputMut<'_>> {
        (0..self.end + 1).get_mut(soa)
    }
}
