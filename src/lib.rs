use std::borrow::Cow;
use std::fmt::Display;
use serde::{forward_to_deserialize_any, Deserialize};
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor, Error
};


pub enum BuilderDataType<'de> {
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(Cow<'de, str>),
    Map( Vec<(BuilderDataType<'de>,BuilderDataType<'de>)> ),
    List( Vec<BuilderDataType<'de>> ),
}

#[derive(Debug)]
pub enum BuilderError {
    InvalidMapAccess
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for BuilderError { 
}

impl Error for BuilderError {
    fn custom<T>(msg:T) -> Self where T:Display {
        todo!()
    }
}
pub struct BuilderDeserializer<'de>(BuilderDataType<'de>);

impl<'de> BuilderDeserializer<'de> {
    pub fn from_data(input: BuilderDataType<'de>) -> Self {
        BuilderDeserializer(input)
    }
}

struct BuilderMapAccess<'de> {
    data: Vec<(BuilderDataType<'de>,BuilderDataType<'de>)>,
    leftover: Option<BuilderDataType<'de>>,
}

impl<'de, 'a> serde::Deserializer<'de> for BuilderDeserializer<'de> {
    type Error = BuilderError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        match self.0 {
            BuilderDataType::Boolean(v) => visitor.visit_bool(v),
            BuilderDataType::Integer(v) => visitor.visit_i64(v),
            BuilderDataType::Number(v) => visitor.visit_f64(v),
            BuilderDataType::String(c) => match c {
                Cow::Borrowed(v) => visitor.visit_borrowed_str(v),
                Cow::Owned(v) => visitor.visit_string(v),
            },
            BuilderDataType::Map(v) => visitor.visit_map(BuilderMapAccess{
                data: v,
                leftover: None
            }),
            BuilderDataType::List(v) => todo!(),
        }
    }
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}


impl<'de> MapAccess<'de> for BuilderMapAccess<'de> {
    type Error = BuilderError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de> {
        if let Some((a,b)) = self.data.pop() {
            self.leftover = Some(b); 
            let v = seed.deserialize(BuilderDeserializer::from_data(a))?;
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de> {
        if let Some(leftover) = self.leftover.take() {
            seed.deserialize(BuilderDeserializer::from_data(leftover))
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
        if let Some((a,b)) = self.data.pop() {
            self.leftover = None;
            let va  = kseed.deserialize(BuilderDeserializer::from_data(a))?;
            let vb  = vseed.deserialize(BuilderDeserializer::from_data(b))?;
            Ok(Some((va,vb)))
        } else {
            Ok(None)
        }
    }
    
}


pub fn from_data<'a, T>(data: BuilderDataType<'a>) -> Result<T,BuilderError>
where
    T: Deserialize<'a>,
{ 
    Ok(T::deserialize(BuilderDeserializer::from_data(data))?)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Debug,PartialEq, Eq,Deserialize)]
    struct Test{
        a: i32,
        b: bool,
        c: String
    }
    
    #[test]
    fn it_works() {

        let j = r#"{
            "a": 123,
            "b": true,
            "c": "test"
        }"#;

        let result: Test = serde_json::from_str(j).unwrap();
        println!("{:?}",&result);

        let data = BuilderDataType::Map(vec![
            (BuilderDataType::String(Cow::from("a")),BuilderDataType::Integer(123)),
            (BuilderDataType::String(Cow::from("b")),BuilderDataType::Boolean(true)),
            (BuilderDataType::String(Cow::from("c")),BuilderDataType::String(Cow::from("test"))),
        ]);

        let test: Test = from_data(data).unwrap();

        assert_eq!(result, test);
    }
}
