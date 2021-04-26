#[cfg(feature = "simd")]
use simdutf8::basic::Utf8Error;
#[cfg(not(feature = "simd"))]
use std::string::FromUtf8Error;

/// This enum stores the error types
#[derive(Debug)]
pub enum Error {
    StackEmpty,
    // InvalidAnsi,
    Utf8Error,
    UnknownLayer,
    ColorParsingError,
    UsizeParsingError,
}

#[cfg(not(feature = "simd"))]
impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        Error::Utf8Error
    }
}
#[cfg(feature = "simd")]
impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::Utf8Error
    }
}
