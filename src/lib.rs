use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::{Rc, Weak};

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
    Closure(Vec<BuilderDataType<'de>>),
    Argument(usize),
    Reference(Rc<BuilderDataType<'de>>),
    SelfReference(Weak<BuilderDataType<'de>>),
    Store(Rc<RefCell<BuilderDataType<'de>>>),
    Take(Rc<RefCell<BuilderDataType<'de>>>),
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
    InvalidSelfRefrence,
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
            BuilderError::InvalidSelfRefrence => todo!(),
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

struct Closure<'de> {
    args: Vec<BuilderDataType<'de>>,
    index: usize,
}

pub struct BuilderDeserializer<'s, 'de> {
    closure: &'s mut Closure<'de>,
    data: BuilderDataType<'de>,
}
pub struct BuilderDeserializerRef<'s, 'r, 'de> {
    closure: &'s mut Closure<'de>,
    data: &'r BuilderDataType<'de>,
}

struct BuilderListAccess<'s, 'de, I>
where
    I: Iterator<Item = BuilderDataType<'de>>,
{
    closure: &'s mut Closure<'de>,
    data: I,
    size_hint: Option<usize>,
    index: usize,
}
struct BuilderListAccessRef<'s, 'r, 'de, I>
where
    'de: 'r,
    I: Iterator<Item = &'r BuilderDataType<'de>>,
{
    closure: &'s mut Closure<'de>,
    data: I,
    size_hint: Option<usize>,
    index: usize,
}
struct BuilderMapAccess<'s, 'de, I>
where
    I: Iterator<Item = (BuilderDataType<'de>, BuilderDataType<'de>)>,
{
    closure: &'s mut Closure<'de>,
    data: I,
    leftover: Option<BuilderDataType<'de>>,
    size_hint: Option<usize>,
}

struct BuilderMapAccessRef<'s, 'r, 'de, I>
where
    'de: 'r,
    I: Iterator<Item = &'r (BuilderDataType<'de>, BuilderDataType<'de>)>,
{
    closure: &'s mut Closure<'de>,
    data: I,
    leftover: Option<&'r BuilderDataType<'de>>,
    size_hint: Option<usize>,
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

    pub fn take_one(&mut self) -> BuilderDataType<'de> {
        match self {
            BuilderDataType::Empty => BuilderDataType::Empty,
            BuilderDataType::Boolean(b) => {
                let result = BuilderDataType::Boolean(*b);
                if *b {
                    *b = false
                };
                result
            }
            BuilderDataType::Integer(b) => {
                let result = BuilderDataType::Integer(*b);
                if *b > 0 {
                    *b -= 1
                } else {
                    *b = 0
                }
                result
            }
            BuilderDataType::Unsigned(b) => {
                let result = BuilderDataType::Unsigned(*b);
                if *b > 0 {
                    *b -= 1
                }
                result
            }
            BuilderDataType::List(c) => c.pop().unwrap_or(BuilderDataType::Empty),
            _ => BuilderDataType::Empty,
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
            BuilderDataType::SelfReference(w) => {
                if let Some(r) = w.upgrade() {
                    r.as_ref().check_true()
                } else {
                    false
                }
            }
            BuilderDataType::Store(r) => r.as_ref().borrow().check_true(),
            BuilderDataType::Take(r) => r.as_ref().borrow_mut().take_one().check_true(),
            BuilderDataType::IfThenElse(v) => BuilderDataType::if_then_else_ref(v)
                .map(|r| r.check_true())
                .unwrap_or(false),
            BuilderDataType::Repeat(v) => v.first().map(|r| r.check_true()).unwrap_or(false),
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
            BuilderDataType::SelfReference(w) => {
                if let Some(r) = w.upgrade() {
                    r.as_ref().to_unsigned()
                } else {
                    0
                }
            }
            BuilderDataType::Store(r) => r.as_ref().borrow().to_unsigned(),
            BuilderDataType::Take(r) => r.as_ref().borrow_mut().take_one().to_unsigned(),
            BuilderDataType::IfThenElse(v) => {
                if let Ok(r) = BuilderDataType::if_then_else_ref(v) {
                    r.to_unsigned()
                } else {
                    0
                }
            }
            BuilderDataType::Repeat(v) => v.first().map(|r| r.to_unsigned()).unwrap_or(0),
            _ => 0,
        }
    }

    pub fn to_signed(&self) -> i64 {
        match self {
            BuilderDataType::Empty => 0,
            BuilderDataType::Boolean(v) => {
                if *v {
                    1
                } else {
                    0
                }
            }
            BuilderDataType::Integer(v) => *v,
            BuilderDataType::Unsigned(v) => (*v).min(i64::MAX as u64) as i64,
            BuilderDataType::Number(v) => {
                if v.is_normal() {
                    *v as i64
                } else {
                    0
                }
            }
            BuilderDataType::String(v) => v.parse().unwrap_or(0),
            BuilderDataType::Map(v) => v.len() as i64,
            BuilderDataType::List(v) => v.len() as i64,
            BuilderDataType::Reference(r) => r.as_ref().to_signed(),
            BuilderDataType::SelfReference(w) => {
                if let Some(r) = w.upgrade() {
                    r.as_ref().to_signed()
                } else {
                    0
                }
            }
            BuilderDataType::Store(r) => r.as_ref().borrow().to_signed(),
            BuilderDataType::Take(r) => r.as_ref().borrow_mut().take_one().to_signed(),
            BuilderDataType::IfThenElse(v) => {
                if let Ok(r) = BuilderDataType::if_then_else_ref(v) {
                    r.to_signed()
                } else {
                    0
                }
            }
            BuilderDataType::Repeat(v) => v.first().map(|r| r.to_signed()).unwrap_or(0),
            _ => 0,
        }
    }

    pub fn to_float(&self) -> f64 {
        match self {
            BuilderDataType::Empty => 0.0,
            BuilderDataType::Boolean(v) => {
                if *v {
                    1.0
                } else {
                    0.0
                }
            }
            BuilderDataType::Integer(v) => *v as f64,
            BuilderDataType::Unsigned(v) => *v as f64,
            BuilderDataType::Number(v) => *v,
            BuilderDataType::String(v) => v.parse().unwrap_or(0.0),
            BuilderDataType::Map(v) => v.len() as f64,
            BuilderDataType::List(v) => v.len() as f64,
            BuilderDataType::Reference(r) => r.as_ref().to_float(),
            BuilderDataType::SelfReference(w) => {
                if let Some(r) = w.upgrade() {
                    r.as_ref().to_float()
                } else {
                    0.0
                }
            }
            BuilderDataType::Store(r) => r.as_ref().borrow().to_float(),
            BuilderDataType::Take(r) => r.as_ref().borrow_mut().take_one().to_float(),
            BuilderDataType::IfThenElse(v) => {
                if let Ok(r) = BuilderDataType::if_then_else_ref(v) {
                    r.to_float()
                } else {
                    0.0
                }
            }
            BuilderDataType::Repeat(v) => v.first().map(|r| r.to_float()).unwrap_or(0.0),
            _ => 0.0,
        }
    }

    pub fn to_string(&self) -> Cow<'de, str> {
        match self {
            BuilderDataType::Empty => Cow::Owned(String::new()),
            BuilderDataType::Boolean(v) => {
                if *v {
                    Cow::Borrowed("true")
                } else {
                    Cow::Borrowed("false")
                }
            }
            BuilderDataType::Integer(v) => Cow::Owned(format!("{}", *v)),
            BuilderDataType::Unsigned(v) => Cow::Owned(format!("{}", *v)),
            BuilderDataType::Number(v) => Cow::Owned(format!("{}", *v)),
            BuilderDataType::String(v) => v.clone(),
            BuilderDataType::Map(v) => v.iter().fold(Cow::Owned(String::new()), |s, e| {
                let key = e.0.to_string();
                if key.is_empty() {
                    return s;
                }
                let value = e.1.to_string();
                if value.is_empty() {
                    return s;
                }
                if s.is_empty() {
                    Cow::Owned(format!("{key}:{value}"))
                } else {
                    Cow::Owned(format!("{s},{key}:{value}"))
                }
            }),
            BuilderDataType::List(v) => v.iter().fold(Cow::Owned(String::new()), |s, e| {
                if s.is_empty() {
                    e.to_string()
                } else {
                    let value = e.to_string();
                    if value.is_empty() {
                        s
                    } else {
                        Cow::Owned(format!("{s},{value}"))
                    }
                }
            }),
            BuilderDataType::Reference(r) => r.as_ref().to_string(),
            BuilderDataType::SelfReference(w) => {
                if let Some(r) = w.upgrade() {
                    r.as_ref().to_string()
                } else {
                    Cow::Owned(String::new())
                }
            }
            BuilderDataType::Store(r) => r.as_ref().borrow().to_string(),
            BuilderDataType::Take(r) => r.as_ref().borrow_mut().take_one().to_string(),
            BuilderDataType::IfThenElse(v) => {
                if let Ok(r) = BuilderDataType::if_then_else_ref(v) {
                    r.to_string()
                } else {
                    Cow::Owned(String::new())
                }
            }
            _ => Cow::Owned(String::new()),
        }
    }
}

impl<'s, 'r, 'de> serde::Deserializer<'de> for BuilderDeserializerRef<'s, 'r, 'de> {
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
                closure: self.closure,
                data: v.iter(),
                leftover: None,
                size_hint: Some(v.len()),
            }),
            BuilderDataType::List(v) => visitor.visit_seq(BuilderListAccessRef {
                closure: self.closure,
                data: v.iter(),
                index: 0,
                size_hint: Some(v.len()),
            }),
            BuilderDataType::Closure(v) => {
                let mut closure = Closure {
                    args: v.clone(),
                    index: self.closure.index,
                };
                if let Some(r) = v.first() {
                    BuilderDeserializerRef {
                        closure: &mut closure,
                        data: r,
                    }.deserialize_any(visitor)
                } else {
                    Err(BuilderError::InvalidFunctionArgument)
                }
            }
            BuilderDataType::Argument(a) => {
                if let Some(p) = self.closure.args.get(*a).cloned() {
                    BuilderDeserializer {
                        closure: self.closure,
                        data: p,
                    }
                    .deserialize_any(visitor)
                } else {
                    Err(BuilderError::InvalidFunctionArgument)
                }
            }
            BuilderDataType::Reference(r) => BuilderDeserializerRef {
                closure: self.closure,
                data: r.as_ref(),
            }
            .deserialize_any(visitor),
            BuilderDataType::SelfReference(w) => {
                if let Some(r) = w.upgrade() {
                    BuilderDeserializerRef {
                        closure: self.closure,
                        data: r.as_ref(),
                    }
                    .deserialize_any(visitor)
                } else {
                    Err(BuilderError::InvalidSelfRefrence)
                }
            }
            BuilderDataType::Store(r) => BuilderDeserializer {
                closure: self.closure,
                data: r.as_ref().borrow().clone(),
            }
            .deserialize_any(visitor),
            BuilderDataType::Take(r) => BuilderDeserializer {
                closure: self.closure,
                data: r.as_ref().borrow_mut().take_one(),
            }
            .deserialize_any(visitor),
            BuilderDataType::IfThenElse(v) => BuilderDeserializerRef {
                closure: self.closure,
                data: BuilderDataType::if_then_else_ref(v)?,
            }
            .deserialize_any(visitor),
            BuilderDataType::Repeat(v) => {
                let mut it = v.iter();
                let times = it.next().map_or(0, |r| r.to_unsigned());
                visitor.visit_seq(BuilderListAccessRef {
                    closure: self.closure,
                    data: it.cycle().take(times as usize),
                    size_hint: Some(times as usize),
                    index: 0,
                })
            }
            _ => todo!(),
        }
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'s, 'de> serde::Deserializer<'de> for BuilderDeserializer<'s, 'de> {
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
            BuilderDataType::Map(v) => {
                let size_hint = Some(v.len());
                visitor.visit_map(BuilderMapAccess {
                    closure: self.closure,
                    data: v.into_iter(),
                    leftover: None,
                    size_hint,
                })
            }
            BuilderDataType::List(v) => {
                let size_hint = Some(v.len());
                visitor.visit_seq(BuilderListAccess {
                    closure: self.closure,
                    data: v.into_iter(),
                    size_hint,
                    index: 0,
                })
            }
            BuilderDataType::Closure(v) => {
                if let Some(r) = v.first().cloned() {
                    let mut closure = Closure {
                        args: v,
                        index: self.closure.index,
                    };
                    BuilderDeserializer {
                        closure: &mut closure,
                        data: r,
                    }.deserialize_any(visitor)
                } else {
                    Err(BuilderError::InvalidFunctionArgument)
                }
            }
            BuilderDataType::Argument(a) => {
                if let Some(p) = self.closure.args.get(a).cloned() {
                    BuilderDeserializer {
                        closure: self.closure,
                        data: p,
                    }
                    .deserialize_any(visitor)
                } else {
                    Err(BuilderError::InvalidFunctionArgument)
                }
            }
            BuilderDataType::Reference(r) => {
                if Rc::weak_count(&r) > 0 {
                    BuilderDeserializerRef {
                        closure: self.closure,
                        data: &r,
                    }
                    .deserialize_any(visitor)
                } else {
                    match Rc::try_unwrap(r) {
                        Ok(data) => BuilderDeserializer {
                            closure: self.closure,
                            data,
                        }
                        .deserialize_any(visitor),
                        Err(r) => BuilderDeserializerRef {
                            closure: self.closure,
                            data: &r,
                        }
                        .deserialize_any(visitor),
                    }
                }
            }
            BuilderDataType::SelfReference(w) => {
                if let Some(r) = w.upgrade() {
                    BuilderDeserializerRef {
                        closure: self.closure,
                        data: r.as_ref(),
                    }
                    .deserialize_any(visitor)
                } else {
                    Err(BuilderError::InvalidSelfRefrence)
                }
            }
            BuilderDataType::Store(r) => match Rc::try_unwrap(r) {
                Ok(c) => BuilderDeserializer {
                    closure: self.closure,
                    data: c.into_inner(),
                }
                .deserialize_any(visitor),
                Err(r) => BuilderDeserializer {
                    closure: self.closure,
                    data: r.as_ref().borrow().clone(),
                }
                .deserialize_any(visitor),
            },
            BuilderDataType::Take(r) => BuilderDeserializer {
                closure: self.closure,
                data: r.as_ref().borrow_mut().take_one(),
            }
            .deserialize_any(visitor),
            BuilderDataType::IfThenElse(v) => BuilderDeserializer {
                closure: self.closure,
                data: BuilderDataType::if_then_else(v)?,
            }
            .deserialize_any(visitor),
            BuilderDataType::Repeat(v) => {
                let mut it = v.iter();
                let times = it.next().map_or(0, |r| r.to_unsigned());
                visitor.visit_seq(BuilderListAccessRef {
                    closure: self.closure,
                    data: it.cycle().take(times as usize),
                    index: 0,
                    size_hint: Some(times as usize),
                })
            }
            _ => todo!(),
        }
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
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
impl<'s, 'de, I> SeqAccess<'de> for BuilderListAccess<'s, 'de, I>
where
    I: Iterator<Item = BuilderDataType<'de>>,
{
    type Error = BuilderError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(data) = self.data.next() {
            self.closure.index = self.index;
            self.index += 1;
            Ok(Some(seed.deserialize(BuilderDeserializer {
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

pub fn from_data<'a, T>(data: BuilderDataType<'a>) -> Result<T, BuilderError>
where
    T: Deserialize<'a>,
{
    let mut closure = Closure {
        args: Vec::new(),
        index: 0,
    };
    let builder = BuilderDeserializer {
        closure: &mut closure,
        data,
    };

    Ok(T::deserialize(builder)?)
}

pub fn from_ref<'a, T>(data: &BuilderDataType<'a>) -> Result<T, BuilderError>
where
    T: Deserialize<'a>,
{
    let mut closure = Closure {
        args: Vec::new(),
        index: 0,
    };
    let builder = BuilderDeserializerRef {
        closure: &mut closure,
        data,
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
        let data = BuilderDataType::Reference(Rc::new(BuilderDataType::List(vec![
            BuilderDataType::Integer(123),
            BuilderDataType::Boolean(true),
            BuilderDataType::String(Cow::from("test")),
        ])));

        let data = BuilderDataType::Reference(Rc::new(BuilderDataType::Repeat(vec![
            BuilderDataType::Unsigned(3),
            data,
        ])));

        let data = BuilderDataType::List(vec![data, BuilderDataType::Map(vec![])]);

        let test: TestComplex = from_data(data).unwrap();

        assert_eq!(fixture_data_complex(0), test);
    }

    #[test]
    fn test_complex_nested() {
        let data = BuilderDataType::Reference(Rc::new(BuilderDataType::List(vec![
            BuilderDataType::Integer(123),
            BuilderDataType::Boolean(true),
            BuilderDataType::String(Cow::from("test")),
        ])));

        let data = BuilderDataType::Reference(Rc::new(BuilderDataType::Repeat(vec![
            BuilderDataType::Unsigned(3),
            data,
        ])));

        let nest_count = Rc::new(RefCell::new(BuilderDataType::Integer(3)));

        let data = Rc::new_cyclic(|self_reference| {
            BuilderDataType::List(vec![
                data,
                BuilderDataType::IfThenElse(vec![
                    BuilderDataType::Take(nest_count),
                    BuilderDataType::Map(vec![(
                        BuilderDataType::String(Cow::from("test")),
                        BuilderDataType::SelfReference(self_reference.clone()),
                    )]),
                    BuilderDataType::Map(vec![]),
                ]),
            ])
        });

        let test: TestComplex = from_ref(&data).unwrap();

        assert_eq!(fixture_data_complex(3), test);
    }
}
