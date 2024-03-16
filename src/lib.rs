use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Display;
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
    Reference(Rc<BuilderDataType<'de>>),
    Store(Rc<RefCell<BuilderDataType<'de>>>),
    IfThenElse(Vec<BuilderDataType<'de>>),
    Repeat(Vec<BuilderDataType<'de>>),
    Range(Vec<BuilderDataType<'de>>),
    Sum(Vec<BuilderDataType<'de>>),
    Multiply(Vec<BuilderDataType<'de>>),
    Index,
    Unique,
}

#[derive(Debug)]
pub enum BuilderError {
    InvalidMapAccess,
    InvalidDeserialization(String),
    InvalidFunctionArgument,
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
            BuilderError::InvalidFunctionArgument => todo!(),
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

pub struct BuilderDeserializer<'de> {
    data: BuilderDataType<'de>,
}
pub struct BuilderDeserializerRef<'r, 'de> {
    data: &'r BuilderDataType<'de>,
}

struct BuilderListAccess<'de, I>
where
    I: Iterator<Item = BuilderDataType<'de>> + ExactSizeIterator,
{
    data: I,
}
struct BuilderListAccessRef<'r, 'de, I>
where
    'de: 'r,
    I: Iterator<Item = &'r BuilderDataType<'de>> + ExactSizeIterator,
{
    data: I,
}
struct BuilderMapAccess<'de, I>
where
    I: Iterator<Item = (BuilderDataType<'de>, BuilderDataType<'de>)> + ExactSizeIterator,
{
    data: I,
    leftover: Option<BuilderDataType<'de>>,
}

struct BuilderMapAccessRef<'r, 'de, I>
where
    'de: 'r,
    I: Iterator<Item = &'r (BuilderDataType<'de>, BuilderDataType<'de>)> + ExactSizeIterator,
{
    data: I,
    leftover: Option<&'r BuilderDataType<'de>>,
}

impl<'de> BuilderDataType<'de> {
    fn if_then_else_ref<'a>(
        v: &'a Vec<BuilderDataType<'de>>,
    ) -> Result<&'a BuilderDataType<'de>, BuilderError> {
        let mut i = v.iter();
        let Some(condition) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        let Some(if_true) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        let Some(if_false) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        if condition.check_true() {
            Ok(if_true)
        } else {
            Ok(if_false)
        }
    }

    fn if_then_else(v: Vec<BuilderDataType<'de>>) -> Result<BuilderDataType<'de>, BuilderError> {
        let mut i = v.into_iter();
        let Some(condition) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        let Some(if_true) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        let Some(if_false) = i.next() else {
            return Err(BuilderError::InvalidFunctionArgument);
        };
        if condition.check_true() {
            Ok(if_true)
        } else {
            Ok(if_false)
        }
    }

    pub fn check_true(&self) -> bool {
        match self {
            BuilderDataType::Empty => false,
            BuilderDataType::Boolean(b) => *b,
            BuilderDataType::Integer(v) => *v != 0,
            BuilderDataType::Unsigned(v) => *v != 0,
            BuilderDataType::Number(v) => *v != 0.0,
            BuilderDataType::String(s) => !s.is_empty(),
            BuilderDataType::Map(c) => !c.is_empty(),
            BuilderDataType::List(c) => !c.is_empty(),
            BuilderDataType::Reference(r) => r.as_ref().check_true(),
            BuilderDataType::Store(r) => r.as_ref().borrow().check_true(),
            BuilderDataType::IfThenElse(v) => {
                BuilderDataType::if_then_else_ref(v).map(|r|r.check_true()).unwrap_or(false)
            },
            BuilderDataType::Repeat(v) => {
                v.first().map(|r|r.check_true()).unwrap_or(false)
            }
            _ => false,
        }
    }

    pub fn to_unsigned(&self) -> u64 {
        match self {
            BuilderDataType::Empty => 0,
            BuilderDataType::Boolean(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            BuilderDataType::Integer(v) => (*v).max(0) as u64,
            BuilderDataType::Unsigned(v) => *v,
            BuilderDataType::Number(v) => {
                if v.is_sign_positive() && v.is_normal() {
                    *v as u64
                } else {
                    0
                }
            }
            BuilderDataType::String(v) => v.parse().unwrap_or(0),
            BuilderDataType::Map(v) => v.len() as u64,
            BuilderDataType::List(v) => v.len() as u64,
            BuilderDataType::Reference(r) => r.as_ref().to_unsigned(),
            BuilderDataType::Store(r) => r.as_ref().borrow().to_unsigned(),
            BuilderDataType::IfThenElse(v) => {
                if let Ok(r) = BuilderDataType::if_then_else_ref(v) {
                    r.to_unsigned()
                } else {
                    0
                }
            },
            BuilderDataType::Repeat(v) => {
                v.first().map(|r|r.to_unsigned()).unwrap_or(0)
            }
            _ => 0,
        }
    }
}

impl<'r, 'de> serde::Deserializer<'de> for BuilderDeserializerRef<'r, 'de> {
    type Error = BuilderError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.data {
            BuilderDataType::Empty => todo!(),
            BuilderDataType::Boolean(v) => visitor.visit_bool(*v),
            BuilderDataType::Integer(v) => visitor.visit_i64(*v),
            BuilderDataType::Unsigned(v) => visitor.visit_u64(*v),
            BuilderDataType::Number(v) => visitor.visit_f64(*v),
            BuilderDataType::String(c) => match c {
                Cow::Borrowed(v) => visitor.visit_borrowed_str(*v),
                Cow::Owned(v) => visitor.visit_str(v),
            },
            BuilderDataType::Map(v) => visitor.visit_map(BuilderMapAccessRef {
                data: v.iter(),
                leftover: None,
            }),
            BuilderDataType::List(v) => visitor.visit_seq(BuilderListAccessRef { data: v.iter() }),
            BuilderDataType::Reference(r) => {
                BuilderDeserializerRef { data: r.as_ref() }.deserialize_any(visitor)
            }
            BuilderDataType::Store(r) => BuilderDeserializer {
                data: r.as_ref().borrow().clone(),
            }
            .deserialize_any(visitor),
            BuilderDataType::IfThenElse(v) => BuilderDeserializerRef {
                data: BuilderDataType::if_then_else_ref(v)?,
            }
            .deserialize_any(visitor),
            BuilderDataType::Repeat(v) =>  {
                let mut it = v.iter();
                let times = it.next().map_or(0, |r|r.to_unsigned());
                visitor.visit_seq(BuilderListAccessRef { data: it.cycle().take(times as usize) })
            },
            _ => todo!(),
        }
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
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
            }),
            BuilderDataType::List(v) => visitor.visit_seq(BuilderListAccess {
                data: v.into_iter(),
            }),
            BuilderDataType::Reference(r) => match Rc::try_unwrap(r) {
                Ok(data) => BuilderDeserializer { data }.deserialize_any(visitor),
                Err(r) => BuilderDeserializerRef { data: &r }.deserialize_any(visitor),
            },
            BuilderDataType::Store(r) => match Rc::try_unwrap(r) {
                Ok(c) => BuilderDeserializer {
                    data: c.into_inner(),
                }
                .deserialize_any(visitor),
                Err(r) => BuilderDeserializer {
                    data: r.as_ref().borrow().clone(),
                }
                .deserialize_any(visitor),
            },
            BuilderDataType::IfThenElse(v) => BuilderDeserializer {
                data: BuilderDataType::if_then_else(v)?,
            }
            .deserialize_any(visitor),
            BuilderDataType::Repeat(v) =>  {
                let mut it = v.iter();
                let times = it.next().map_or(0, |r|r.to_unsigned());
                visitor.visit_seq(BuilderListAccessRef { data: it.cycle().take(times) })
            },
            _ => todo!(),
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
            let v = seed.deserialize(BuilderDeserializer { data: a })?;
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
            seed.deserialize(BuilderDeserializer { data: leftover })
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
            let va = kseed.deserialize(BuilderDeserializer { data: a })?;
            let vb = vseed.deserialize(BuilderDeserializer { data: b })?;
            Ok(Some((va, vb)))
        } else {
            Ok(None)
        }
    }
}

impl<'r, 'de, I> MapAccess<'de> for BuilderMapAccessRef<'r, 'de, I>
where
    I: Iterator<Item = &'r (BuilderDataType<'de>, BuilderDataType<'de>)> + ExactSizeIterator,
{
    type Error = BuilderError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((a, b)) = self.data.next() {
            self.leftover = Some(b);
            let v = seed.deserialize(BuilderDeserializerRef { data: a })?;
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
            seed.deserialize(BuilderDeserializerRef { data: leftover })
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
            let va = kseed.deserialize(BuilderDeserializerRef { data: a })?;
            let vb = vseed.deserialize(BuilderDeserializerRef { data: b })?;
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
            Ok(Some(seed.deserialize(BuilderDeserializer { data })?))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.data.len())
    }
}
impl<'r, 'de, I> SeqAccess<'de> for BuilderListAccessRef<'r, 'de, I>
where
    I: Iterator<Item = &'r BuilderDataType<'de>> + ExactSizeIterator,
{
    type Error = BuilderError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(data) = self.data.next() {
            Ok(Some(seed.deserialize(BuilderDeserializerRef { data })?))
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
    let builder = BuilderDeserializer { data };

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
        let data = BuilderDataType::Reference(Rc::new(BuilderDataType::List(vec![
            BuilderDataType::Integer(123),
            BuilderDataType::Boolean(true),
            BuilderDataType::String(Cow::from("test")),
        ])));

        let data = BuilderDataType::Reference(Rc::new(BuilderDataType::List(vec![
            data.clone(),
            data.clone(),
            data.clone(),
        ])));

        let data = BuilderDataType::List(vec![data, BuilderDataType::Map(vec![])]);

        let test: TestComplex = from_data(data).unwrap();

        assert_eq!(fixture_data_complex(0), test);
    }
}
