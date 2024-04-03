use crate::{
    BuilderDataType, BuilderDeserializer, BuilderError, BuilderListAccessRef, BuilderMapAccessRef,
    Closure,
};
use serde::de::Visitor;
use serde::forward_to_deserialize_any;
use std::borrow::Cow;

pub struct BuilderDeserializerRef<'s, 'r, 'de> {
    pub(crate) closure: &'s mut Closure<'de>,
    pub(crate) data: &'r BuilderDataType<'de>,
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
                    args: v
                        .iter()
                        .map(|a| self.closure.resolve_clone(a))
                        .collect::<Result<Vec<_>, _>>()?,
                    index: self.closure.index,
                };
                if let Some(r) = v.first() {
                    BuilderDeserializerRef {
                        closure: &mut closure,
                        data: r,
                    }
                    .deserialize_any(visitor)
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
            BuilderDataType::TakeFromArgument(a) => BuilderDeserializer {
                data: self.closure.take_from_argument(*a)?,
                closure: self.closure,
            }
            .deserialize_any(visitor),
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
                data: self.closure.if_then_else_ref(v)?,
                closure: self.closure,
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
