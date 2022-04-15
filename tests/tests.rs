use ansi_to_tui::{ansi_to_text, ansi_to_text_override_style};
use pretty_assertions::assert_eq;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};

#[test]
fn test_anyhow() -> anyhow::Result<()> {
    let text = ansi_to_text("foo".bytes())?;
    println!("{:#?}", text);
    Ok(())
}

#[test]
#[ignore]
fn test_bytes() {
    let bytes: Vec<u8> = vec![27_u8, 91, 51, 49, 109, 65, 65, 65];
    println!("{:#?}", ansi_to_text(bytes))
}

#[test]
fn test_generic() {
    let string = "\x1b[33mYellow\x1b[31mRed\x1b[32mGreen\x1b[0m";
    println!("{:?}\n\n", string);

    let text = ansi_to_text(string.bytes()).unwrap();
    println!("{:#?}", &text);

    let expecting = Text::from(Spans::from(vec![
        Span::styled("Yellow", Style::default().fg(Color::Yellow)),
        Span::styled("Red", Style::default().fg(Color::Red)),
        Span::styled("Green", Style::default().fg(Color::Green)),
    ]));
    assert_eq!(text, expecting);
}

#[test]
fn test_string() {
    let string = "FOO".to_string();

    let text = ansi_to_text(string.bytes()).unwrap();
    println!("{:#?}", &text);

    let expecting = Text::raw("FOO");
    assert_eq!(text, expecting);
}

#[test]
fn test_unicode() {
    // these are 8 byte unicode charachters
    // first 4 bytes are for the unicode and the last 4 bytes are for the color / variant
    let bytes = "AAA🅱️🅱️🅱️".as_bytes().to_vec();

    let text = ansi_to_text(bytes).unwrap();
    println!("{:#?}", &text);

    let expecting = Text::raw("AAA🅱️🅱️🅱️");
    assert_eq!(text, expecting);
}

#[test]
fn test_ascii_rgb() {
    let bytes: Vec<u8> = b"\x1b[38;2;100;100;100mAAABBB".to_vec();

    let text = ansi_to_text(bytes).unwrap();
    println!("{:#?}", &text);

    let expecting = Text::styled("AAABBB", Style::default().fg(Color::Rgb(100, 100, 100)));
    assert_eq!(text, expecting);
}
#[test]
fn test_ascii_multi() {
    let bytes = "\x1b[31m\x1b[4m\x1b[1mHELLO".as_bytes().to_vec();

    let text = ansi_to_text(bytes).unwrap();
    println!("{:#?}", &text);

    let expecting = Text::styled(
        "HELLO",
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    );
    assert_eq!(text, expecting);
}
#[test]
fn test_ascii_newlines() {
    let bytes = "LINE_1\n\n\n\n\n\n\nLINE_8".as_bytes().to_vec();

    let text = ansi_to_text(bytes).unwrap();
    println!("{:#?}", &text);

    let expecting = Text::from(vec![
        Spans::from(vec![Span::styled("LINE_1", Style::default())]),
        Spans::from(vec![]),
        Spans::from(vec![]),
        Spans::from(vec![]),
        Spans::from(vec![]),
        Spans::from(vec![]),
        Spans::from(vec![]),
        Spans::from(vec![Span::styled("LINE_8", Style::default())]),
    ]);
    assert_eq!(text, expecting);
}

#[test]
#[ignore = "Gives a lot of output"]
fn test_arch_ascii() {
    use crate::ansi_to_text;
    use std::{fs::File, io::Read};
    let mut ascii = File::open("ascii/arch.ascii").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    ascii.read_to_end(&mut buffer).unwrap();
    let text = ansi_to_text(buffer).unwrap();
    println!("{:#?}", text);
}

#[test]
fn test_override_style() {
    use std::{fs::File, io::Read};
    let mut ascii = File::open("ascii/archlinux.ascii").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    ascii.read_to_end(&mut buffer).unwrap();
    let text = ansi_to_text_override_style(buffer, Style::default().fg(Color::Red)).unwrap();
    println!("{:#?}", text);
}

#[test]
#[ignore = "Gives a lot of output"]
fn test_archlinux_ascii() {
    use crate::ansi_to_text;
    use std::{fs::File, io::Read};
    let mut ascii = File::open("ascii/archlinux.ascii").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    ascii.read_to_end(&mut buffer).unwrap();
    let text = ansi_to_text(buffer).unwrap();
    println!("{:#?}", text);
}

#[test]
#[ignore]
fn test_command() {
    use std::process::Command;

    let c = Command::new("ls")
        .args(&["--color=always", "/"])
        .output()
        .unwrap();
    let text = ansi_to_text(c.stdout);
    println!("{:?}", text);
}

#[test]
fn test_reset() {
    let string = "\x1b[33mA\x1b[0mB";
    println!("{:?}\n\n", string);

    let text = ansi_to_text(string.bytes()).unwrap();
    println!("{:#?}", &text);

    let expecting = Text::from(Spans::from(vec![
        Span::styled("A", Style::default().fg(Color::Yellow)),
        Span::styled("B", Style::default()),
    ]));
    assert_eq!(text, expecting);
}
