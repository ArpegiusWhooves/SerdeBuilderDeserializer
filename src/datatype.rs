


use std::borrow::Cow;
use std::cell::RefCell;
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
    TakeFromArgument(usize),
    PopArgument,
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

impl<'de> BuilderDataType<'de> {
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
            _ => Cow::Owned(String::new()),
        }
    }
}
