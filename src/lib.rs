#![allow(unused_imports)]
#![warn(missing_docs)]

//! Convert ANSI color and style codes into Ratatui [`Text`][Text].
//!
//! This crate parses bytes containing ANSI SGR escape sequences (like `\x1b[31m`).
//! It produces a Ratatui [`Text`][Text] with equivalent foreground/background [`Color`][Color] and
//! [`Modifier`][Modifier] settings via [`Style`][Style].
//!
//! Unknown or malformed escape sequences are ignored, so you can feed it real terminal output
//! without having to pre-clean it.
//!
//! # Features
//!
//! - UTF-8 decoding via `String::from_utf8` (default) or [`simdutf8`][simdutf8] (`simd` feature).
//! - SGR styles such as bold, italic, underline, and strikethrough.
//! - Colors: named (3/4-bit, 8/16-color), indexed (8-bit, 256-color), and truecolor (24-bit RGB).
//! - Optional `zero-copy` API that borrows from the input.
//!
//! # Supported Color Codes
//!
//! | Color Mode                  | Supported | SGR Example              | Ratatui `Color` Example |
//! | --------------------------- | :-------: | ------------------------ | ----------------------- |
//! | Named (3/4-bit, 8/16-color) |     ✓     | `\x1b[30..37;40..47m`    | `Color::Blue`           |
//! | Indexed (8-bit, 256-color)  |     ✓     | `\x1b[38;5;<N>m`         | `Color::Indexed(1)`     |
//! | Truecolor (24-bit RGB)      |     ✓     | `\x1b[38;2;<R>;<G>;<B>m` | `Color::Rgb(255, 0, 0)` |
//!
//! The SGR examples above set the foreground color (`38`). For background colors, replace `38`
//! with `48` (for example, `\x1b[48;5;<N>m` and `\x1b[48;2;<R>;<G>;<B>m`).
//!
//! # Example
//!
//! The input type implements `AsRef<[u8]>`, so it is not consumed.
//!
//! ```rust
//! # fn doctest() -> eyre::Result<()> {
//! use ansi_to_tui::IntoText as _;
//! let bytes = b"\x1b[38;2;225;192;203mAAAAA\x1b[0m".to_vec();
//! let text = bytes.into_text()?;
//! # Ok(()) }
//! ```
//!
//! Parsing from a file.
//!
//! ```rust
//! # fn doctest() -> eyre::Result<()> {
//! use ansi_to_tui::IntoText as _;
//! let buffer = std::fs::read("ascii/text.ascii")?;
//! let text = buffer.into_text()?;
//! # Ok(()) }
//! ```
//!
//! [Text]: https://docs.rs/ratatui-core/latest/ratatui_core/text/struct.Text.html
//! [Color]: https://docs.rs/ratatui-core/latest/ratatui_core/style/enum.Color.html
//! [Style]: https://docs.rs/ratatui-core/latest/ratatui_core/style/struct.Style.html
//! [Modifier]: https://docs.rs/ratatui-core/latest/ratatui_core/style/struct.Modifier.html
//! [simdutf8]: https://github.com/rusticstuff/simdutf8

// mod ansi;
mod code;
mod error;
mod parser;
pub use error::Error;
use ratatui_core::text::Text;

/// Parse ANSI SGR styled bytes into a Ratatui [`Text`].
///
/// This trait is implemented for all `T: AsRef<[u8]>`, so most byte containers can call
/// [`IntoText::into_text`]. With the `zero-copy` feature enabled, you can also call
/// [`IntoText::to_text`].
///
/// For example, `String`, `&str`, `Vec<u8>`, and `&[u8]` all implement `AsRef<[u8]>`.
///
/// You may also implement this trait for your own types if you want custom conversions.
///
/// # Example
///
/// ```rust
/// use ansi_to_tui::IntoText as _;
///
/// let s: &str = "\x1b[34mblue\x1b[0m";
/// let _text = s.into_text()?;
///
/// let owned: String = "\x1b[31mred\x1b[0m".to_owned();
/// let _text = owned.into_text()?;
/// # Ok::<(), ansi_to_tui::Error>(())
/// ```
pub trait IntoText {
    /// Convert the type to an owned `Text`.
    ///
    /// This always returns a `Text<'static>`, so it allocates owned strings for the parsed spans.
    #[allow(clippy::wrong_self_convention)]
    fn into_text(&self) -> Result<Text<'static>, Error>;

    /// Convert the type to a borrowed `Text` while trying to copy as little as possible.
    ///
    /// This method borrows the span contents from the input instead of allocating new strings,
    /// so the returned `Text` is only valid as long as the input is alive.
    ///
    /// Use this when you only need the parsed `Text` temporarily (for example, render it
    /// immediately). If you need to store the result beyond the lifetime of the input, use
    /// [`IntoText::into_text`] instead.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[cfg(feature = "zero-copy")]
    /// # {
    /// use ansi_to_tui::IntoText as _;
    ///
    /// let bytes = b"\x1b[32mgreen\x1b[0m";
    /// let _text = bytes.to_text()?;
    /// # }
    /// # Ok::<(), ansi_to_tui::Error>(())
    /// ```
    #[cfg(feature = "zero-copy")]
    fn to_text(&self) -> Result<Text<'_>, Error>;
}

/// Blanket implementation for all `AsRef<[u8]>` types.
impl<T> IntoText for T
where
    T: AsRef<[u8]>,
{
    fn into_text(&self) -> Result<Text<'static>, Error> {
        Ok(crate::parser::text(self.as_ref())?.1)
    }

    #[cfg(feature = "zero-copy")]
    fn to_text(&self) -> Result<Text<'_>, Error> {
        Ok(crate::parser::text_fast(self.as_ref())?.1)
    }
}
