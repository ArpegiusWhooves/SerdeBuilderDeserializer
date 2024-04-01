

use crate::{BuilderDataType, BuilderDeserializerRef, BuilderError, Closure};
use serde::de::{DeserializeSeed, SeqAccess}; 

pub struct BuilderListAccessRef<'s, 'r, 'de, I>
where
    'de: 'r,
    I: Iterator<Item = &'r BuilderDataType<'de>>,
{
    pub(crate) closure: &'s mut Closure<'de>,
    pub(crate) data: I,
    pub(crate) size_hint: Option<usize>,
    pub(crate) index: usize,
}

impl<'s, 'r, 'de, I> SeqAccess<'de> for BuilderListAccessRef<'s, 'r, 'de, I>
where
    I: Iterator<Item = &'r BuilderDataType<'de>>,
{
    type Error = BuilderError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(data) = self.data.next() {
            self.closure.index = self.index;
            self.index += 1;
            Ok(Some(seed.deserialize(BuilderDeserializerRef {
                closure: self.closure,
                data,
            })?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.size_hint
    }
}