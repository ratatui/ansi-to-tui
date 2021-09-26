use crate::error::Error;
use crate::stack::{AnsiGraphicsStack, Stack};
#[cfg(feature = "simd")]
use simdutf8::basic::from_utf8;
use tui::{
    style::Style,
    text::{Span, Spans, Text},
};

/// This functions converts the ascii byte sequence with ansi colors to [tui::text::Text][Text] type  
/// This functions's argument implements into_iter so the buffer will be consumed on use.
///
/// Example
/// ```rust
/// use ansi_to_tui::ansi_to_text;
/// let bytes : Vec<u8> = vec![b'\x1b', b'[', b'3', b'1', b'm', b'A', b'A', b'A', b'\x1b', b'[', b'0'];
/// let text = ansi_to_text(bytes);
/// ```
/// Example parsing from a file.
/// ```rust
/// use ansi_to_tui::ansi_to_text;
/// use std::io::Read;
///
/// let mut file = std::fs::File::open("ascii/text.ascii").unwrap();
/// let mut buffer: Vec<u8> = Vec::new();
/// file.read_to_end(&mut buffer);
/// let text = ansi_to_text(buffer);
/// ```
/// [Text]: https://docs.rs/tui/0.15.0/tui/text/struct.Text.html
///
pub fn ansi_to_text<'t, B: IntoIterator<Item = u8>>(bytes: B) -> Result<Text<'t>, Error> {
    // let reader = bytes.as_ref().iter().copied(); // copies the whole buffer to memory
    let reader = bytes.into_iter();
    // let _read = bytes.as_ref().into_iter();

    let mut buffer: Vec<Spans> = Vec::new();
    let mut line_buffer: Vec<u8> = Vec::new(); // this contains all the text in a single styled ( including utf-8 )
    let mut line_styled_buffer: Vec<u8> = Vec::new(); //this is used to store the text while style is being processed.
    let mut span_buffer: Vec<Span> = Vec::new(); // this contains text with a style and there maybe multiple per line

    let mut style: Style = Style::default();

    let mut stack: Stack<u8> = Stack::new();
    let mut ansi_stack: AnsiGraphicsStack = AnsiGraphicsStack::new();
    let mut style_stack: Stack<Style> = Stack::new();

    // style_stack.push(style);

    let mut last_byte = 0_u8;

    for byte in reader {
        // let byte_char = char::from(byte);

        if ansi_stack.is_unlocked() && last_byte == b'\x1b' && byte != b'[' {
            // if byte after \x1b was not [ lock the stack
            ansi_stack.lock();
        }

        if ansi_stack.is_locked() && byte != b'\n' && byte != b'\x1b' {
            if line_styled_buffer.is_empty()
                && !line_buffer.is_empty()
                && style_stack.last() != Some(&style)
            {
                span_buffer.push(Span::styled(
                    #[cfg(feature = "simd")]
                    from_utf8(&line_buffer)?.to_owned(),
                    #[cfg(not(feature = "simd"))]
                    String::from_utf8(line_buffer.clone())?,
                    style_stack.pop().unwrap(),
                ));
                line_buffer.clear();
                style_stack.push(style);
            }
            line_styled_buffer.push(byte);
        } else {
            match byte {
                b'\x1b' => {
                    if !line_styled_buffer.is_empty() {
                        line_buffer.append(&mut line_styled_buffer);
                        line_styled_buffer.clear();
                    }
                    ansi_stack.unlock();
                } // this clears the stack

                b'\n' => {
                    // If line buffer is not empty when a newline is detected push the line_buffer
                    // to the span_buffer since we need the spans.
                    if !line_styled_buffer.is_empty() {
                        line_buffer.append(&mut line_styled_buffer);
                        line_styled_buffer.clear();
                    }

                    if !line_buffer.is_empty() {
                        span_buffer.push(Span::styled(
                            #[cfg(feature = "simd")]
                            from_utf8(&line_buffer)?.to_owned(),
                            #[cfg(not(feature = "simd"))]
                            String::from_utf8(line_buffer.clone())?,
                            style,
                        ));
                        line_buffer.clear();
                        // style_stack.push(style);
                    }

                    if !span_buffer.is_empty() {
                        buffer.push(Spans::from(span_buffer.clone()));
                        span_buffer.clear();
                    } else {
                        buffer.push(Spans::default())
                    }
                    span_buffer.clear();
                }

                b';' => ansi_stack.push(stack.parse_usize()?),

                b'0'..=b'9' => stack.push(byte),

                b'm' => {
                    ansi_stack.push(stack.parse_usize()?);
                    // patch since the last style is not overwritten, only modified with a new
                    // sequence.

                    let _style_new = ansi_stack.parse_ansi()?;

                    if _style_new == Style::default() {
                        style = _style_new;
                    } else {
                        // style.patch doesn't work for Style::default() for some reason.
                        style = style.patch(_style_new);
                    }

                    if style_stack.is_empty() {
                        style_stack.push(style);
                    }
                    // lock after parse since lock will clear
                    ansi_stack.lock();
                }

                b'[' => (),

                _ => {
                    // any unexpected sequence will cause the ansi graphics stack to lock up
                    ansi_stack.lock();
                }
            }
        }
        last_byte = byte;
    }

    if !line_styled_buffer.is_empty() {
        line_buffer.append(&mut line_styled_buffer);
        line_styled_buffer.clear();
    }

    if !line_buffer.is_empty() {
        span_buffer.push(Span::styled(
            #[cfg(feature = "simd")]
            from_utf8(&line_buffer)?.to_owned(),
            #[cfg(not(feature = "simd"))]
            String::from_utf8(line_buffer.clone())?,
            style,
        ));
        line_buffer.clear();
    }
    if !span_buffer.is_empty() {
        buffer.push(Spans::from(span_buffer));
        // span_buffer.clear();
    }

    Ok(buffer.into())
}

/// Same as ansi_to_text but with a custom override style for the whole text.
pub fn ansi_to_text_override_style<'t, B: IntoIterator<Item = u8>>(
    bytes: B,
    style: tui::style::Style,
) -> Result<Text<'t>, Error> {
    let reader = bytes.into_iter();

    let mut buffer: Vec<Spans> = Vec::new();
    let mut line_buffer: Vec<u8> = Vec::new();

    let mut parsing_ansi_code: bool = false;
    let mut last_byte: u8 = 0_u8;

    for byte in reader {
        if parsing_ansi_code && last_byte == b'\x1b' && byte != b'[' {
            parsing_ansi_code = false;
        }
        if !parsing_ansi_code && byte != b'\n' && byte != b'\x1b' {
            line_buffer.push(byte);
        } else {
            match byte {
                b'\x1b' => {
                    parsing_ansi_code = true;
                }
                b'0'..=b'9' | b';' => (),
                b'\n' => {
                    if !line_buffer.is_empty() {
                        buffer.push(Spans::from(Span::styled(
                            #[cfg(feature = "simd")]
                            from_utf8(&line_buffer)?.to_owned(),
                            #[cfg(not(feature = "simd"))]
                            String::from_utf8(line_buffer.clone())?,
                            style,
                        )));
                        line_buffer.clear();
                    } else {
                        buffer.push(Spans::default());
                    }
                }
                b'[' => (),
                _ => {
                    parsing_ansi_code = false;
                }
            }
        }
        last_byte = byte;
    }
    if !line_buffer.is_empty() {
        buffer.push(Spans::from(Span::styled(
            #[cfg(feature = "simd")]
            from_utf8(&line_buffer)?.to_owned(),
            #[cfg(not(feature = "simd"))]
            String::from_utf8(line_buffer.clone())?,
            style,
        )));
        line_buffer.clear();
    }
    Ok(buffer.into())
}
