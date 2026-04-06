mod line;
mod paragraph;
mod span;
mod text;

use std::borrow::Cow;

pub use line::HyperlinkedLine;
pub use paragraph::HyperlinkedParagraph;
pub use span::HyperlinkedSpan;

pub use text::HyperlinkedText;

use crate::parser::AnsiStates;

#[derive(Eq, PartialEq, Hash)]
pub struct Hyperlink<'a, T: ?Sized + ToOwned + 'a> {
    pub text: Cow<'a, T>,
    pub url: Cow<'a, T>,
}

impl<T> core::fmt::Debug for Hyperlink<'_, T>
where
    T: ?Sized + ToOwned + core::fmt::Debug,
    <T as ToOwned>::Owned: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Hyperlink")
            .field("text", &self.text)
            .field("url", &self.url)
            .finish()
    }
}

impl<T> Clone for Hyperlink<'_, T>
where
    T: ?Sized + ToOwned,
    <T as ToOwned>::Owned: Clone,
{
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            url: self.url.clone(),
        }
    }
}

impl<'a, T: ?Sized + ToOwned + 'a> Hyperlink<'a, T> {
    pub fn new<U: Into<Cow<'a, T>>>(text: U, url: U) -> Self {
        Self {
            text: text.into(),
            url: url.into(),
        }
    }
}

impl<'a> Hyperlink<'a, [u8]> {
    #[cfg(not(feature = "simd"))]
    pub fn parse(self) -> Result<Hyperlink<'a, str>, std::str::Utf8Error> {
        let text = match self.text {
            Cow::Borrowed(bytes) => Cow::Borrowed(std::str::from_utf8(bytes)?),
            Cow::Owned(bytes) => Cow::Owned(std::str::from_utf8(&bytes)?.to_owned()),
        };
        let url = match self.url {
            Cow::Borrowed(bytes) => Cow::Borrowed(std::str::from_utf8(bytes)?),
            Cow::Owned(bytes) => Cow::Owned(std::str::from_utf8(&bytes)?.to_owned()),
        };
        Ok(Hyperlink::new(text, url))
    }

    #[cfg(feature = "simd")]
    pub fn parse(self) -> Result<Hyperlink<'a, str>, simdutf8::basic::Utf8Error> {
        let text = match self.text {
            Cow::Borrowed(bytes) => Cow::Borrowed(simdutf8::basic::from_utf8(bytes)?),
            Cow::Owned(bytes) => Cow::Owned(simdutf8::basic::from_utf8(&bytes)?.to_owned()),
        };
        let url = match self.url {
            Cow::Borrowed(bytes) => Cow::Borrowed(simdutf8::basic::from_utf8(bytes)?),
            Cow::Owned(bytes) => Cow::Owned(simdutf8::basic::from_utf8(&bytes)?.to_owned()),
        };
        Ok(Hyperlink::new(text, url))
    }
}
