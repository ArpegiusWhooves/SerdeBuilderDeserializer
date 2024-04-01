


use std::borrow::Cow;
use std::rc::Rc;

use crate::{BuilderDataType, BuilderDeserializerRef, BuilderError, BuilderListAccess, BuilderListAccessRef, BuilderMapAccess, Closure};
use serde::de::Visitor;
use serde::forward_to_deserialize_any;

pub struct BuilderDeserializer<'s, 'de> {
    pub(crate) closure: &'s mut Closure<'de>,
    pub(crate) data: BuilderDataType<'de>,
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
                        args: v.into_iter().map(|a|self.closure.resolve(a)).collect::<Result<Vec<_>, _>>()?,
                        index: self.closure.index,
                    };
                    BuilderDeserializer {
                        closure: &mut closure,
                        data: r,
                    }
                    .deserialize_any(visitor)
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
            BuilderDataType::TakeFromArgument(a) => {
                if let Some(p) = self.closure.args.get_mut(a).map(|r| r.take_one()) {
                    BuilderDeserializer {
                        closure: self.closure,
                        data: p,
                    }
                    .deserialize_any(visitor)
                } else {
                    Err(BuilderError::InvalidFunctionArgument)
                }
            }
            BuilderDataType::PopArgument => {
                if let Some(p) = self.closure.args.pop() {
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
                data: self.closure.if_then_else(v)?,
                closure: self.closure,
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
            BuilderDataType::Index => visitor.visit_u64(self.closure.index as u64),
            _ => todo!(),
        }
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
