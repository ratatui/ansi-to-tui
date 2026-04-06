use crate::{
    code::AnsiCode,
    hyperlink::{Hyperlink, HyperlinkedLine, HyperlinkedSpan, HyperlinkedText},
};
use nom::{
    AsChar, IResult, Parser,
    branch::alt,
    bytes::complete::*,
    character::complete::*,
    combinator::{cond, map_res, opt},
    multi::*,
    sequence::{delimited, preceded},
};
use ratatui_core::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ColorType {
    /// Eight Bit color
    EightBit,
    /// 24-bit color or true color
    TrueColor,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnsiItem {
    pub(crate) code: AnsiCode,
    pub(crate) color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnsiStates {
    pub items: smallvec::SmallVec<[AnsiItem; 2]>,
    pub style: Style,
}

impl From<AnsiStates> for ratatui_core::style::Style {
    fn from(states: AnsiStates) -> Self {
        let mut style = states.style;
        if states.items.is_empty() {
            // https://github.com/uttarayan21/ansi-to-tui/issues/40
            // [m should be treated as a reset as well
            style = Style::reset();
        }
        for item in states.items {
            match item.code {
                AnsiCode::Reset => style = Style::reset(),
                AnsiCode::Bold => style = style.add_modifier(Modifier::BOLD),
                AnsiCode::Faint => style = style.add_modifier(Modifier::DIM),
                AnsiCode::Normal => {
                    style = style.remove_modifier(Modifier::BOLD | Modifier::DIM);
                }
                AnsiCode::Italic => style = style.add_modifier(Modifier::ITALIC),
                AnsiCode::NotItalic => style = style.remove_modifier(Modifier::ITALIC),
                AnsiCode::Underline => style = style.add_modifier(Modifier::UNDERLINED),
                AnsiCode::UnderlineOff => style = style.remove_modifier(Modifier::UNDERLINED),
                AnsiCode::SlowBlink => style = style.add_modifier(Modifier::SLOW_BLINK),
                AnsiCode::RapidBlink => style = style.add_modifier(Modifier::RAPID_BLINK),
                AnsiCode::BlinkOff => {
                    style = style.remove_modifier(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK)
                }
                AnsiCode::Reverse => style = style.add_modifier(Modifier::REVERSED),
                AnsiCode::InvertOff => style = style.remove_modifier(Modifier::REVERSED),
                AnsiCode::Conceal => style = style.add_modifier(Modifier::HIDDEN),
                AnsiCode::Reveal => style = style.remove_modifier(Modifier::HIDDEN),
                AnsiCode::CrossedOut => style = style.add_modifier(Modifier::CROSSED_OUT),
                AnsiCode::CrossedOutOff => style = style.remove_modifier(Modifier::CROSSED_OUT),
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
                _ => (),
            }
        }
        style
    }
}

pub(crate) fn text(mut s: &[u8]) -> IResult<&[u8], HyperlinkedText<'_>> {
    let mut lines = Vec::new();
    let mut last = Style::new();
    while let Ok((_s, (line, style))) = line(last)(s) {
        lines.push(line);
        last = style;
        s = _s;
        if s.is_empty() {
            break;
        }
    }
    Ok((s, HyperlinkedText::from(lines)))
}

fn newline(s: &[u8]) -> IResult<&[u8], ()> {
    let (s, _) = alt((tag("\r\n"), tag("\n"), tag("\r"))).parse(s)?;
    Ok((s, ()))
}

fn line(style: Style) -> impl Fn(&[u8]) -> IResult<&[u8], (HyperlinkedLine<'_>, Style)> {
    move |s: &[u8]| -> IResult<&[u8], (HyperlinkedLine<'_>, Style)> {
        let (s, mut text) = take_while(|c| c != b'\n' && c != b'\r').parse(s)?;
        let (s, _) = opt(newline).parse(s)?;
        let mut spans = Vec::new();
        let mut last = style;
        while let Ok((s, span)) = span(last)(text) {
            last = last.patch(span.style());
            // If the spans is empty then it might be possible that the style changes
            // but there is no text change
            if !span.is_empty() {
                spans.push(span);
            }
            text = s;
            if text.is_empty() {
                break;
            }
        }

        Ok((s, (HyperlinkedLine::from(spans), last)))
    }
}

fn span(
    last: Style,
) -> impl Fn(&[u8]) -> IResult<&[u8], HyperlinkedSpan<'_>, nom::error::Error<&[u8]>> {
    move |s: &[u8]| -> IResult<&[u8], HyperlinkedSpan<'_>> {
        let mut last = last;
        let (s, style) = style(last).parse(s)?;

        #[cfg(feature = "simd")]
        let text_parser = map_res(
            take_while(|c| c != b'\x1b' && c != b'\n' && c != b'\r'),
            |t| simdutf8::basic::from_utf8(t),
        );

        #[cfg(not(feature = "simd"))]
        let text_parser = map_res(
            take_while(|c| c != b'\x1b' && c != b'\n' && c != b'\r'),
            |t| std::str::from_utf8(t),
        );

        if let Some(style) = style {
            last = last.patch(style);
        }

        let text_span = text_parser.map(|v: &str| HyperlinkedSpan::styled(v, last));

        let hyperlink_span = hyperlink
            .map_res(|v| v.parse())
            .map(|v| HyperlinkedSpan::styled_hyperlink(v.text, v.url, last));

        let text_span = cond(style.is_none(), opt(any_escape_sequence))
            .and(text_span)
            .map(|(_, v)| v);

        hyperlink_span.or(text_span).parse(s)
    }
}

#[allow(clippy::type_complexity)]
fn style(
    style: Style,
) -> impl Fn(&[u8]) -> IResult<&[u8], Option<Style>, nom::error::Error<&[u8]>> {
    move |s: &[u8]| -> IResult<&[u8], Option<Style>> {
        opt(ansi_sgr_code.map(|items| Style::from(AnsiStates { style, items }))).parse(s)
    }
}

/// A complete ANSI SGR code
fn ansi_sgr_code(
    s: &[u8],
) -> IResult<&[u8], smallvec::SmallVec<[AnsiItem; 2]>, nom::error::Error<&[u8]>> {
    delimited(
        tag("\x1b["),
        fold_many0(ansi_sgr_item, smallvec::SmallVec::new, |mut items, item| {
            items.push(item);
            items
        }),
        char('m'),
    )
    .parse(s)
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

    let (input, garbage) = preceded(
        char('\x1b'),
        opt(alt((
            delimited(char('['), take_till(AsChar::is_alpha), opt(take(1u8))),
            delimited(char(']'), take_till(|c| c == b'\x07'), opt(take(1u8))),
        ))),
    )
    .parse(s)?;
    Ok((input, garbage))
}

/// An ANSI SGR attribute
fn ansi_sgr_item(s: &[u8]) -> IResult<&[u8], AnsiItem> {
    let (s, c) = u8(s)?;
    let code = AnsiCode::from(c);
    let (s, color) = match code {
        AnsiCode::SetForegroundColor | AnsiCode::SetBackgroundColor => {
            let (s, _) = opt(tag(";")).parse(s)?;
            let (s, color) = color(s)?;
            (s, Some(color))
        }
        _ => (s, None),
    };
    let (s, _) = opt(tag(";")).parse(s)?;
    Ok((s, AnsiItem { code, color }))
}

fn color(s: &[u8]) -> IResult<&[u8], Color> {
    let (s, c_type) = color_type(s)?;
    let (s, _) = opt(tag(";")).parse(s)?;
    match c_type {
        ColorType::TrueColor => {
            let (s, (r, _, g, _, b)) = (u8, tag(";"), u8, tag(";"), u8).parse(s)?;
            Ok((s, Color::Rgb(r, g, b)))
        }
        ColorType::EightBit => {
            let (s, index) = u8(s)?;
            Ok((s, Color::Indexed(index)))
        }
    }
}

/// A osc 8 hyperlink
/// Format: `\x1b]8;;<url>\x1b\\<text>\x1b]8;;\x1b\\`
///
/// format!("\u{1b}]8;;{url}\u{1b}\\{label}\u{1b}]8;;\u{1b}\\")
fn hyperlink(s: &[u8]) -> IResult<&[u8], Hyperlink<'_, [u8]>> {
    let (s, _) = tag("\x1b]8;;").parse(s)?;
    let (s, url) = take_until("\x1b\\").parse(s)?;
    let (s, _) = tag("\x1b\\").parse(s)?;
    let (s, text) = take_until("\x1b]8;;").parse(s)?;
    let (s, _) = tag("\x1b]8;;\x1b\\").parse(s)?;
    Ok((s, Hyperlink::new(text, url)))
}

fn color_type(s: &[u8]) -> IResult<&[u8], ColorType> {
    let (s, t) = i64(s)?;
    // NOTE: This isn't opt because a color type must always be followed by a color
    // let (s, _) = opt(tag(";")).parse(s)?;
    let (s, _) = tag(";").parse(s)?;
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
    let err = color(b"10;255");
    assert_ne!(err, Ok(c));
}

#[test]
fn ansi_items_test() {
    let sc = Default::default();
    let t = style(sc)(b"\x1b[38;2;3;3;3m").unwrap().1.unwrap();
    assert_eq!(
        t,
        Style::from(AnsiStates {
            style: sc,
            items: vec![AnsiItem {
                code: AnsiCode::SetForegroundColor,
                color: Some(Color::Rgb(3, 3, 3))
            }]
            .into()
        })
    );
    assert_eq!(
        style(sc)(b"\x1b[38;5;3m").unwrap().1.unwrap(),
        Style::from(AnsiStates {
            style: sc,
            items: vec![AnsiItem {
                code: AnsiCode::SetForegroundColor,
                color: Some(Color::Indexed(3))
            }]
            .into()
        })
    );
    assert_eq!(
        style(sc)(b"\x1b[38;5;3;48;5;3m").unwrap().1.unwrap(),
        Style::from(AnsiStates {
            style: sc,
            items: vec![
                AnsiItem {
                    code: AnsiCode::SetForegroundColor,
                    color: Some(Color::Indexed(3))
                },
                AnsiItem {
                    code: AnsiCode::SetBackgroundColor,
                    color: Some(Color::Indexed(3))
                }
            ]
            .into()
        })
    );
    assert_eq!(
        style(sc)(b"\x1b[38;5;3;48;5;3;1m").unwrap().1.unwrap(),
        Style::from(AnsiStates {
            style: sc,
            items: vec![
                AnsiItem {
                    code: AnsiCode::SetForegroundColor,
                    color: Some(Color::Indexed(3))
                },
                AnsiItem {
                    code: AnsiCode::SetBackgroundColor,
                    color: Some(Color::Indexed(3))
                },
                AnsiItem {
                    code: AnsiCode::Bold,
                    color: None
                }
            ]
            .into()
        })
    );
}

#[cfg(test)]
mod test_hyperlinks {
    fn encode_osc8(label: &str, url: &str) -> String {
        format!("\u{1b}]8;;{url}\u{1b}\\{label}\u{1b}]8;;\u{1b}\\")
    }

    #[test]
    fn unit_test_hyperlink() {
        let label = "Google";
        let url = "https://www.google.com";
        let encoded = encode_osc8(label, url);
        let parsed = super::hyperlink(encoded.as_bytes()).unwrap().1;
        assert_eq!(parsed.text, label.as_bytes());
        assert_eq!(parsed.url, url.as_bytes());
    }

    #[test]
    fn test_hyperlink_in_text() {
        let label = "Google";
        let url = "https://www.google.com";
        let encoded = format!(
            "Hello {}! This should be a hyperlink",
            encode_osc8(label, url)
        );
        let parsed = super::text(encoded.as_bytes()).unwrap().1;
        let line = &parsed.lines[0];
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content(), "Hello ");
        assert_eq!(line.spans[1].content(), label);
        assert_eq!(line.spans[2].content(), "! This should be a hyperlink");
    }

    #[test]
    fn test_hyperlink_with_style() {
        let label = "Google";
        let url = "https://www.google.com";
        let encoded = format!(
            "Hello \x1b[1m{}{}! This should be a bold hyperlink\x1b[0m",
            encode_osc8(label, url),
            "\x1b[1m"
        );
        let parsed = super::text(encoded.as_bytes()).unwrap().1;
        assert_eq!(parsed.lines.len(), 1);
        let line = &parsed.lines[0];
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content(), "Hello ");
        assert_eq!(line.spans[1].content(), label);
        assert_eq!(line.spans[2].content(), "! This should be a bold hyperlink");
        assert!(
            line.spans[1]
                .style()
                .has_modifier(ratatui::style::Modifier::BOLD)
        );
    }
}
