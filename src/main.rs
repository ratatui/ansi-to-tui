#![allow(dead_code)]
mod ansi;
mod code;
mod color;
mod error;
mod stack;
mod style;
mod tests;
use io::{Read, Stdout};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    buffer::Cell,
    layout::Rect,
    widgets::{Paragraph, Widget},
};
use unicode_width::UnicodeWidthStr;

use ansi::ansi_to_text;
pub fn main() {
    let mut file = std::fs::File::open("log").unwrap();
    // let mut file = std::fs::File::open("archlinux.ascii").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    let mut backend = CrosstermBackend::new(io::stdout());
    let mut tmp_buffer = Buffer::empty(Rect::new(0, 0, 500, 500));

    file.read_to_end(&mut buffer).unwrap();
    let _text = ansi_to_text(buffer).unwrap();
    // println!("{:?}", _text);

    Paragraph::new(_text).render(Rect::new(1, 1, 10, 10), &mut tmp_buffer);
    write_buffer_to_console(&mut backend, &mut tmp_buffer).unwrap();
}

fn find_last_buffer_cell_index(buf: &Buffer) -> Option<(u16, u16)> {
    let empty_cell = Cell::default();

    if let Some((idx, _)) = buf
        .content
        .iter()
        .enumerate()
        .filter(|p| !(*(p.1)).eq(&empty_cell))
        .last()
    {
        return Some(buf.pos_of(idx));
    }

    None
}

fn write_buffer_to_console(
    backend: &mut CrosstermBackend<Stdout>,
    tmp_buffer: &mut Buffer,
) -> Result<(), io::Error> {
    let (_, last_y) =
        find_last_buffer_cell_index(tmp_buffer).expect("Error while writing to terminal buffer.");

    print!("{}", "\n".repeat(last_y as usize + 1));

    let mut cursor_y: u16 = 0;

    let term_size = backend.size().unwrap_or_default();
    // We need a checked subtraction here, because (cursor_y - last_y - 1) might underflow if the
    // cursor_y is smaller than (last_y - 1).
    let starting_pos = cursor_y.saturating_sub(last_y).saturating_sub(1);
    let mut skip_n = 0;

    let iter = tmp_buffer
        .content
        .iter()
        .enumerate()
        .filter(|(_previous, cell)| {
            let curr_width = cell.symbol.width();
            if curr_width == 0 {
                return false;
            }

            let old_skip = skip_n;
            skip_n = curr_width.saturating_sub(1);
            old_skip == 0
        })
        .map(|(idx, cell)| {
            let (x, y) = tmp_buffer.pos_of(idx);
            (x, y, cell)
        })
        .filter(|(x, y, _)| *x < term_size.width && *y <= last_y)
        .map(|(x, y, cell)| (x, y + starting_pos, cell));

    backend.draw(iter)?;
    Ok(())
}
