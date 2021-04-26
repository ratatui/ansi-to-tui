use crate::error::Error;
use crate::stack::{AnsiGraphicsStack, Stack};
#[cfg(feature = "simd")]
use simdutf8::basic::from_utf8;
use tui::{
    style::Style,
    text::{Span, Spans, Text},
};

/// This functions converts the ascii byte sequence with ansi colors to tui::text::Text type  
///
/// Example
/// ```rust
/// use ansi_to_tui::ansi_to_text;
/// let bytes : Vec<u8> = vec![b'\x1b', b'[', b'3', b'1', b'm', b'A', b'A', b'A', b'\x1b', b'[', b'0'];
/// let text = ansi_to_text(&bytes);
/// ```
///
pub fn ansi_to_text<'t, B: AsRef<[u8]>>(bytes: B) -> Result<Text<'t>, Error> {
    let reader = bytes.as_ref().iter().copied(); // copies the whole buffer to memory

    let mut buffer: Vec<Spans> = Vec::new();
    let mut span_buffer: Vec<Span> = Vec::new();
    let mut style: Style = Style::default();

    let mut ansi_stack: AnsiGraphicsStack = AnsiGraphicsStack::new();
    let mut stack: Stack<u8> = Stack::new();

    let mut line_buffer: Vec<u8> = Vec::new();

    let mut last_byte = 0_u8;

    for byte in reader {
        // let byte_char = char::from(byte);

        if ansi_stack.is_unlocked() && last_byte == b'\x1b' && byte != b'[' {
            // if byte after \x1b was not [ lock the stack
            ansi_stack.lock();
        }
        // if ansi_stack.is_locked() && UnicodeWidthChar::width(byte_char).is_some() {
        if ansi_stack.is_locked() && byte != b'\n' && byte != b'\x1b' {
            line_buffer.push(byte)
        } else {
            match byte {
                b'\x1b' => {
                    if !line_buffer.is_empty() {
                        span_buffer.push(Span::styled(
                            #[cfg(feature = "simd")]
                            from_utf8(&line_buffer.clone())?.to_owned(),
                            #[cfg(not(feature = "simd"))]
                            String::from_utf8(line_buffer.clone())?,
                            style,
                        ));
                        line_buffer.clear();
                    }

                    ansi_stack.unlock();
                    // ansi_stack.push(byte as usize);
                } // this clears the stack

                b'\n' => {
                    if !span_buffer.is_empty() {
                        buffer.push(Spans::from(span_buffer.clone()));
                        span_buffer.clear();
                    }
                }

                b';' => ansi_stack.push(stack.parse_usize()?),

                b'0'..=b'9' => stack.push(byte),

                b'm' => {
                    ansi_stack.push(stack.parse_usize()?);
                    style = ansi_stack.parse_ansi()?;

                    // lock after parse since lock will clear
                    ansi_stack.lock();
                }
                b'[' => (),
                _ => {
                    // anything other than numbers or ; or newline or 'm' will lock the stack
                    ansi_stack.lock();
                }
            }
        }
        last_byte = byte;
    }

    if !line_buffer.is_empty() {
        span_buffer.push(Span::styled(
            #[cfg(feature = "simd")]
            from_utf8(&line_buffer.clone())?.to_owned(),
            #[cfg(not(feature = "simd"))]
            String::from_utf8(line_buffer.clone())?,
            style,
        ));
        line_buffer.clear();
    }
    if !span_buffer.is_empty() {
        buffer.push(Spans::from(span_buffer.clone()));
        span_buffer.clear();
    }

    Ok(buffer.into())
}
