// #![allow(
//     dead_code,
//     unused_mut,
//     unused_imports,
//     unused_must_use,
//     unused_variables
// )]
mod ansi;
mod code;
mod color;
mod error;
mod stack;

pub use ansi::ansi_to_text;
pub use code::AnsiCode;
pub use color::AnsiColor;
