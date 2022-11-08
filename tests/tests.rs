// use ansi_to_tui::{ansi_to_text, ansi_to_text_override_style};
use ansi_to_tui::IntoText;
use tui::{
    style::{Color, Style},
    text::{Span, Spans, Text},
};

// #[test]
// fn test_anyhow() -> anyhow::Result<()> {
//     let text = ansi_to_text("foo".bytes())?;
//     println!("{:#?}", text);
//     Ok(())
// }

// #[test]
// #[ignore]
// fn test_bytes() {
//     let bytes: Vec<u8> = vec![27_u8, 91, 51, 49, 109, 65, 65, 65];
//     println!("{:#?}", ansi_to_text(bytes))
// }

// #[test]
// fn test_generic() {
//     let string = "\x1b[33mYellow\x1b[31mRed\x1b[32mGreen\x1b[0m";
//     println!("{:?}\n\n", string);
//     // ansi_to_text(string.bytes()).unwrap();
//     println!("{:#?}", ansi_to_text(string.bytes()));
// }

#[test]
fn test_string() {
    use ansi_to_tui::IntoText;
    let string: Vec<u8> = "FOO".to_string().bytes().collect();
    println!("{:?}", string.into_text().unwrap());
}

#[test]
fn test_unicode() {
    // these are 8 byte unicode charachters
    // first 4 bytes are for the unicode and the last 4 bytes are for the color / variant
    let bytes = "AAA🅱️🅱️🅱️".as_bytes().to_vec();
    let output = some_text("AAA🅱️🅱️🅱️");
    assert_eq!(bytes.into_text(), output);
}

#[test]
fn test_ascii_rgb() {
    let bytes: Vec<u8> = b"\x1b[38;2;100;100;100mAAABBB".to_vec();
    let output = Ok(Text::from(Spans::from(Span::styled(
        "AAABBB",
        Style {
            fg: Some(Color::Rgb(100, 100, 100)),
            ..Default::default()
        },
    ))));
    assert_eq!(bytes.into_text(), output);
}

// #[test]
// fn test_ascii_multi() {
//     let bytes = "\x1b[31m\x1b[4m\x1b[1mHELLO".as_bytes().to_vec();
//     println!("{:#?}", ansi_to_text(bytes));
// }

#[test]
fn test_ascii_newlines() {
    let bytes = "LINE_1\n\n\n\n\n\n\nLINE_8".as_bytes().to_vec();
    let output = Ok(Text::from(vec![
        Spans::from(Span::raw("LINE_1")),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("LINE_8")),
    ]));

    // println!("{:#?}", bytes.into_text());
    assert_eq!(bytes.into_text(), output);
}

// #[test]
// #[ignore = "Gives a lot of output"]
// fn test_arch_ascii() {
//     use crate::ansi_to_text;
//     use std::{fs::File, io::Read};
//     let mut ascii = File::open("ascii/arch.ascii").unwrap();
//     let mut buffer: Vec<u8> = Vec::new();
//     ascii.read_to_end(&mut buffer).unwrap();
//     let text = ansi_to_text(buffer).unwrap();
//     println!("{:#?}", text);
// }

// #[test]
// #[ignore = "Gives a lot of output"]
// fn test_archlinux_ascii() {
//     use crate::ansi_to_text;
//     use std::{fs::File, io::Read};
//     let mut ascii = File::open("ascii/archlinux.ascii").unwrap();
//     let mut buffer: Vec<u8> = Vec::new();
//     ascii.read_to_end(&mut buffer).unwrap();
//     let text = ansi_to_text(buffer).unwrap();
//     println!("{:#?}", text);
// }

// #[test]
// #[ignore]
// fn test_command() {
//     use std::process::Command;

//     let c = Command::new("ls")
//         .args(&["--color=always", "/"])
//         .output()
//         .unwrap();
//     let text = ansi_to_text(c.stdout);
//     println!("{:?}", text);
// }

#[test]
fn test_reset() {
    let string = "\x1b[33mA\x1b[0mB";
    let output = Ok(Text {
        lines: vec![Spans(vec![
            Span::styled(
                "A",
                Style {
                    fg: Some(Color::Yellow),
                    ..Default::default()
                },
            ),
            Span::styled(
                "B",
                Style {
                    ..Default::default()
                },
            ),
        ])],
    });
    assert_eq!(string.into_text(), output);
}

#[test]
fn test_screen_modes() {
    let bytes: Vec<u8> = b"\x1b[?25hAAABBB".to_vec();
    let output = Ok(Text::from(Spans::from(Span::styled(
        "AAABBB", // or "AAABBB"
        Style::default(),
    ))));
    assert_eq!(bytes.into_text(), output);
}

#[test]
fn test_cursor_shape_and_color() {
    let bytes: Vec<u8> = b"\x1b[4 q\x1b]12;#fab1ed\x07".to_vec();
    let output = Ok(Text::from(Spans::from(Span::styled("", Style::default()))));
    assert_eq!(bytes.into_text(), output);
}

#[test]
fn test_malformed_simple() {
    let bytes: Vec<u8> = b"\x1b[".to_vec();
    let output = Ok(Text::from(Spans::from(Span::styled("", Style::default()))));
    assert_eq!(bytes.into_text(), output);
}

#[test]
fn test_malformed_complex() {
    let bytes: Vec<u8> = b"\x1b\x1b[0\x1b[m\x1b".to_vec();
    let output = Ok(Text::from(Spans::from(Span::styled("", Style::default()))));
    assert_eq!(bytes.into_text(), output);
}

fn some_text(s: &'static str) -> Result<Text<'static>, ansi_to_tui::Error> {
    Ok(Text {
        lines: vec![Spans(vec![Span {
            content: s.into(),
            style: Default::default(),
        }])],
    })
}
