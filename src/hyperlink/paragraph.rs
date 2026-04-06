use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Alignment, Position, Rect};
use ratatui_core::style::{Style, Styled};
use ratatui_core::widgets::Widget;
use ratatui_widgets::block::BlockExt;
use ratatui_widgets::paragraph::Wrap;

use crate::hyperlink::{HyperlinkedLine, HyperlinkedText};

type Horizontal = u16;
type Vertical = u16;

/// A widget to display some text with hyperlink support.
///
/// It is used to display a block of text. The text can be styled and aligned. It can also be
/// wrapped to the next line if it is too long to fit in the given area.
///
/// The text can be any type that can be converted into a [`HyperlinkedText`]. By default, the text
/// is styled with [`Style::default()`], not wrapped, and aligned to the left.
///
/// The text can be wrapped to the next line if it is too long to fit in the given area. The
/// wrapping can be configured with the [`wrap`](HyperlinkedParagraph::wrap) method.
///
/// The text can be aligned to the left, right, or center. The alignment can be configured with
/// the [`alignment`](HyperlinkedParagraph::alignment) method or with the
/// [`left_aligned`](HyperlinkedParagraph::left_aligned),
/// [`right_aligned`](HyperlinkedParagraph::right_aligned), and
/// [`centered`](HyperlinkedParagraph::centered) methods.
///
/// The text can be scrolled to show a specific part of the text. The scroll offset can be set
/// with the [`scroll`](HyperlinkedParagraph::scroll) method.
///
/// The text can be surrounded by a [`Block`] with a title and borders. The block can be
/// configured with the [`block`](HyperlinkedParagraph::block) method.
///
/// The style of the text can be set with the [`style`](HyperlinkedParagraph::style) method.
/// This style will be applied to the entire widget, including the block if one is present.
/// Any style set on the block or text will be added to this style. See the [`Style`] type for
/// more information on how styles are combined.
///
/// # Example
///
/// ```rust
/// use ansi_to_tui::HyperlinkedParagraph;
/// use ratatui_core::layout::Alignment;
/// use ratatui_core::style::{Style, Stylize};
/// use ratatui_widgets::block::Block;
/// use ratatui_widgets::paragraph::Wrap;
///
/// let paragraph = HyperlinkedParagraph::new("Hello, world!")
///     .block(Block::bordered().title("Paragraph"))
///     .style(Style::new().white().on_black())
///     .alignment(Alignment::Center)
///     .wrap(Wrap { trim: true });
/// ```
///
/// [`Block`]: ratatui_widgets::block::Block
/// [`Style`]: ratatui_core::style::Style
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct HyperlinkedParagraph<'a> {
    /// A block to wrap the widget in
    block: Option<ratatui_widgets::block::Block<'a>>,
    /// Widget style
    style: Style,
    /// How to wrap the text
    wrap: Option<Wrap>,
    /// The text to display
    text: HyperlinkedText<'a>,
    /// Scroll
    scroll: Position,
    /// Alignment of the text
    alignment: Alignment,
}

impl<'a> HyperlinkedParagraph<'a> {
    /// Creates a new [`HyperlinkedParagraph`] widget with the given text.
    ///
    /// The `text` parameter can be a [`HyperlinkedText`] or any type that can be converted into a
    /// [`HyperlinkedText`]. By default, the text is styled with [`Style::default()`], not wrapped,
    /// and aligned to the left.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ansi_to_tui::HyperlinkedParagraph;
    /// use ansi_to_tui::HyperlinkedText;
    /// use ansi_to_tui::HyperlinkedLine;
    ///
    /// let paragraph = HyperlinkedParagraph::new("Hello, world!");
    /// let paragraph = HyperlinkedParagraph::new(String::from("Hello, world!"));
    /// let paragraph = HyperlinkedParagraph::new(HyperlinkedText::raw("Hello, world!"));
    /// let paragraph = HyperlinkedParagraph::new(HyperlinkedLine::from("Hello, world!"));
    /// ```
    pub fn new<T>(text: T) -> Self
    where
        T: Into<HyperlinkedText<'a>>,
    {
        Self {
            block: None,
            style: Style::default(),
            wrap: None,
            text: text.into(),
            scroll: Position::ORIGIN,
            alignment: Alignment::Left,
        }
    }

    /// Surrounds the [`HyperlinkedParagraph`] widget with a [`Block`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use ansi_to_tui::HyperlinkedParagraph;
    /// use ratatui_widgets::block::Block;
    ///
    /// let paragraph = HyperlinkedParagraph::new("Hello, world!")
    ///     .block(Block::bordered().title("Paragraph"));
    /// ```
    ///
    /// [`Block`]: ratatui_widgets::block::Block
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the style of the entire widget.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This applies to the entire widget, including the block if one is present. Any style set
    /// on the block or text will be added to this style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ansi_to_tui::HyperlinkedParagraph;
    /// use ratatui_core::style::{Style, Stylize};
    ///
    /// let paragraph = HyperlinkedParagraph::new("Hello, world!")
    ///     .style(Style::new().red().on_white());
    /// ```
    ///
    /// [`Color`]: ratatui_core::style::Color
    /// [`Style`]: ratatui_core::style::Style
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the wrapping configuration for the widget.
    ///
    /// See [`Wrap`] for more information on the different options.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Set the scroll offset for the given paragraph
    ///
    /// The scroll offset is a tuple of (y, x) offset. The y offset is the number of lines to
    /// scroll, and the x offset is the number of characters to scroll. The scroll offset is
    /// applied after the text is wrapped and aligned.
    ///
    /// Note: the order of the tuple is (y, x) instead of (x, y), which is different from
    /// general convention across the crate.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn scroll(mut self, offset: (Vertical, Horizontal)) -> Self {
        self.scroll = Position {
            x: offset.1,
            y: offset.0,
        };
        self
    }

    /// Set the text alignment for the given paragraph
    ///
    /// The alignment is a variant of the [`Alignment`] enum which can be one of Left, Right, or
    /// Center. If no alignment is specified, the text in a paragraph will be left-aligned.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Left-aligns the text in the given paragraph.
    ///
    /// Convenience shortcut for [`HyperlinkedParagraph::alignment`]\([`Alignment::Left`]\).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn left_aligned(self) -> Self {
        self.alignment(Alignment::Left)
    }

    /// Center-aligns the text in the given paragraph.
    ///
    /// Convenience shortcut for [`HyperlinkedParagraph::alignment`]\([`Alignment::Center`]\).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }

    /// Right-aligns the text in the given paragraph.
    ///
    /// Convenience shortcut for [`HyperlinkedParagraph::alignment`]\([`Alignment::Right`]\).
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn right_aligned(self) -> Self {
        self.alignment(Alignment::Right)
    }

    /// Converts this [`HyperlinkedParagraph`] into a [`ratatui_widgets::paragraph::Paragraph`].
    ///
    /// Note: Hyperlink information is lost during conversion.
    pub fn into_ratatui_lossy(self) -> ratatui_widgets::paragraph::Paragraph<'a> {
        let mut paragraph =
            ratatui_widgets::paragraph::Paragraph::new(self.text.into_ratatui_lossy())
                .alignment(self.alignment);
        if let Some(block) = self.block {
            paragraph = paragraph.block(block);
        }
        if let Some(wrap) = self.wrap {
            paragraph = paragraph.wrap(wrap);
        }
        paragraph
            .style(self.style)
            .scroll((self.scroll.y, self.scroll.x))
    }
}

impl Widget for HyperlinkedParagraph<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &HyperlinkedParagraph<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        buf.set_style(area, self.style);
        self.block.as_ref().render(area, buf);
        let inner = self.block.inner_if_some(area);
        self.render_paragraph(inner, buf);
    }
}

impl HyperlinkedParagraph<'_> {
    fn render_paragraph(&self, text_area: Rect, buf: &mut Buffer) {
        if text_area.is_empty() {
            return;
        }

        buf.set_style(text_area, self.style);

        let lines_iter = self.text.lines.iter().skip(self.scroll.y as usize);

        for (y, line) in lines_iter.enumerate().map(|(i, v)| (i as u16, v)) {
            if y >= text_area.height {
                break;
            }
            let line_area = Rect {
                x: text_area.x.saturating_sub(self.scroll.x),
                y: text_area.y + y,
                width: text_area.width.saturating_add(self.scroll.x),
                height: 1,
            };
            line.render_with_alignment(line_area, buf, Some(self.alignment));
        }
    }
}

impl Styled for HyperlinkedParagraph<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}
