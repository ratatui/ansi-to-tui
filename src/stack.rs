use crate::code::AnsiCode;
use crate::error::Error;
use std::{convert::TryInto, slice::Iter};
use tui::style::{Color, Modifier, Style};

#[derive(Debug)]
pub struct Stack<T> {
    st: Vec<T>,
    lock: bool,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            st: Vec::<T>::new(),
            lock: true,
        }
    }
    pub fn push(&mut self, value: T) {
        self.st.push(value);
    }
    pub fn pop(&mut self) -> Option<T> {
        self.st.pop()
    }
    pub fn first(&self) -> Option<&T> {
        self.st.first()
    }
    pub fn last(&self) -> Option<&T> {
        self.st.last()
    }
    pub fn iter(&mut self) -> Iter<T> {
        self.st.iter()
    }
    pub fn len(&mut self) -> usize {
        self.st.len()
    }
    // pub fn append(&mut self, other: &mut Vec<T>) {
    //     self.st.append(other)
    // }
    pub fn clear(&mut self) {
        self.st.clear();
    }
    pub fn lock(&mut self) {
        self.st.clear();
        self.lock = true
    }
    pub fn unlock(&mut self) {
        self.st.clear();
        self.lock = false
    }
    // pub fn is_locked(&self) -> bool {
    //     self.lock
    // }
    pub fn is_empty(&self) -> bool {
        self.st.is_empty()
    }
    pub fn is_unlocked(&self) -> bool {
        !self.lock
    }
}

impl Stack<u8> {
    pub fn parse_usize(&mut self) -> Result<usize, Error> {
        if self.is_empty() {
            return Err(Error::UsizeParsingError);
        }
        let mut num: usize = 0;
        for n in self.iter() {
            // num = num * 10 + (n.saturating_sub(48_u8)) as usize
            num = num * 10 + (n - 48_u8) as usize
        }
        self.clear();
        Ok(num)
    }
    pub fn parse_color(&mut self) -> Result<Color, Error> {
        let color: Color;
        let length = self.len();
        match length {
            2 => color = Color::Indexed(self.pop().expect("Shouldn't happen len check in place")),
            4 => {
                let b = self.pop().expect("Shouldn't happen len check in place");
                let g = self.pop().expect("Shouldn't happen len check in place");
                let r = self.pop().expect("Shouldn't happen len check in place");
                color = Color::Rgb(r, g, b);
            }
            _ => {
                return Err(Error::ColorParsingError);
            }
        }
        self.clear();
        Ok(color)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum ColorLayer {
    Background,
    Foreground,
}

#[derive(Debug)]
pub struct AnsiGraphicsStack {
    stack: Vec<usize>,
    lock: bool,
}

impl AnsiGraphicsStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            lock: true,
        }
    }
    pub fn push(&mut self, sequence: usize) {
        self.stack.push(sequence);
    }
    pub fn iter(&mut self) -> Iter<usize> {
        self.stack.iter()
    }
    pub fn lock(&mut self) {
        self.stack.clear();
        self.lock = true;
    }
    pub fn unlock(&mut self) {
        self.stack.clear();
        self.lock = false;
    }
    pub fn is_locked(&self) -> bool {
        self.lock
    }
    pub fn is_unlocked(&self) -> bool {
        !self.lock
    }

    // pub fn len(&self) -> usize {
    //     self.stack.len()
    // }

    pub fn parse_ansi(&mut self) -> Result<Style, Error> {
        let mut style = Style::default();

        // let mut stack: Stack<u8> = Stack::new();
        let mut color_stack: Stack<u8> = Stack::new();

        let mut layer: Option<ColorLayer> = None;

        for sequence in self.iter().copied() {
            // sequence should always be an u8
            // but since you can actually write more than u8 (incase of erroneous input)
            // i'm using usize
            // if input is greater than u8 simply skip the iteration and clear the color_stack.

            let code;
            let _seq: Result<u8, _> = sequence.try_into();

            let sequence: u8;
            if let Ok(_s) = _seq {
                sequence = _s;
                code = AnsiCode::from(_s);
            } else {
                // More than a u8
                color_stack.lock();
                continue;
            }

            if color_stack.is_unlocked() {
                // don't match against other stuff
                // on first run it will push 2/5 ie rgb or indexed color
                if color_stack.is_empty() {
                    // sequence should be either 2 or 5
                    color_stack.push(sequence);
                    continue;
                }
                match color_stack.first().ok_or(Error::StackEmpty)? {
                    2 => {
                        // first number is 2 ,second, third and fourth are r, g, b
                        if color_stack.len() <= 4 {
                            color_stack.push(sequence);
                        }
                        if color_stack.len() == 4 {
                            match layer.ok_or(Error::UnknownLayer)? {
                                ColorLayer::Foreground => {
                                    style = style.fg(color_stack.parse_color()?);
                                }
                                ColorLayer::Background => {
                                    style = style.bg(color_stack.parse_color()?);
                                }
                            }
                        }
                    }
                    5 => {
                        if color_stack.len() <= 2 {
                            // first number is  5 second is the color index
                            color_stack.push(sequence);
                        }
                        if color_stack.len() == 2 {
                            match layer.ok_or(Error::UnknownLayer)? {
                                ColorLayer::Foreground => {
                                    style = style.fg(color_stack.parse_color()?);
                                }
                                ColorLayer::Background => {
                                    style = style.bg(color_stack.parse_color()?);
                                }
                            }
                        }
                    }
                    _ => {
                        color_stack.lock();
                    } // first number is not 2 or 5 lock
                }
                //
                continue;
            }
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
                AnsiCode::SetForegroundColor => {
                    // color_layer = Some(ColorLayer::Foreground)
                    // current byte is 38
                    layer = Some(ColorLayer::Foreground);
                    color_stack.unlock();
                }
                AnsiCode::SetBackgroundColor => {
                    // color_layer = Some(ColorLayer::Background)
                    // current byte is 48
                    layer = Some(ColorLayer::Background);
                    color_stack.unlock();
                }
                AnsiCode::ForegroundColor(color) => style = style.fg(color),
                AnsiCode::BackgroundColor(color) => style = style.bg(color),
                _ => (),
            }
        }

        Ok(style)
    }
}
