use crate::hyperlink::Hyperlink;

use core::fmt;
use ratatui_core::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Style, Styled},
    text::Span,
    widgets::Widget,
};
use std::borrow::Cow;
use std::string::String;
use unicode_width::UnicodeWidthStr;

/// A `Span` that can optionally be a hyperlink. This allows us to render hyperlinks with the same API as regular spans, while still supporting the full range of styling options.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum HyperlinkedSpan<'a> {
    /// A regular span with no hyperlink.
    Span(ratatui_core::text::Span<'a>),
    /// A hyperlink with associated style.
    Hyperlink(StyledHyperlink<'a>),
}

impl<'a> From<Span<'a>> for HyperlinkedSpan<'a> {
    fn from(span: Span<'a>) -> Self {
        HyperlinkedSpan::Span(span)
    }
}

impl<'a> From<StyledHyperlink<'a>> for HyperlinkedSpan<'a> {
    fn from(hyperlink: StyledHyperlink<'a>) -> Self {
        HyperlinkedSpan::Hyperlink(hyperlink)
    }
}

impl core::fmt::Display for HyperlinkedSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HyperlinkedSpan::Span(span) => write!(f, "{}", span),
            HyperlinkedSpan::Hyperlink(hyperlink) => write!(f, "{}", hyperlink.hyperlink.text),
        }
    }
}

impl<'a> HyperlinkedSpan<'a> {
    /// Get the style of the span or hyperlink.
    pub fn style(&self) -> Style {
        match self {
            HyperlinkedSpan::Span(span) => span.style,
            HyperlinkedSpan::Hyperlink(hyperlink) => hyperlink.style,
        }
    }

    /// Set the style of the span or hyperlink, returning a new `HyperlinkedSpan` with the updated style.
    pub fn set_style<S: Into<Style>>(self, style: S) -> Self {
        match self {
            HyperlinkedSpan::Span(span) => HyperlinkedSpan::Span(Span {
                content: span.content,
                style: style.into(),
            }),
            HyperlinkedSpan::Hyperlink(hyperlink) => HyperlinkedSpan::Hyperlink(StyledHyperlink {
                hyperlink: hyperlink.hyperlink,
                style: style.into(),
            }),
        }
    }

    /// Check if the content of the span or hyperlink is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            HyperlinkedSpan::Span(span) => span.content.is_empty(),
            HyperlinkedSpan::Hyperlink(hyperlink) => hyperlink.hyperlink.text.is_empty(),
        }
    }

    /// Create a new `HyperlinkedSpan` with the given text, URL, and style.
    pub fn styled_hyperlink<T, S>(text: T, url: T, style: S) -> Self
    where
        T: Into<Cow<'a, str>>,
        S: Into<Style>,
    {
        HyperlinkedSpan::Hyperlink(StyledHyperlink {
            hyperlink: Hyperlink::new(text, url),
            style: style.into(),
        })
    }

    /// Create a new `HyperlinkedSpan` with the given text and no style or hyperlink
    pub fn raw<T>(content: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        HyperlinkedSpan::Span(Span::raw(content))
    }

    /// Create a new `HyperlinkedSpan` with the given text and URL, using the default style.
    pub fn hyperlink<T>(text: T, url: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        HyperlinkedSpan::Hyperlink(StyledHyperlink {
            hyperlink: Hyperlink::new(text, url),
            style: Style::default(),
        })
    }

    /// Create a new `HyperlinkedSpan` with the given text and style, but no hyperlink.
    pub fn styled<T, S>(content: T, style: S) -> Self
    where
        T: Into<Cow<'a, str>>,
        S: Into<Style>,
    {
        HyperlinkedSpan::Span(Span::styled(content, style))
    }

    /// Set the URL of the spana or hyperlink, returning a new `HyperlinkedSpan` with the updated URL. If the original `HyperlinkedSpan` was a regular span, it will be converted into a hyperlink with the same content and style.
    pub fn link<U>(self, url: U) -> Self
    where
        U: Into<Cow<'a, str>>,
    {
        match self {
            HyperlinkedSpan::Span(span) => HyperlinkedSpan::Hyperlink(StyledHyperlink {
                hyperlink: Hyperlink {
                    text: span.content,
                    url: url.into(),
                },
                style: span.style,
            }),
            HyperlinkedSpan::Hyperlink(hyperlink) => HyperlinkedSpan::Hyperlink(StyledHyperlink {
                hyperlink: Hyperlink {
                    text: hyperlink.hyperlink.text,
                    url: url.into(),
                },
                style: hyperlink.style,
            }),
        }
    }

    // pub fn span(&self) -> Span<'_> {
    //     match self {
    //         HyperlinkedSpan::Span(span) => span.clone(),
    //         HyperlinkedSpan::Hyperlink(hyperlink) => Span {
    //             content: hyperlink.hyperlink.text.clone(),
    //             style: hyperlink.style,
    //         },
    //     }
    // }

    /// Map the content of the span or hyperlink using the provided function, returning a new `HyperlinkedSpan` with the updated content. The style and URL (if applicable) will be preserved.
    pub fn map_content<F>(self, f: F) -> Self
    where
        F: FnOnce(Cow<'a, str>) -> Cow<'a, str>,
    {
        match self {
            HyperlinkedSpan::Span(span) => HyperlinkedSpan::Span(Span {
                content: f(span.content),
                style: span.style,
            }),
            HyperlinkedSpan::Hyperlink(hyperlink) => HyperlinkedSpan::Hyperlink(StyledHyperlink {
                hyperlink: Hyperlink {
                    text: f(hyperlink.hyperlink.text),
                    url: hyperlink.hyperlink.url,
                },
                style: hyperlink.style,
            }),
        }
    }

    /// Create a new `HyperlinkedSpan` with the given content, preserving the style and URL (if applicable) of the original span or hyperlink.
    pub fn with_content(&'a self, content: Cow<'a, str>) -> HyperlinkedSpan<'a> {
        match self {
            HyperlinkedSpan::Span(span) => HyperlinkedSpan::Span(Span {
                content,
                style: span.style,
            }),
            HyperlinkedSpan::Hyperlink(hyperlink) => HyperlinkedSpan::Hyperlink(StyledHyperlink {
                hyperlink: Hyperlink {
                    text: content,
                    url: hyperlink.hyperlink.url.clone(),
                },
                style: hyperlink.style,
            }),
        }
    }

    /// Get the content of the span or hyperlink as a string slice.
    /// This is the part that is rendered, so for hyperlinks it returns the text rather than the URL.
    pub fn content(&self) -> &str {
        match self {
            HyperlinkedSpan::Span(span) => &span.content,
            HyperlinkedSpan::Hyperlink(hyperlink) => &hyperlink.hyperlink.text,
        }
    }

    /// Convert the `HyperlinkedSpan` into a version with `'static` lifetime by cloning the content and URL (if applicable). This is useful for storing the `HyperlinkedSpan` in a context where the original lifetime cannot be guaranteed.
    pub fn make_static(self) -> HyperlinkedSpan<'static> {
        match self {
            HyperlinkedSpan::Span(span) => HyperlinkedSpan::Span(Span {
                content: Cow::Owned(span.content.into_owned()),
                style: span.style,
            }),
            HyperlinkedSpan::Hyperlink(hyperlink) => HyperlinkedSpan::Hyperlink(StyledHyperlink {
                hyperlink: Hyperlink {
                    text: Cow::Owned(hyperlink.hyperlink.text.into_owned()),
                    url: Cow::Owned(hyperlink.hyperlink.url.into_owned()),
                },
                style: hyperlink.style,
            }),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StyledHyperlink<'a> {
    pub(crate) hyperlink: Hyperlink<'a, str>,
    pub(crate) style: Style,
}

impl StyledHyperlink<'_> {
    pub fn style(&self) -> Style {
        self.style
    }

    pub fn make_static(self) -> StyledHyperlink<'static> {
        StyledHyperlink {
            hyperlink: Hyperlink {
                text: Cow::Owned(self.hyperlink.text.into_owned()),
                url: Cow::Owned(self.hyperlink.url.into_owned()),
            },
            style: self.style,
        }
    }
}

impl UnicodeWidthStr for StyledHyperlink<'_> {
    fn width(&self) -> usize {
        self.hyperlink.text.width()
    }

    fn width_cjk(&self) -> usize {
        self.hyperlink.text.width_cjk()
    }
}

impl UnicodeWidthStr for HyperlinkedSpan<'_> {
    fn width(&self) -> usize {
        match self {
            HyperlinkedSpan::Span(span) => span.content.width(),
            HyperlinkedSpan::Hyperlink(hyperlink) => hyperlink.hyperlink.text.width(),
        }
    }

    fn width_cjk(&self) -> usize {
        match self {
            HyperlinkedSpan::Span(span) => span.content.width_cjk(),
            HyperlinkedSpan::Hyperlink(hyperlink) => hyperlink.hyperlink.text.width_cjk(),
        }
    }
}

impl Widget for HyperlinkedSpan<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &HyperlinkedSpan<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        match self {
            HyperlinkedSpan::Span(span) => {
                Widget::render(span, area, buf);
            }
            HyperlinkedSpan::Hyperlink(styled_hyperlink) => {
                render_hyperlink(styled_hyperlink, area, buf);
            }
        }
    }
}

pub(crate) fn render_hyperlink(hyperlink: &StyledHyperlink<'_>, area: Rect, buf: &mut Buffer) {
    if area.is_empty() {
        return;
    }

    let url = &hyperlink.hyperlink.url;
    let text = &hyperlink.hyperlink.text;
    let style = hyperlink.style;

    let label_width = text.width().min(area.width as usize);
    if label_width == 0 {
        return;
    }

    // Truncate the visible label to fit the available width.
    let label = if text.width() > area.width as usize {
        use unicode_truncate::UnicodeTruncateStr;
        let (truncated, _) = text.unicode_truncate(area.width as usize);
        truncated
    } else {
        text.as_ref()
    };

    let encoded = encode_osc8(label, url);

    // First cell: the full OSC 8 encoded string.
    if let Some(cell) = buf.cell_mut(Position::new(area.x, area.y)) {
        cell.set_symbol(&encoded);
        cell.set_style(style);
        cell.set_skip(false);
    }

    // Remaining cells: blank and skipped so the diff does not re-emit them.
    for offset in 1..label_width as u16 {
        let x = area.x + offset;
        if let Some(cell) = buf.cell_mut(Position::new(x, area.y)) {
            cell.set_symbol(" ");
            cell.set_style(style);
            cell.set_skip(true);
        }
    }
}

#[inline]
fn encode_osc8(label: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\")
}

impl Styled for HyperlinkedSpan<'_> {
    type Item = Self;
    fn style(&self) -> Style {
        self.style()
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.set_style(style)
    }
}
