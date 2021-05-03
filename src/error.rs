#[cfg(feature = "simd")]
use simdutf8::basic::Utf8Error;
#[cfg(not(feature = "simd"))]
use std::string::FromUtf8Error;

/// This enum stores the error types
#[derive(Debug)]
pub enum Error {
    /// Stack is empty (should never happen)
    StackEmpty,
    // InvalidAnsi,
    /// Error parsing the input as utf-8
    Utf8Error,
    /// Cannot determine the foreground or background 
    UnknownLayer,
    /// Error while parsing the color
    ColorParsingError,
    /// Error while paring the ansi sequnce as a usize ( should really be u8 max )
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
