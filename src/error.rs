/// This enum stores the error types
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// Stack is empty (should never happen)
    #[error("Nom Error")]
    NomError(String),

    /// Error parsing the input as utf-8
    #[cfg(feature = "simdutf8")]
    /// Cannot determine the foreground or background
    #[error("{0:?}")]
    Utf8Error(#[from] simdutf8::basic::Utf8Error),

    #[cfg(not(feature = "simdutf8"))]
    /// Cannot determine the foreground or background
    #[error("{0:?}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for Error {
    fn from(e: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::NomError(format!("{:?}", e))
    }
}
