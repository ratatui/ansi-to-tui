#![allow(dead_code, unused_mut, unused_imports)]
use crate::code::AnsiCode;
use crate::color::AnsiColor;
use crate::error::Error;
use crate::stack::Stack;
use std::slice::Iter;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
};

impl Stack<u8> {
    pub fn parse_usize(&mut self) -> usize {
        let mut num: usize = 0 as usize;
        for n in self.iter() {
            // num = num * 10 + (n.saturating_sub(48_u8)) as usize
            num = num * 10 + (n - 48_u8) as usize
        }
        self.clear();
        num
    }
    pub fn parse_color(&mut self) -> Result<Color, Error> {
        let mut color: Color;
        let length = self.len();
        if length == 1 {
            color = Color::Indexed(self.pop().expect("Shouldn't happen len check in place"))
        } else if length == 3 {
            let b = self.pop().unwrap();
            let g = self.pop().unwrap();
            let r = self.pop().unwrap();
            color = Color::Rgb(r, g, b);
        } else {
            return Err(Error::UnknownColor);
        }
        self.clear();
        Ok(color)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum AnsiColorMode {
    RGB = 2,
    Indexed = 5,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum AnsiColorLayer {
    Background,
    Foreground,
}

#[derive(Debug)]
pub struct AnsiGraphicsStack {
    stack: Stack<usize>,
}

impl AnsiGraphicsStack {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }
    pub fn push(&mut self, sequence: usize) {
        self.stack.push(sequence);
    }
    pub fn iter(&mut self) -> Iter<usize> {
        self.stack.iter()
    }
    pub fn parse_ansi(&mut self) -> Style {
        let mut style = Style::default();
        let mut color_stack: Stack<u8> = Stack::new();
        let mut color_parse: bool = false;
        let mut color_parse_mode: Option<AnsiColorMode> = None;
        let mut last_sequence: usize = 0;
        let mut color_layer: Option<AnsiColorLayer> = None;
        for sequence in self.iter().cloned() {
            if color_parse {
                if sequence < 255 {
                    if AnsiCode::from(last_sequence as u8) == AnsiCode::ForegroundColorIndex
                        || AnsiCode::from(last_sequence as u8) == AnsiCode::BackgroundColorIndex
                        || sequence == AnsiColorMode::RGB as usize
                    {
                        color_parse_mode = Some(AnsiColorMode::RGB)
                    } else if AnsiCode::from(last_sequence as u8) == AnsiCode::ForegroundColorIndex
                        || AnsiCode::from(last_sequence as u8) == AnsiCode::BackgroundColorIndex
                        || sequence == AnsiColorMode::Indexed as usize
                    {
                        color_parse_mode = Some(AnsiColorMode::Indexed)
                    } else if color_parse_mode.is_some() {
                        let mode = color_parse_mode.unwrap();
                        match mode {
                            AnsiColorMode::Indexed => {
                                color_stack.push(sequence as u8);
                                match color_layer.unwrap() {
                                    AnsiColorLayer::Foreground => {
                                        style = style.fg(color_stack.parse_color().unwrap())
                                    }
                                    AnsiColorLayer::Background => {
                                        style = style.bg(color_stack.parse_color().unwrap())
                                    }
                                }
                            }
                            AnsiColorMode::RGB => {
                                color_stack.push(sequence as u8);
                                if color_stack.len() == 3 {
                                    match color_layer.unwrap() {
                                        AnsiColorLayer::Foreground => {
                                            style = style.fg(color_stack.parse_color().unwrap())
                                        }
                                        AnsiColorLayer::Background => {
                                            style = style.bg(color_stack.parse_color().unwrap())
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        color_parse = false;
                    }
                } else {
                    color_parse = false;
                }
                last_sequence = sequence;
                continue;
            }
            let code = AnsiCode::from(sequence as u8);
            match code {
                AnsiCode::Reset => style = Style::default(),
                AnsiCode::Bold => style = style.add_modifier(Modifier::BOLD),
                AnsiCode::Faint => style = style.add_modifier(Modifier::DIM),
                AnsiCode::Italic => style = style.add_modifier(Modifier::ITALIC),
                AnsiCode::Underline => style = style.add_modifier(Modifier::UNDERLINED),
                AnsiCode::SlowBlink => style = style.add_modifier(Modifier::SLOW_BLINK),
                AnsiCode::RapidBlink => style = style.add_modifier(Modifier::RAPID_BLINK),
                AnsiCode::Reverse => style = style.add_modifier(Modifier::REVERSED),
                AnsiCode::Conceal => style = style.add_modifier(Modifier::HIDDEN),
                AnsiCode::CrossedOut => style = style.add_modifier(Modifier::CROSSED_OUT),
                AnsiCode::DefaultForegroundColor => style = style.fg(Color::Reset),
                AnsiCode::DefaultBackgroundColor => style = style.bg(Color::Reset),
                AnsiCode::ForegroundColorIndex => {
                    color_parse = true;
                    color_layer = Some(AnsiColorLayer::Foreground)
                }
                AnsiCode::BackgroundColorIndex => {
                    color_parse = true;
                    color_layer = Some(AnsiColorLayer::Background)
                }
                AnsiCode::ForegroundColor(color) => style = style.fg(Color::from(color)),
                AnsiCode::BackgroundColor(color) => style = style.bg(Color::from(color)),
                _ => (),
            }
            last_sequence = sequence;
        }
        self.stack.clear();
        style
    }
}

pub fn ansi_to_text<'t, B: AsRef<[u8]>>(bytes: B) -> Result<Text<'t>, Error> {
    let mut reader = bytes.as_ref().iter();
    let mut buffer: Vec<Spans> = Vec::new();
    let mut style: Option<Style> = None;
    let mut ansi_stack: AnsiGraphicsStack = AnsiGraphicsStack::new();
    let mut num_stack: Stack<u8> = Stack::new();
    let mut graphics_start: bool = false;
    let mut spans_buffer: Vec<Span> = Vec::new();
    let mut line_buffer = String::new();
    // let mut last_byte: &u8 = reader.next().expect("Zero size bytes buffer");
    let mut last_byte: &u8 = &0_u8;
    for byte in reader {
        match (last_byte, byte) {
            (&b'\x1b', &b'[') => {
                if style.is_some() {
                    spans_buffer.push(Span::styled(line_buffer.clone(), style.unwrap()));
                    line_buffer.clear();
                }
                graphics_start = true;
            }

            (_, &b'\n') => {
                buffer.push(Spans::from(spans_buffer.clone()));
                spans_buffer.clear();
            }
            (_, code) => {
                if graphics_start {
                    if code == &b'm' {
                        ansi_stack.push(num_stack.parse_usize());
                        style = Some(ansi_stack.parse_ansi());
                        graphics_start = false;
                    } else if code == &b';' {
                        ansi_stack.push(num_stack.parse_usize());
                    } else {
                        num_stack.push(*code);
                    }
                } else if code != &b'\x1b' {
                    line_buffer.push(*code as char)
                }
            }
        }
        last_byte = byte;
    }
    if !spans_buffer.is_empty() {
        buffer.push(Spans::from(spans_buffer.clone()));
        spans_buffer.clear();
    }

    Ok(buffer.into())
}
