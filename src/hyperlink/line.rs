use crate::hyperlink::span::{HyperlinkedSpan, render_hyperlink};

use ratatui_core::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Style, Styled},
    widgets::Widget,
};
use std::borrow::Cow;
use unicode_truncate::UnicodeTruncateStr;
use unicode_width::UnicodeWidthStr;

/// A line of text consisting of one or more [`HyperlinkedSpan`]s.
///
/// This is the hyperlink-aware equivalent of [`ratatui_core::text::HyperlinkedLine`]. Each span may be a
/// plain styled span or a styled hyperlink. When rendered, hyperlinks produce OSC 8 escape
/// sequences so that supporting terminals display clickable links.
#[derive(Default, Clone, Eq, PartialEq, Hash, Debug)]
pub struct HyperlinkedLine<'a> {
    /// The style of this line of text.
    pub style: Style,

    /// The alignment of this line of text.
    pub alignment: Option<Alignment>,

    /// The spans that make up this line of text.
    pub spans: Vec<HyperlinkedSpan<'a>>,
}

impl<'a> HyperlinkedLine<'a> {
    /// Sets the spans of this line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn spans<I>(mut self, spans: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<HyperlinkedSpan<'a>>,
    {
        self.spans = spans.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the style of this line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the alignment of this line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn alignment(self, alignment: Alignment) -> Self {
        Self {
            alignment: Some(alignment),
            ..self
        }
    }

    /// Left-aligns this line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn left_aligned(self) -> Self {
        self.alignment(Alignment::Left)
    }

    /// Center-aligns this line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }

    /// Right-aligns this line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn right_aligned(self) -> Self {
        self.alignment(Alignment::Right)
    }

    /// Returns the unicode display width of this line.
    #[must_use]
    pub fn width(&self) -> usize {
        UnicodeWidthStr::width(self)
    }

    /// Patches the style of this line, adding modifiers from the given style.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn patch_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = self.style.patch(style);
        self
    }

    /// Resets the style of this line.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn reset_style(self) -> Self {
        self.patch_style(Style::reset())
    }

    /// Returns an iterator over the spans of this line.
    pub fn iter(&self) -> core::slice::Iter<'_, HyperlinkedSpan<'a>> {
        self.spans.iter()
    }

    /// Returns a mutable iterator over the spans of this line.
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, HyperlinkedSpan<'a>> {
        self.spans.iter_mut()
    }

    /// Adds a span to the line.
    pub fn push_span<T: Into<HyperlinkedSpan<'a>>>(&mut self, span: T) {
        self.spans.push(span.into());
    }

    /// Adds a hyperlink span to the line with the given content and URL.
    pub fn make_static(self) -> HyperlinkedLine<'static> {
        HyperlinkedLine {
            style: self.style,
            alignment: self.alignment,
            spans: self
                .spans
                .into_iter()
                .map(|span| span.make_static())
                .collect(),
        }
    }

    /// Creates a new `HyperlinkedLine` from the given content, splitting it into spans by newlines.
    pub fn raw<T>(content: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        Self {
            spans: cow_to_spans(content),
            ..Default::default()
        }
    }
}

fn cow_to_spans<'a>(content: impl Into<Cow<'a, str>>) -> Vec<HyperlinkedSpan<'a>> {
    match content.into() {
        Cow::Borrowed(s) => s.lines().map(HyperlinkedSpan::raw).collect(),
        Cow::Owned(s) => s
            .lines()
            .map(|v| HyperlinkedSpan::raw(v.to_string()))
            .collect(),
    }
}

impl UnicodeWidthStr for HyperlinkedLine<'_> {
    fn width(&self) -> usize {
        self.spans.iter().map(UnicodeWidthStr::width).sum()
    }

    fn width_cjk(&self) -> usize {
        self.spans.iter().map(UnicodeWidthStr::width_cjk).sum()
    }
}

impl core::fmt::Display for HyperlinkedLine<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for span in &self.spans {
            write!(f, "{span}")?;
        }
        Ok(())
    }
}

impl Styled for HyperlinkedLine<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a> From<Vec<HyperlinkedSpan<'a>>> for HyperlinkedLine<'a> {
    fn from(spans: Vec<HyperlinkedSpan<'a>>) -> Self {
        Self {
            spans,
            ..Default::default()
        }
    }
}

impl<'a> From<Vec<ratatui_core::text::Span<'a>>> for HyperlinkedLine<'a> {
    fn from(spans: Vec<ratatui_core::text::Span<'a>>) -> Self {
        Self {
            spans: spans.into_iter().map(HyperlinkedSpan::from).collect(),
            ..Default::default()
        }
    }
}

impl Widget for HyperlinkedLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &HyperlinkedLine<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_with_alignment(area, buf, None);
    }
}

impl HyperlinkedLine<'_> {
    /// An internal implementation method for `Widget::render` that allows the parent widget to
    /// define a default alignment, to be used if `HyperlinkedLine::alignment` is `None`.
    pub(crate) fn render_with_alignment(
        &self,
        area: Rect,
        buf: &mut Buffer,
        parent_alignment: Option<Alignment>,
    ) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        let area = Rect { height: 1, ..area };
        let line_width = self.width();
        if line_width == 0 {
            return;
        }

        buf.set_style(area, self.style);

        let alignment = self.alignment.or(parent_alignment);

        let area_width = usize::from(area.width);
        let can_render_complete_line = line_width <= area_width;
        if can_render_complete_line {
            let indent_width = match alignment {
                Some(Alignment::Center) => (area_width.saturating_sub(line_width)) / 2,
                Some(Alignment::Right) => area_width.saturating_sub(line_width),
                Some(Alignment::Left) | None => 0,
            };
            let indent_width = u16::try_from(indent_width).unwrap_or(u16::MAX);
            let area = Rect {
                x: area.x.saturating_add(indent_width),
                width: area.width.saturating_sub(indent_width),
                ..area
            };
            render_spans(&self.spans, area, buf, 0);
        } else {
            // There is not enough space to render the whole line. As the right side is truncated by
            // the area width, only truncate the left.
            let skip_width = match alignment {
                Some(Alignment::Center) => (line_width.saturating_sub(area_width)) / 2,
                Some(Alignment::Right) => line_width.saturating_sub(area_width),
                Some(Alignment::Left) | None => 0,
            };
            render_spans(&self.spans, area, buf, skip_width);
        }
    }
}

/// Renders all the spans of the line that should be visible.
fn render_spans(
    spans: &[HyperlinkedSpan],
    mut area: Rect,
    buf: &mut Buffer,
    span_skip_width: usize,
) {
    for (span, span_width, offset) in spans_after_width(spans, span_skip_width) {
        area = Rect {
            x: area.x.saturating_add(offset),
            width: area.width.saturating_sub(offset),
            ..area
        };
        if area.is_empty() {
            break;
        }
        span.render(area, buf);
        let span_width = u16::try_from(span_width).unwrap_or(u16::MAX);
        area = Rect {
            x: area.x.saturating_add(span_width),
            width: area.width.saturating_sub(span_width),
            ..area
        };
    }
}

/// Returns an iterator over the spans that lie after a given skip width from the start of the
/// `HyperlinkedLine` (including a partially visible span if the `skip_width` lands within a span).
fn spans_after_width<'a>(
    spans: &'a [HyperlinkedSpan],
    mut skip_width: usize,
) -> impl Iterator<Item = (HyperlinkedSpan<'a>, usize, u16)> {
    spans
        .iter()
        .map(|span| (span, span.width()))
        // Filter non visible spans out.
        .filter_map(move |(span, span_width)| {
            // Ignore spans that are completely before the offset. Decrement `span_skip_width` by
            // the span width until we find a span that is partially or completely visible.
            if skip_width >= span_width {
                skip_width = skip_width.saturating_sub(span_width);
                return None;
            }

            // Apply the skip from the start of the span, not the end as the end will be trimmed
            // when rendering the span to the buffer.
            let available_width = span_width.saturating_sub(skip_width);
            skip_width = 0; // ensure the next span is rendered in full
            Some((span, span_width, available_width))
        })
        .map(|(span, span_width, available_width)| {
            if span_width <= available_width {
                // HyperlinkedSpan is fully visible. Clone here is fast as the underlying content is `Cow`.
                return (span.clone(), span_width, 0u16);
            }
            // HyperlinkedSpan is only partially visible. As the end is truncated by the area width, only
            // truncate the start of the span.
            let (content, actual_width) = span.content().unicode_truncate_start(available_width);

            // When the first grapheme of the span was truncated, start rendering from a position
            // that takes that into account by indenting the start of the area
            let first_grapheme_offset = available_width.saturating_sub(actual_width);
            let first_grapheme_offset = u16::try_from(first_grapheme_offset).unwrap_or(u16::MAX);
            (
                span.with_content(Cow::Borrowed(content)),
                actual_width,
                first_grapheme_offset,
            )
        })
}

impl From<String> for HyperlinkedLine<'_> {
    fn from(s: String) -> Self {
        Self::raw(s)
    }
}

impl<'a> From<&'a str> for HyperlinkedLine<'a> {
    fn from(s: &'a str) -> Self {
        Self::raw(s)
    }
}

impl<'a> From<Cow<'a, str>> for HyperlinkedLine<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        Self::raw(s)
    }
}

impl<'a> From<HyperlinkedSpan<'a>> for HyperlinkedLine<'a> {
    fn from(span: HyperlinkedSpan<'a>) -> Self {
        Self::from(vec![span])
    }
}

impl<'a, T> FromIterator<T> for HyperlinkedLine<'a>
where
    T: Into<HyperlinkedSpan<'a>>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(iter.into_iter().map(Into::into).collect::<Vec<_>>())
    }
}

impl<'a> From<ratatui_core::text::Span<'a>> for HyperlinkedLine<'a> {
    fn from(span: ratatui_core::text::Span<'a>) -> Self {
        Self::from(vec![span])
    }
}
