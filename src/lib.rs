//! Parses a `Vec<u8>` as an byte sequence with ansi colors to
//! [`tui::text::Text`](https://docs.rs/tui/0.14.0/tui/text/struct.Text.html).  
//!
//! Invalid ansi colors / sequences will be ignored.  
//!
//!
//! Supported features
//! - UTF-8 using `String::from_utf8` or [`simdutf8`](https://github.com/rusticstuff/simdutf8).
//! - Most stuff like **Bold** / *Italic* / <u>Underline</u> / ~~Striketrhough~~.
//! - Supports 4-bit color palletes.
//! - Supports 8-bit color.
//! - Supports True color ( RGB / 24-bit color ).
//!
//! ## Example
//! ```rust
//! use ansi_to_tui::ansi_to_text;
//! let bytes = b"\x1b[38;2;225;192;203mAAAAA\x1b[0m";
//! let text = ansi_to_text(&bytes).unwrap();
//! ```
//! You can use this text in a [tui](https://docs.rs/tui/) application.
//!
//! ## Cargo.toml
//!
//! ```toml
//! [dependencies]
//! ansi_to_tui = { git = "https://github.com/uttarayan21/ansi-to-tui" }
//! ```
//! If you want to use [`simdutf8`](https://github.com/rusticstuff/simdutf8) instead of `String::from_utf8()`  
//! for parsing UTF-8 then enable optional feature `simd`  

mod ansi;
mod code;
mod color;
mod error;
mod stack;

pub use ansi::ansi_to_text;
pub use code::AnsiCode;
pub use color::AnsiColor;
pub use error::Error;
