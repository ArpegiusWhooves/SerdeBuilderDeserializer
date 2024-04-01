

use std::fmt::Display;

use serde::de::Error;

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
