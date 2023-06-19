use crate::code::AnsiCode;
use nom::{
    branch::alt,
    bytes::complete::*,
    character::complete::*,
    character::is_alphabetic,
    combinator::{map_res, opt, recognize, value},
    error,
    error::FromExternalError,
    multi::*,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};
use std::str::FromStr;
use tui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ColorType {
    /// Eight Bit color
    EightBit,
    /// 24-bit color or true color
    TrueColor,
}

#[derive(Debug, Clone, PartialEq)]
struct AnsiItem {
    code: AnsiCode,
    color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
struct AnsiStates {
    pub items: Vec<AnsiItem>,
    pub style: Style,
}

impl From<AnsiStates> for tui::style::Style {
    fn from(states: AnsiStates) -> Self {
        let mut style = states.style;
        for item in states.items {
            match item.code {
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
                    if let Some(color) = item.color {
                        style = style.fg(color)
                    }
                }
                AnsiCode::SetBackgroundColor => {
                    if let Some(color) = item.color {
                        style = style.bg(color)
                    }
                }
                AnsiCode::ForegroundColor(color) => style = style.fg(color),
                AnsiCode::BackgroundColor(color) => style = style.bg(color),
                AnsiCode::AlternateFonts(_) => (),
                _ => (),
            }
        }
        style
    }
}

pub(crate) fn text(mut s: &[u8]) -> IResult<&[u8], Text<'static>> {
    let mut lines = Vec::new();
    let mut last = Default::default();
    while let Ok((_s, (line, style))) = line(last)(s) {
        lines.push(line);
        last = style;
        s = _s;
        if s.is_empty() {
            break;
        }
    }
    Ok((s, Text::from(lines)))
}

fn line(style: Style) -> impl Fn(&[u8]) -> IResult<&[u8], (Line<'static>, Style)> {
    // let style_: Style = Default::default();
    move |s: &[u8]| -> IResult<&[u8], (Line<'static>, Style)> {
        let (s, mut text) = take_while(|c| c != b'\n')(s)?;
        let (s, _) = opt(tag("\n"))(s)?;
        let mut spans = Vec::new();
        let mut last = style;
        while let Ok((s, span)) = span(last)(text) {
            if span.style == Style::default() && span.content.is_empty() {
                // Reset styles
                last = Style::default();
            } else {
                last = last.patch(span.style);
            }
            // Don't include empty spans but keep changing the style
            if spans.is_empty() || span.content != "" {
                spans.push(span);
            }
            text = s;
            if text.is_empty() {
                break;
            }
        }

        Ok((s, (Line::from(spans), last)))
    }
}

// fn span(s: &[u8]) -> IResult<&[u8], tui::text::Span> {
fn span(last: Style) -> impl Fn(&[u8]) -> IResult<&[u8], Span<'static>, nom::error::Error<&[u8]>> {
    move |s: &[u8]| -> IResult<&[u8], Span<'static>> {
        let mut last = last;
        let (s, style) = opt(style(last))(s)?;

        #[cfg(feature = "simd")]
        let (s, text) = map_res(take_while(|c| c != b'\x1b' | b'\n'), |t| {
            simdutf8::basic::from_utf8(t)
        })(s)?;

        #[cfg(not(feature = "simd"))]
        let (s, text) = map_res(take_while(|c| c != b'\x1b' | b'\n'), |t| {
            std::str::from_utf8(t)
        })(s)?;

        if let Some(style) = style {
            if style == Default::default() {
                last = Default::default();
            } else {
                last = last.patch(style);
            }
        }

        Ok((s, Span::styled(text.to_owned(), last)))
    }
}

fn style(style: Style) -> impl Fn(&[u8]) -> IResult<&[u8], Style, nom::error::Error<&[u8]>> {
    move |s: &[u8]| -> IResult<&[u8], Style> {
        let (s, r) = match opt(ansi_sgr_code)(s)? {
            (s, Some(r)) => (s, r),
            (s, None) => {
                let (s, _) = any_escape_sequence(s)?;
                (s, Vec::new())
            }
        };
        Ok((s, Style::from(AnsiStates { style, items: r })))
    }
}

/// A complete ANSI SGR code
fn ansi_sgr_code(s: &[u8]) -> IResult<&[u8], Vec<AnsiItem>, nom::error::Error<&[u8]>> {
    delimited(
        tag("\x1b["),
        separated_list1(tag(";"), ansi_sgr_item),
        char('m'),
    )(s)
}

fn any_escape_sequence(s: &[u8]) -> IResult<&[u8], Option<&[u8]>> {
    // Attempt to consume most escape codes, including a single escape char.
    //
    // Most escape codes begin with ESC[ and are terminated by an alphabetic character,
    // but OSC codes begin with ESC] and are terminated by an ascii bell (\x07)
    // and a truncated/invalid code may just be a standalone ESC or not be terminated.
    //
    // We should try to consume as much of it as possible to match behavior of most terminals;
    // where we fail at that we should at least consume the escape char to avoid infinitely looping

    preceded(
        char('\x1b'),
        opt(alt((
            delimited(char('['), take_till(|c| is_alphabetic(c)), opt(take(1u8))),
            delimited(char(']'), take_till(|c| c == b'\x07'), opt(take(1u8))),
        ))),
    )(s)
}

/// An ANSI SGR attribute
fn ansi_sgr_item(s: &[u8]) -> IResult<&[u8], AnsiItem> {
    let (s, c) = u8(s)?;
    let code = AnsiCode::from(c);
    let (s, color) = match code {
        AnsiCode::SetForegroundColor | AnsiCode::SetBackgroundColor => {
            let (s, _) = opt(tag(";"))(s)?;
            let (s, color) = color(s)?;
            (s, Some(color))
        }
        _ => (s, None),
    };
    Ok((s, AnsiItem { code, color }))
}

fn color(s: &[u8]) -> IResult<&[u8], Color> {
    let (s, c_type) = color_type(s)?;
    let (s, _) = opt(tag(";"))(s)?;
    match c_type {
        ColorType::TrueColor => {
            let (s, (r, _, g, _, b)) = tuple((u8, tag(";"), u8, tag(";"), u8))(s)?;
            Ok((s, Color::Rgb(r, g, b)))
        }
        ColorType::EightBit => {
            let (s, index) = u8(s)?;
            Ok((s, Color::Indexed(index)))
        }
    }
}

fn color_type(s: &[u8]) -> IResult<&[u8], ColorType> {
    let (s, t) = i64(s)?;
    // NOTE: This isn't opt because a color type must always be followed by a color
    // let (s, _) = opt(tag(";"))(s)?;
    let (s, _) = tag(";")(s)?;
    match t {
        2 => Ok((s, ColorType::TrueColor)),
        5 => Ok((s, ColorType::EightBit)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Alt,
        ))),
    }
}

#[test]
fn color_test() {
    let c = color(b"2;255;255;255").unwrap();
    assert_eq!(c.1, Color::Rgb(255, 255, 255));
    let c = color(b"5;255").unwrap();
    assert_eq!(c.1, Color::Indexed(255));
}

#[test]
fn ansi_items_test() {
    let sc = Default::default();
    let t = style(sc)(b"\x1b[38;2;3;3;3m").unwrap();
    assert_eq!(
        t.1,
        Style::from(AnsiStates {
            style: sc,
            items: vec![AnsiItem {
                code: AnsiCode::SetForegroundColor,
                color: Some(Color::Rgb(3, 3, 3))
            }]
        })
    );
}
