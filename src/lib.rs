mod errors;
pub use errors::BuilderError;
mod datatype;
pub use datatype::BuilderDataType;
mod closure;
pub use closure::Closure;
mod deserialize;
pub use deserialize::BuilderDeserializer;
mod deserialize_ref;
pub use deserialize_ref::BuilderDeserializerRef;
mod list_access;
pub use list_access::BuilderListAccess;
mod list_access_ref;
pub use list_access_ref::BuilderListAccessRef;
mod map_access;
pub use map_access::BuilderMapAccess;
mod map_access_ref;
pub use map_access_ref::BuilderMapAccessRef;
use serde::Deserialize;

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
mod tests;
