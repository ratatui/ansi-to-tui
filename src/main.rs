mod ansi;
mod code;
mod color;
mod error;
mod stack;
mod style;
mod tests;
use std::io::Read;

use ansi::ansi_to_text;
pub fn main() {
    // let mut file = std::fs::File::open("log").unwrap();
    let mut file = std::fs::File::open("archlinux.ascii").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let _text = ansi_to_text(buffer);
}
