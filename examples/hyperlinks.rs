/// Renders a paragraph of text with embedded clickable OSC 8 hyperlinks.
///
/// The text is built as a plain string with `encode_osc8` calls spliced in,
/// styled with `anstyle`, then parsed with `to_text_hyperlinked()` and rendered
/// as a `HyperlinkedText` widget.  Click behaviour depends on your terminal
/// emulator (kitty, WezTerm, iTerm2, foot, etc.).
use std::io;

use ansi_to_tui::IntoText as _;
use anstyle::{AnsiColor, Color, Effects, Reset, Style as AnsiStyle};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Padding, Widget},
};

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(draw)?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                break;
            }
        }
    }
    Ok(())
}

/// Encode a single OSC 8 hyperlink: the terminal displays `label` but clicking
/// it opens `url`.
fn encode_osc8(label: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\")
}

/// Wrap `text` in an anstyle style, returning the rendered ANSI string.
fn styled(text: &str, style: AnsiStyle) -> String {
    format!("{style}{text}{Reset}")
}

/// Convenience: bold + underlined link in the given color.
fn link(label: &str, url: &str, color: AnsiColor) -> String {
    let style = AnsiStyle::new()
        .fg_color(Some(Color::Ansi(color)))
        .effects(Effects::BOLD);
    styled(&encode_osc8(label, url), style)
}

fn draw(frame: &mut Frame) {
    let area = frame.area();

    let [title_area, body_area] =
        Layout::vertical([Constraint::Length(2), Constraint::Min(0)]).areas(area);

    frame.render_widget(
        Line::styled(
            " Hyperlinks Demo  (press q to quit)",
            Style::default()
                .fg(ratatui::style::Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        title_area,
    );

    let block = Block::bordered()
        .title(" About Rust ")
        .padding(Padding::new(2, 2, 1, 1));
    let inner = block.inner(body_area);
    frame.render_widget(block, body_area);

    // Build a multi-line paragraph with inline hyperlinks and ANSI colors.
    let rust = link("Rust", "https://www.rust-lang.org", AnsiColor::Red);
    let ratatui = link("Ratatui", "https://ratatui.rs", AnsiColor::Cyan);
    let crates = link("crates.io", "https://crates.io", AnsiColor::Yellow);
    let docs = link("docs.rs", "https://docs.rs", AnsiColor::Green);
    let github = link(
        "GitHub",
        "https://github.com/rust-lang/rust",
        AnsiColor::Magenta,
    );
    let book = link(
        "The Rust Book",
        "https://doc.rust-lang.org/book/",
        AnsiColor::Blue,
    );

    let paragraph = [
        format!("{rust} is a systems programming language focused on safety, speed,"),
        format!("and concurrency. You can learn more by reading {book}."),
        String::new(),
        format!("This example is built with {ratatui}, a library for cooking up"),
        format!("terminal user interfaces. Browse its source on {github}."),
        String::new(),
        format!("Discover community crates at {crates} and read their"),
        format!("documentation on {docs}."),
    ]
    .join("\n");

    match paragraph.to_text() {
        Ok(text) => text.render(inner, frame.buffer_mut()),
        Err(_) => frame.render_widget(Line::raw("(failed to parse)"), inner),
    }
}
