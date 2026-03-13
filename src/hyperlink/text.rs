use std::borrow::Cow;

use ratatui_core::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Style, Styled},
    widgets::Widget,
};
use unicode_width::UnicodeWidthStr;

use crate::hyperlink::{HyperlinkedSpan, line::HyperlinkedLine};

/// A string split over one or more lines, where each line may contain hyperlinks.
///
/// This is the hyperlink-aware equivalent of [`HyperlinkedText`]. When rendered,
/// hyperlinks produce OSC 8 escape sequences so that supporting terminals display clickable
/// links.
#[derive(Default, Clone, Eq, PartialEq, Hash, Debug)]
pub struct HyperlinkedText<'a> {
    /// The alignment of this text.
    pub alignment: Option<Alignment>,
    /// The style of this text.
    pub style: Style,
    /// The lines that make up this text.
    pub lines: Vec<HyperlinkedLine<'a>>,
}

impl<'a> HyperlinkedText<'a> {
    /// Returns the max width of all the lines.
    pub fn width(&self) -> usize {
        UnicodeWidthStr::width(self)
    }

    /// Returns the height (number of lines).
    pub fn height(&self) -> usize {
        self.lines.len()
    }

    /// Sets the style of this text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Patches the style of this text, adding modifiers from the given style.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn patch_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = self.style.patch(style);
        self
    }

    /// Resets the style of this text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn reset_style(self) -> Self {
        self.patch_style(Style::reset())
    }

    /// Sets the alignment for this text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn alignment(self, alignment: Alignment) -> Self {
        Self {
            alignment: Some(alignment),
            ..self
        }
    }

    /// Left-aligns the whole text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn left_aligned(self) -> Self {
        self.alignment(Alignment::Left)
    }

    /// Center-aligns the whole text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }

    /// Right-aligns the whole text.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn right_aligned(self) -> Self {
        self.alignment(Alignment::Right)
    }

    /// Returns an iterator over the lines of the text.
    pub fn iter(&self) -> core::slice::Iter<'_, HyperlinkedLine<'a>> {
        self.lines.iter()
    }

    /// Returns an iterator that allows modifying each line.
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, HyperlinkedLine<'a>> {
        self.lines.iter_mut()
    }

    /// Adds a line to the text.
    pub fn push_line<T: Into<HyperlinkedLine<'a>>>(&mut self, line: T) {
        self.lines.push(line.into());
    }

    /// Converts this text into one with a static lifetime, cloning all the lines in the process.
    pub fn make_static(self) -> HyperlinkedText<'static> {
        HyperlinkedText {
            lines: self
                .lines
                .into_iter()
                .map(|line| line.make_static())
                .collect(),
            ..self
        }
    }

    /// Creates a new `HyperlinkedText` from the given content, splitting it into lines on newline characters.
    pub fn raw<T>(content: T) -> Self
    where
        T: Into<Cow<'a, str>>,
    {
        let lines: Vec<_> = match content.into() {
            Cow::Borrowed("") => vec![HyperlinkedLine::from("")],
            Cow::Borrowed(s) => s.lines().map(HyperlinkedLine::from).collect(),
            Cow::Owned(s) if s.is_empty() => vec![HyperlinkedLine::from("")],
            Cow::Owned(s) => s
                .lines()
                .map(|l| HyperlinkedLine::from(l.to_owned()))
                .collect(),
        };
        Self::from(lines)
    }
}

impl UnicodeWidthStr for HyperlinkedText<'_> {
    fn width(&self) -> usize {
        self.lines
            .iter()
            .map(UnicodeWidthStr::width)
            .max()
            .unwrap_or_default()
    }

    fn width_cjk(&self) -> usize {
        self.lines
            .iter()
            .map(UnicodeWidthStr::width_cjk)
            .max()
            .unwrap_or_default()
    }
}

impl Widget for HyperlinkedText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &HyperlinkedText<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        buf.set_style(area, self.style);
        for (line, row) in self.lines.iter().zip(area.rows()) {
            line.render_with_alignment(row, buf, self.alignment);
        }
    }
}

impl Styled for HyperlinkedText<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

impl<'a> From<Vec<HyperlinkedLine<'a>>> for HyperlinkedText<'a> {
    fn from(lines: Vec<HyperlinkedLine<'a>>) -> Self {
        Self {
            lines,
            ..Default::default()
        }
    }
}

impl<'a> IntoIterator for HyperlinkedText<'a> {
    type Item = HyperlinkedLine<'a>;
    type IntoIter = std::vec::IntoIter<HyperlinkedLine<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

impl<'a> IntoIterator for &'a HyperlinkedText<'a> {
    type Item = &'a HyperlinkedLine<'a>;
    type IntoIter = core::slice::Iter<'a, HyperlinkedLine<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut HyperlinkedText<'a> {
    type Item = &'a mut HyperlinkedLine<'a>;
    type IntoIter = core::slice::IterMut<'a, HyperlinkedLine<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl From<String> for HyperlinkedText<'_> {
    fn from(s: String) -> Self {
        Self::raw(s)
    }
}

impl<'a> From<&'a str> for HyperlinkedText<'a> {
    fn from(s: &'a str) -> Self {
        Self::raw(s)
    }
}

impl<'a> From<Cow<'a, str>> for HyperlinkedText<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        Self::raw(s)
    }
}

impl<'a> From<HyperlinkedSpan<'a>> for HyperlinkedText<'a> {
    fn from(span: HyperlinkedSpan<'a>) -> Self {
        Self {
            lines: vec![HyperlinkedLine::from(span)],
            ..Default::default()
        }
    }
}

impl<'a> From<ratatui_core::text::Span<'a>> for HyperlinkedText<'a> {
    fn from(span: ratatui_core::text::Span<'a>) -> Self {
        Self {
            lines: vec![HyperlinkedLine::from(span)],
            ..Default::default()
        }
    }
}

impl<'a> From<HyperlinkedLine<'a>> for HyperlinkedText<'a> {
    fn from(line: HyperlinkedLine<'a>) -> Self {
        Self {
            lines: vec![line],
            ..Default::default()
        }
    }
}

impl<'a, T> FromIterator<T> for HyperlinkedText<'a>
where
    T: Into<HyperlinkedLine<'a>>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let lines = iter.into_iter().map(Into::into).collect();
        Self {
            lines,
            ..Default::default()
        }
    }
}
