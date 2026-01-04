/// Errors returned by this crate.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    /// Parsing failed.
    ///
    /// This is currently a formatted representation of a `nom` parse error.
    #[error("Parse error: {0}")]
    NomError(String),

    /// The input contains invalid UTF-8.
    #[cfg(feature = "simd")]
    #[error(transparent)]
    Utf8Error(#[from] simdutf8::basic::Utf8Error),

    /// The input contains invalid UTF-8.
    #[cfg(not(feature = "simd"))]
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for Error {
    fn from(e: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Self::NomError(format!("{:?}", e))
    }
}
