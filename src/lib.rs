use std::borrow::Cow;
use std::fmt::Display;
use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor, Error
};


pub enum BuilderDataType<'de> {
    Boolean(bool),
    Integer(i64),
    Number(f64),
    Map( Vec<(Cow<'de, str>,BuilderDataType<'de>)> ),
    String(Cow<'de, str>),
}

#[derive(Debug)]
pub enum BuilderError {

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


pub struct BuilderDeserializer<'de>(&'de BuilderDataType<'de>);

impl<'de> BuilderDeserializer<'de> {
    pub fn from_data(input: &'de BuilderDataType) -> Self {
        BuilderDeserializer(input)
    }
}

impl<'de, 'a> serde::de::Deserializer<'de> for &'a mut BuilderDeserializer<'de> {
    type Error = BuilderError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        match self.0 {
            BuilderDataType::Boolean(v) => todo!(),
            BuilderDataType::Integer(v) => todo!(),
            BuilderDataType::Number(v) => todo!(),
            BuilderDataType::Map(v) => todo!(),
            BuilderDataType::String(v) => todo!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
            todo!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        todo!()
    }
}


pub fn from_data<'a, T>(data: &'a BuilderDataType<'a>) -> Result<T,BuilderError>
where
    T: Deserialize<'a>,
{
    let mut deserializer = BuilderDeserializer::from_data(data);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
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
            (Cow::from("a"),BuilderDataType::Integer(123)),
            (Cow::from("b"),BuilderDataType::Boolean(true)),
            (Cow::from("c"),BuilderDataType::String(Cow::from("test"))),
        ]);

        let test: Test = from_data(&data).unwrap();

        assert_eq!(result, test);
    }
}
