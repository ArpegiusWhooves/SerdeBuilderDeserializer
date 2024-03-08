use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::Cell;
use std::fmt::Display;

use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum BuilderDataType<'de> {
    Empty,
    Boolean(bool),
    Integer(i64),
    Unsigned(u64),
    Number(f64),
    String(Cow<'de, str>),
    Map(Vec<(BuilderDataType<'de>, BuilderDataType<'de>)>),
    List(Vec<BuilderDataType<'de>>),
    Reference(u64),
    FunctionCall(Vec<BuilderDataType<'de>>),
 }

#[derive(Debug)]
pub enum BuilderError {
    InvalidMapAccess,
    InvalidDeserialization(String),
}

impl Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::InvalidMapAccess => {
                f.write_fmt(format_args!("Invalid map access sequence."))
            }
            BuilderError::InvalidDeserialization(err) => {
                f.write_fmt(format_args!("Invalid deserialization: {err}"))
            }
        }
    }
}

impl std::error::Error for BuilderError {}

impl Error for BuilderError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        BuilderError::InvalidDeserialization(format!("{msg}"))
    }
}

pub struct BuilderReferenceMapper<'de> {
    refs: BTreeMap< u64, Cow<'de, [BuilderDataType<'de>]> >,
}

pub struct BuilderDeserializer<'de> {
    data: BuilderDataType<'de>,
    refs: Option<Rc<BuilderReferenceMapper<'de>>>,
}


struct BuilderListAccess<'de, I>
where
    I: Iterator<Item = BuilderDataType<'de>> + ExactSizeIterator,
{
    data: I,
    refs: Option<Rc<BuilderReferenceMapper<'de>>>,
}

struct BuilderMapAccess<'de, I>
where
    I: Iterator<Item = (BuilderDataType<'de>, BuilderDataType<'de>)> + ExactSizeIterator,
{
    data: I,
    leftover: Option<BuilderDataType<'de>>,
    refs: Option<Rc<BuilderReferenceMapper<'de>>>,
}

impl<'de> serde::Deserializer<'de> for BuilderDeserializer<'de> {
    type Error = BuilderError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.data {
            BuilderDataType::Empty => todo!(),
            BuilderDataType::Boolean(v) => visitor.visit_bool(v),
            BuilderDataType::Integer(v) => visitor.visit_i64(v),
            BuilderDataType::Unsigned(v) => visitor.visit_u64(v),
            BuilderDataType::Number(v) => visitor.visit_f64(v),
            BuilderDataType::String(c) => match c {
                Cow::Borrowed(v) => visitor.visit_borrowed_str(v),
                Cow::Owned(v) => visitor.visit_string(v),
            },
            BuilderDataType::Map(v) => visitor.visit_map(BuilderMapAccess {
                data: v.into_iter(),
                leftover: None,
                refs: self.refs,
            }),
            BuilderDataType::List(v) => visitor.visit_seq(BuilderListAccess {
                data: v.into_iter(),
                refs: self.refs,
            }),
            BuilderDataType::Reference(_) => todo!(),
            BuilderDataType::FunctionCall(_) => todo!(),
        }
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, I> MapAccess<'de> for BuilderMapAccess<'de, I>
where
    I: Iterator<Item = (BuilderDataType<'de>, BuilderDataType<'de>)> + ExactSizeIterator,
{
    type Error = BuilderError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((a, b)) = self.data.next() {
            self.leftover = Some(b);
            let v = seed.deserialize(BuilderDeserializer{
                refs: self.refs.clone(),
                data: a
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
            seed.deserialize(BuilderDeserializer{
                refs: self.refs.clone(),
                data: leftover
            })
        } else {
            Err(BuilderError::InvalidMapAccess)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.data.len())
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
            let va = kseed.deserialize(BuilderDeserializer{
                refs: self.refs.clone(),
                data:a
            })?;
            let vb = vseed.deserialize(BuilderDeserializer{
                refs: self.refs.clone(),
                data:b
            })?;
            Ok(Some((va, vb)))
        } else {
            Ok(None)
        }
    }
}

impl<'de, I> SeqAccess<'de> for BuilderListAccess<'de, I>
where
    I: Iterator<Item = BuilderDataType<'de>> + ExactSizeIterator,
{
    type Error = BuilderError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(data) = self.data.next() {
            Ok(Some(seed.deserialize(BuilderDeserializer {
                refs: self.refs.clone(),
                data
            })?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.data.len())
    }
}

pub fn from_data<'a, T>(data: BuilderDataType<'a>) -> Result<T, BuilderError>
where
    T: Deserialize<'a>,
{
    let builder = BuilderDeserializer { 
        refs: None,
        data 
    };

    Ok(T::deserialize(builder)?)
}

#[cfg(test)]
mod tests {

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
    struct TestSimple {
        a: i32,
        b: bool,
        c: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
    struct TestComplex {
        a: Vec<TestSimple>,
        b: BTreeMap<String, TestComplex>,
    }

    use super::*;

    fn fixture_data_simple() -> TestSimple {
        TestSimple {
            a: 123,
            b: true,
            c: "test".to_owned(),
        }
    }

    fn fixture_data_complex(req: u32) -> TestComplex {
        TestComplex {
            a: vec![
                fixture_data_simple(),
                fixture_data_simple(),
                fixture_data_simple(),
            ],
            b: if req == 0 {
                BTreeMap::new()
            } else {
                BTreeMap::from([("test".to_owned(), fixture_data_complex(req - 1))])
            },
        }
    }

    #[test]
    fn test_map_access_with_names() {
        let data = BuilderDataType::Map(vec![
            (
                BuilderDataType::String(Cow::from("a")),
                BuilderDataType::Integer(123),
            ),
            (
                BuilderDataType::String(Cow::from("b")),
                BuilderDataType::Boolean(true),
            ),
            (
                BuilderDataType::String(Cow::from("c")),
                BuilderDataType::String(Cow::from("test")),
            ),
        ]);

        let test: TestSimple = from_data(data).unwrap();

        assert_eq!(fixture_data_simple(), test);
    }
    #[test]
    fn test_map_access_with_idnex() {
        let data = BuilderDataType::Map(vec![
            (BuilderDataType::Unsigned(0), BuilderDataType::Integer(123)),
            (BuilderDataType::Unsigned(1), BuilderDataType::Boolean(true)),
            (
                BuilderDataType::Unsigned(2),
                BuilderDataType::String(Cow::from("test")),
            ),
        ]);

        let test: TestSimple = from_data(data).unwrap();

        assert_eq!(fixture_data_simple(), test);
    }
    #[test]
    fn test_list_access() {
        let data = BuilderDataType::List(vec![
            BuilderDataType::Integer(123),
            BuilderDataType::Boolean(true),
            BuilderDataType::String(Cow::from("test")),
        ]);

        let test: TestSimple = from_data(data).unwrap();

        assert_eq!(fixture_data_simple(), test);
    }

    #[test]
    fn test_complex() {
        let data = BuilderDataType::List(vec![
            BuilderDataType::Integer(123),
            BuilderDataType::Boolean(true),
            BuilderDataType::String(Cow::from("test")),
        ]);

        let data = BuilderDataType::List(vec![
            BuilderDataType::List(vec![data.clone(), data.clone(), data.clone()]),
            BuilderDataType::Map(vec![]),
        ]);

        let test: TestComplex = from_data(data).unwrap();

        assert_eq!(fixture_data_complex(0), test);
    }
}
