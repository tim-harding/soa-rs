use crate::{Soa, Soars};
use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
use std::{
    fmt::{self, Formatter},
    marker::PhantomData,
};

impl<T> Serialize for Soa<T>
where
    T: Soars,
    for<'a> T::Ref<'a>: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for el in self {
            seq.serialize_element(&el)?;
        }
        seq.end()
    }
}

impl<'de, T> Deserialize<'de> for Soa<T>
where
    T: Soars + Deserialize<'de>,
    T: Deserializer<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SoaVisitor(PhantomData))
    }
}

struct SoaVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for SoaVisitor<T>
where
    T: Soars + Deserialize<'de>,
{
    type Value = Soa<T>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a sequence of maps")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut out = Soa::<T>::new();
        while let Some(next) = seq.next_element()? {
            out.push(next);
        }
        Ok(out)
    }
}
