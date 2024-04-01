


use crate::{BuilderDataType, BuilderDeserializerRef, BuilderError, Closure};
use serde::de::{DeserializeSeed, MapAccess};

pub struct BuilderMapAccessRef<'s, 'r, 'de, I>
where
    'de: 'r,
    I: Iterator<Item = &'r (BuilderDataType<'de>, BuilderDataType<'de>)>,
{
    pub(crate) closure: &'s mut Closure<'de>,
    pub(crate) data: I,
    pub(crate) leftover: Option<&'r BuilderDataType<'de>>,
    pub(crate) size_hint: Option<usize>,
}


impl<'s, 'r, 'de, I> MapAccess<'de> for BuilderMapAccessRef<'s, 'r, 'de, I>
where
    I: Iterator<Item = &'r (BuilderDataType<'de>, BuilderDataType<'de>)>,
{
    type Error = BuilderError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((a, b)) = self.data.next() {
            self.leftover = Some(b);
            let v = seed.deserialize(BuilderDeserializerRef {
                closure: self.closure,
                data: a,
            })?;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(leftover) = self.leftover.take() {
            seed.deserialize(BuilderDeserializerRef {
                closure: self.closure,
                data: leftover,
            })
        } else {
            Err(BuilderError::InvalidMapAccess)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.size_hint
    }

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        if let Some((a, b)) = self.data.next() {
            self.leftover = None;
            let va = kseed.deserialize(BuilderDeserializerRef {
                closure: self.closure,
                data: a,
            })?;
            let vb = vseed.deserialize(BuilderDeserializerRef {
                closure: self.closure,
                data: b,
            })?;
            Ok(Some((va, vb)))
        } else {
            Ok(None)
        }
    }
}