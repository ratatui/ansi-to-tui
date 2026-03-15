/// Renders a paragraph with embedded hyperlinks using the `HyperlinkedParagraph` widget.
///
/// This example shows how to use `HyperlinkedParagraph` with blocks, alignment, scrolling,
/// and styling. Click behaviour depends on your terminal emulator.
///
/// Controls:
/// - Up/Down arrows: scroll vertically
/// - 'c'/'l'/'r': center/left/right align
/// - 'q': quit
use std::io;

use ansi_to_tui::{HyperlinkedParagraph, HyperlinkedText, IntoText as _};
use anstyle::{AnsiColor, Color, Effects, Reset, Style as AnsiStyle};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Alignment,
    style::{Modifier, Style},
    widgets::{Block, Padding, Widget},
};
use ratatui_widgets::paragraph::Wrap;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let mut scroll = 0u16;
    let mut alignment = Alignment::Left;

    loop {
        terminal.draw(|f| draw(f, scroll, alignment))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => scroll = scroll.saturating_sub(1),
                    KeyCode::Down => scroll = scroll.saturating_add(1),
                    KeyCode::Char('c') => alignment = Alignment::Center,
                    KeyCode::Char('l') => alignment = Alignment::Left,
                    KeyCode::Char('r') => alignment = Alignment::Right,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn encode_osc8(label: &str, url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{label}\x1b]8;;\x1b\\")
}

fn styled(text: &str, style: AnsiStyle) -> String {
    format!("{style}{text}{Reset}")
}

fn link(label: &str, url: &str, color: AnsiColor) -> String {
    let style = AnsiStyle::new()
        .fg_color(Some(Color::Ansi(color)))
        .effects(Effects::BOLD);
    styled(&encode_osc8(label, url), style)
}

fn draw(frame: &mut Frame, scroll: u16, alignment: Alignment) {
    let area = frame.area();

    let [title_area, help_area, body_area] = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Length(1),
        ratatui::layout::Constraint::Length(1),
        ratatui::layout::Constraint::Min(0),
    ])
    .areas(area);

    frame.render_widget(
        ratatui::text::Line::styled(
            " HyperlinkedParagraph Demo",
            Style::default()
                .fg(ratatui::style::Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        title_area,
    );

    frame.render_widget(
        ratatui::text::Line::styled(
            " ↑/↓: scroll  c/l/r: align  q: quit",
            Style::default().fg(ratatui::style::Color::DarkGray),
        ),
        help_area,
    );

    let block = Block::bordered()
        .title(" Rust Programming Language Resources ")
        .title_bottom(format!(" Alignment: {:?} | Scroll: {} ", alignment, scroll))
        .padding(Padding::new(1, 1, 0, 0));
    let inner = block.inner(body_area);
    frame.render_widget(block, body_area);

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
        "The Rust Programming Language",
        "https://doc.rust-lang.org/book/",
        AnsiColor::Blue,
    );
    let nom = link(
        "nom",
        "https://github.com/rust-bakery/nom",
        AnsiColor::BrightBlue,
    );
    let serde = link("serde", "https://serde.rs", AnsiColor::BrightGreen);
    let tokio = link("tokio", "https://tokio.rs", AnsiColor::BrightCyan);
    let clap = link(
        "clap",
        "https://github.com/clap-rs/clap",
        AnsiColor::BrightYellow,
    );
    let rayon = link(
        "rayon",
        "https://github.com/rayon-rs/rayon",
        AnsiColor::BrightRed,
    );
    let anyhow_ = link("anyhow", "https://docs.rs/anyhow", AnsiColor::BrightMagenta);
    let thiserror = link(
        "thiserror",
        "https://docs.rs/thiserror",
        AnsiColor::BrightWhite,
    );

    let content = [
        format!(
            "{rust} is a systems programming language focused on safety, speed, and concurrency."
        ),
        format!("It empowers developers to build reliable and efficient software."),
        String::new(),
        format!("Getting Started with Rust:"),
        format!("  - Read {book} to learn the fundamentals."),
        format!("  - Install via rustup from the official website."),
        format!("  - Use `cargo new project_name` to start a new project."),
        String::new(),
        format!("Key Language Features:"),
        format!("  - Memory safety without garbage collection"),
        format!("  - Zero-cost abstractions for high performance"),
        format!("  - Fearless concurrency with ownership model"),
        format!("  - Pattern matching and algebraic data types"),
        format!("  - Trait-based generics for code reuse"),
        String::new(),
        format!("Popular Ecosystem Crates:"),
        format!("  - {serde}: A framework for serializing and deserializing data."),
        format!("  - {tokio}: An async runtime for writing reliable network applications."),
        format!("  - {clap}: A command-line argument parser that derives from structs."),
        format!("  - {rayon}: A data-parallelism library for easy parallel computation."),
        format!("  - {nom}: A parser combinators library for building efficient parsers."),
        format!("  - {anyhow_} & {thiserror}: Error handling utilities."),
        String::new(),
        format!("Building Terminal User Interfaces:"),
        format!("  - {ratatui} is a library for cooking up terminal UIs in Rust."),
        format!("  - Create widgets like paragraphs, charts, and tables."),
        format!("  - Handle keyboard and mouse events with ease."),
        format!("  - Supports ANSI styling and OSC 8 hyperlinks."),
        String::new(),
        format!("Resources and Community:"),
        format!("  - {crates}: The central registry for Rust packages."),
        format!("  - {docs}: Documentation hosting for all published crates."),
        format!("  - {github}: The official Rust language repository."),
        String::new(),
        format!("Try scrolling with arrow keys and changing alignment with c/l/r."),
    ]
    .join("\n");

    let text: HyperlinkedText<'_> = content.to_text().unwrap();

    let paragraph = HyperlinkedParagraph::new(text)
        .alignment(alignment)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: true });

    paragraph.render(inner, frame.buffer_mut());
}
