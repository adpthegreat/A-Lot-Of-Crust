use std;
use std::fmt::{self,Display};

use serde::{de,ser};

pub type Result<T> = std::result::Result<T,Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayComma,
     ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedEnum,
    TrailingCharacters,
}

impl ser::Error for Error {
    fn custom<T>(msg:T) -> Self where T:Display {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T>(msg:T) -> Self where T:Display {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            Error::Syntax => todo!(),
            Error::ExpectedBoolean => todo!(),
            Error::ExpectedInteger => todo!(),
            Error::ExpectedString => todo!(),
            Error::ExpectedNull => todo!(),
            Error::ExpectedArray => todo!(),
            Error::ExpectedArrayComma => todo!(),
            Error::ExpectedArrayEnd => todo!(),
            Error::ExpectedMap => todo!(),
            Error::ExpectedMapColon => todo!(),
            Error::ExpectedMapComma => todo!(),
            Error::ExpectedMapEnd => todo!(),
            Error::ExpectedEnum => todo!(),
            Error::TrailingCharacters => todo!(),
        }
    }
}


impl std::error::Error for Error {}