use core::fmt;
use ratatui_core::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Style, Styled},
    widgets::Widget,
};
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

/// A `Span` that can optionally be a hyperlink. This allows us to render hyperlinks with the same API as regular spans, while still supporting the full range of styling options.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct HyperlinkedSpan<'a> {
    /// The style of the span.
    pub style: Style,
    /// The content of the span as a Clone-on-write string.
    pub content: Cow<'a, str>,
    /// The URL associated with the span.
    pub url: Option<Cow<'a, str>>,
}

impl core::fmt::Display for HyperlinkedSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl<'a> HyperlinkedSpan<'a> {
    /// Get the style of the span.
    pub fn style(&self) -> Style {
        self.style
    }

    /// Set the style of the span, returning a new [`HyperlinkedSpan`] with the updated style.
    pub fn set_style<S: Into<Style>>(self, style: S) -> Self {
        Self {
            style: style.into(),
            ..self
        }
    }

    /// Check if the content of the span is empty.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Create a new [`HyperlinkedSpan`] with the given text, URL, and style.
    #[inline]
    pub fn styled_hyperlink<T, S>(text: T, url: T, style: S) -> Self
    where
        T: Into<Cow<'a, str>>,
        S: Into<Style>,
    {
        Self {
            style: style.into(),
            content: text.into(),
            url: Some(url.into()),
        }
    }

    /// Create a new [`HyperlinkedSpan`] with the given text and no style or URL
    pub fn raw<T>(content: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            style: Style::default(),
            content: content.into(),
            url: None,
        }
    }

    /// Create a new [`HyperlinkedSpan`] with the given text and URL, using the default style.
    pub fn hyperlink<T>(text: T, url: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            style: Style::default(),
            content: text.into(),
            url: Some(url.into()),
        }
    }

    /// Create a new [`HyperlinkedSpan`] with the given text and style, but no URL.
    #[inline]
    pub fn styled<T, S>(content: T, style: S) -> Self
    where
        T: Into<Cow<'a, str>>,
        S: Into<Style>,
    {
        Self {
            style: style.into(),
            content: content.into(),
            url: None,
        }
    }

    /// Set the URL of the span, returning a new [`HyperlinkedSpan`] with the updated URL.
    pub fn link<U>(self, url: U) -> Self
    where
        U: Into<Cow<'a, str>>,
    {
        Self {
            url: Some(url.into()),
            ..self
        }
    }

    /// Map the content of the span using the provided function, returning a new [`HyperlinkedSpan`] with the updated content. The style and URL will be preserved.
    pub fn map_content<F>(self, f: F) -> Self
    where
        F: FnOnce(Cow<'a, str>) -> Cow<'a, str>,
    {
        Self {
            content: f(self.content),
            ..self
        }
    }

    /// Create a new [`HyperlinkedSpan`] with the given content, preserving the style and URL of the original span.
    pub fn with_content(&'a self, content: Cow<'a, str>) -> HyperlinkedSpan<'a> {
        Self {
            content,
            style: self.style,
            url: self.url.clone(),
        }
    }

    /// Get the content of the span as a string slice.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Convert the [`HyperlinkedSpan`] into a version with `'static` lifetime by cloning the content and URL. This is useful for storing the [`HyperlinkedSpan`] in a context where the original lifetime cannot be guaranteed.
    pub fn make_static(self) -> HyperlinkedSpan<'static> {
        HyperlinkedSpan {
            style: self.style,
            content: Cow::Owned(self.content.into_owned()),
            url: self.url.map(|u| Cow::Owned(u.into_owned())),
        }
    }

    /// Converts this [`HyperlinkedSpan`] into a [`ratatui_core::text::Span`].
    ///
    /// Note: Hyperlink information is lost during conversion.
    pub fn into_ratatui_lossy(self) -> ratatui_core::text::Span<'a> {
        ratatui_core::text::Span {
            content: self.content,
            style: self.style,
        }
    }
}

impl<'a> From<ratatui_core::text::Span<'a>> for HyperlinkedSpan<'a> {
    fn from(span: ratatui_core::text::Span<'a>) -> Self {
        Self {
            style: span.style,
            content: span.content,
            url: None,
        }
    }
}

impl UnicodeWidthStr for HyperlinkedSpan<'_> {
    fn width(&self) -> usize {
        self.content.width()
    }

    fn width_cjk(&self) -> usize {
        self.content.width_cjk()
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

        if let Some(url) = &self.url {
            render_hyperlink(&self.content, url, self.style, area, buf);
        } else {
            let span = ratatui_core::text::Span {
                content: self.content.clone(),
                style: self.style,
            };
            Widget::render(&span, area, buf);
        }
    }
}

pub(crate) fn render_hyperlink(text: &str, url: &str, style: Style, area: Rect, buf: &mut Buffer) {
    if area.is_empty() {
        return;
    }

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
        text
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
