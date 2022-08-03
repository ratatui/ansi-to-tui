use crate::code::AnsiCode;
use nom::{
    branch::alt,
    bytes::complete::*,
    character::complete::*,
    combinator::{map_res, opt, recognize, value},
    error,
    error::FromExternalError,
    multi::*,
    sequence::tuple,
    IResult, Parser,
};
use std::str::FromStr;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
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
                        style = style.fg(color)
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
    let mut line_spans = Vec::new();
    let mut last = Default::default();
    while let Ok((_s, (spans, style))) = spans(last)(s) {
        line_spans.push(spans);
        last = style;
        s = _s;
        if s.is_empty() {
            break;
        }
    }
    Ok((s, Text::from(line_spans)))
}

fn spans(style: Style) -> impl Fn(&[u8]) -> IResult<&[u8], (Spans<'static>, Style)> {
    // let style_: Style = Default::default();
    move |s: &[u8]| -> IResult<&[u8], (Spans<'static>, Style)> {
        let (s, mut text) = take_while(|c| c != b'\n')(s)?;
        let (s, _) = opt(tag("\n"))(s)?;
        let mut spans = Vec::new();
        let mut last = style;
        loop {
            let (s, span) = span(last)(text)?;
            last = span.style;
            spans.push(span);
            text = s;
            if text.is_empty() {
                break;
            }
        }

        Ok((s, (Spans(spans), last)))
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
        let (s, _) = tag("\x1b[")(s)?;
        let (s, r) = separated_list0(tag(";"), ansi_item)(s)?;
        // the ones ending with m are styles and the ones ending with h are screen mode escapes
        let (s, _) = alt((char('m'), alt((char('h'), char('l')))))(s)?;
        Ok((s, Style::from(AnsiStates { style, items: r })))
    }
}

/// An ansi item is a code with a possible color.
fn ansi_item(s: &[u8]) -> IResult<&[u8], AnsiItem> {
    // Screen escape modes start with '?' or '=' or non-number
    let (s, nc) = opt(alt((char('?'), char('='))))(s)?;
    let (mut s, c) = i64(s)?;
    if let Some(nc) = nc {
        return Ok((
            s,
            AnsiItem {
                code: AnsiCode::Code(vec![nc as u8]),
                color: None,
            },
        ));
    }
    let code = AnsiCode::from(c as u8);
    let color = if matches!(
        code,
        AnsiCode::SetBackgroundColor | AnsiCode::SetForegroundColor
    ) {
        let (_s, _) = opt(tag(";"))(s)?;
        let (_s, color) = color(_s)?;
        s = _s;
        Some(color)
    } else {
        None
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
