use crate::{BuilderDataType, BuilderDeserializer, BuilderError, Closure};
use serde::de::{DeserializeSeed, MapAccess};

pub struct BuilderMapAccess<'s, 'de, I>
where
    I: Iterator<Item = (BuilderDataType<'de>, BuilderDataType<'de>)>,
{
    pub(crate) closure: &'s mut Closure<'de>,
    pub(crate) data: I,
    pub(crate) leftover: Option<BuilderDataType<'de>>,
    pub(crate) size_hint: Option<usize>,
}

impl<'s, 'de, I> MapAccess<'de> for BuilderMapAccess<'s, 'de, I>
where
    I: Iterator<Item = (BuilderDataType<'de>, BuilderDataType<'de>)>,
{
    type Error = BuilderError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((a, b)) = self.data.next() {
            self.leftover = Some(b);
            let v = seed.deserialize(BuilderDeserializer {
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
            seed.deserialize(BuilderDeserializer {
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
            let va = kseed.deserialize(BuilderDeserializer {
                closure: self.closure,
                data: a,
            })?;
            let vb = vseed.deserialize(BuilderDeserializer {
                closure: self.closure,
                data: b,
            })?;
            Ok(Some((va, vb)))
        } else {
            Ok(None)
        }
    }
}
