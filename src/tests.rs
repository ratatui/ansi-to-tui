use crate::IntoText as _;
use pretty_assertions::assert_eq;
use ratatui_core::{
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
};

#[test]
fn parses_plain_text_without_styles() {
    let string: Vec<u8> = "FOO".to_string().bytes().collect();
    test_both(string, Text::raw("FOO"));
}

#[test]
fn parses_unicode_text() {
    // These are 8 byte unicode characters.
    // First 4 bytes are for the unicode and the last 4 bytes are for the color / variant.
    let bytes = "AAA🅱️🅱️🅱️".as_bytes().to_vec();
    let output = Text::raw("AAA🅱️🅱️🅱️");
    test_both(bytes, output);
}

#[test]
fn preserves_empty_lines_when_splitting_on_newlines() {
    let bytes = "LINE_1\n\n\n\n\n\n\nLINE_8".as_bytes().to_vec();
    let output = Text::from(vec![
        Line::from("LINE_1"),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from("LINE_8"),
    ]);

    test_both(bytes, output);
}

#[test]
/// Characterization: this parser only treats `\n` as a line break.
///
/// This matches the current implementation (`take_while(|c| c != b'\n')`) and means `\r` is
/// preserved as a literal character when parsing Windows line endings (`\r\n`).
///
/// Whether this is "correct" depends on the caller:
/// - For files with CRLF, callers may want to pre-normalize line endings.
/// - For terminal output, `\r` can be meaningful (carriage return without a newline).
fn splits_only_on_lf_and_preserves_cr_in_crlf() {
    let bytes = "LINE_1\r\nLINE_2\r\nLINE_3".as_bytes().to_vec();
    let output = Text::from(vec![
        Line::from("LINE_1\r"),
        Line::from("LINE_2\r"),
        Line::from("LINE_3"),
    ]);
    test_both(bytes, output);
}

#[test]
/// Characterization: `\r` is not treated as a newline.
///
/// This is consistent with terminals using `\r` as "carriage return" for in-place updates, and it
/// keeps the parser from guessing at terminal semantics that Ratatui's `Text` cannot represent.
fn lone_cr_is_not_treated_as_a_newline() {
    let bytes = "ABC\rDEF".as_bytes().to_vec();
    let output = Text::raw("ABC\rDEF");
    test_both(bytes, output);
}

#[test]
/// Characterization: mixed `\n`/`\r\n` line endings keep the `\r` on CRLF lines.
///
/// This locks in the current behavior so that any future "normalize CRLF" change is explicit and
/// comes with updated expectations.
fn mixed_lf_and_crlf_line_endings_preserve_cr() {
    let bytes = "A\nB\r\nC\nD\r\nE".as_bytes().to_vec();
    let output = Text::from(vec![
        Line::from("A"),
        Line::from("B\r"),
        Line::from("C"),
        Line::from("D\r"),
        Line::from("E"),
    ]);
    test_both(bytes, output);
}

#[test]
/// Characterization: a CRLF-only input produces a single line containing `"\r"`.
///
/// If we ever decide to normalize CRLF to LF, this would become an empty line instead.
fn crlf_blank_line_is_parsed_as_a_carriage_return() {
    let bytes = "\r\n".as_bytes().to_vec();
    let output = Text::from(Line::from("\r"));
    test_both(bytes, output);
}

#[test]
/// Characterization: `\r` is preserved even when followed by non-SGR escape sequences.
///
/// Many CLIs use `\r` (return to start of line) + `ESC[K` (erase to end of line) for progress
/// updates. This crate currently ignores `ESC[K` and does not implement cursor/erase semantics, so
/// the best it can do is preserve the raw `\r` and keep the visible text.
fn cr_before_non_sgr_escape_sequence_is_preserved() {
    let bytes: Vec<u8> = b"\r\x1b[KOVERWRITE".to_vec();
    let output = Text::from(Line::from(vec![Span::raw("\r"), Span::raw("OVERWRITE")]));
    test_both(bytes, output);
}

#[test]
/// Characterization: `\r` stays inside the current span, and the style carries across the `\n`.
///
/// This documents two behaviors:
/// - CRLF does not end (or strip) the current styled text; the `\r` is part of the span content.
/// - The "current style" is carried across lines, which matches how this crate renders streams of
///   terminal output.
fn crlf_preserves_cr_in_spans_and_carries_style_across_lines() {
    let bytes: Vec<u8> = b"A\x1b[31mB\r\nC".to_vec();
    let output = Text::from(vec![
        Line::from(vec![Span::raw("A"), "B\r".red()]),
        Line::from("C".red()),
    ]);
    test_both(bytes, output);
}

#[test]
fn ignores_truncated_escape_sequence() {
    let bytes = b"\x1b[";
    let output = Text::raw("");
    test_both(bytes, output);
}

#[test]
fn ignores_garbage_escape_sequences() {
    let bytes: Vec<u8> = b"\x1b\x1b[0\x1b[m\x1b".to_vec();
    let output = Text::raw("");
    test_both(bytes, output);
}

#[test]
fn ignores_non_sgr_escape_sequences() {
    let bytes: Vec<u8> = b"\x1b[?25hAAABBB".to_vec();
    let output = Text::raw("AAABBB");
    test_both(bytes, output);
}

#[test]
fn ignores_osc_and_other_non_sgr_sequences() {
    // Malformed -> malformed -> empty
    let bytes = b"\x1b[4 q\x1b]12;#fab1ed\x07";
    let output = Text::raw("");
    test_both(bytes, output);
}

#[test]
fn unknown_sgr_codes_are_ignored_and_chained_items_still_apply() {
    let bytes: Vec<u8> = b"\x1b[200;31mred".to_vec();
    let output = Text::from("red".red());
    test_both(bytes, output);
}

#[test]
fn empty_sgr_sequence_is_treated_as_reset() {
    let string = b"\x1b[32mGREEN\x1b[mFOO\nFOO";
    let output = Text::from(vec![
        Line::from(vec!["GREEN".green(), Span::styled("FOO", Style::reset())]),
        Line::from(Span::styled("FOO", Style::reset())),
    ]);
    test_both(string, output);
}

#[test]
fn chained_sgr_items_in_single_escape_sequence_are_applied_in_order() {
    let bytes: Vec<u8> = b"\x1b[31;44;1mX".to_vec();
    let output = Text::from("X".red().on_blue().bold());
    test_both(bytes, output);
}

#[test]
fn does_not_emit_empty_spans_for_style_only_changes() {
    // Yellow -> Red -> Green -> "Hello" -> Reset -> "World"
    let bytes: Vec<u8> = b"\x1b[33m\x1b[31m\x1b[32mHello\x1b[0mWorld".to_vec();
    let output = Text::from(Line::from(vec![
        "Hello".green(),
        Span::styled("World", Style::reset()),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_0_resets_style() {
    let string = "\x1b[33mA\x1b[0mB";
    let output = Text::from(Line::from(vec![
        "A".yellow(),
        Span::styled("B", Style::reset()),
    ]));
    test_both(string, output);
}

#[test]
fn sgr_1_and_22_toggle_bold() {
    let bytes = "not, \x1b[1mbold\x1b[22m, not anymore".as_bytes().to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "bold".bold(),
        ", not anymore".not_bold().not_dim(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_2_and_22_toggle_faint() {
    let bytes = "not, \x1b[2mfaint\x1b[22m, not anymore".as_bytes().to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "faint".dim(),
        ", not anymore".not_bold().not_dim(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_3_and_23_toggle_italic() {
    let bytes = "not, \x1b[3mitalic\x1b[23m, not anymore"
        .as_bytes()
        .to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "italic".italic(),
        ", not anymore".not_italic(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_4_and_24_toggle_underline() {
    let bytes = "not, \x1b[4munderlined\x1b[24m, not anymore"
        .as_bytes()
        .to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "underlined".underlined(),
        ", not anymore".not_underlined(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_5_and_25_toggle_slow_blink() {
    let bytes = "not, \x1b[5mblinking\x1b[25m, not anymore"
        .as_bytes()
        .to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "blinking".slow_blink(),
        ", not anymore".not_slow_blink().not_rapid_blink(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_6_and_25_toggle_rapid_blink() {
    let bytes = "not, \x1b[6mrapid\x1b[25m, not anymore".as_bytes().to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "rapid".rapid_blink(),
        ", not anymore".not_slow_blink().not_rapid_blink(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_7_and_27_toggle_reverse_video() {
    let bytes = "not, \x1b[7mreversed\x1b[27m, not anymore"
        .as_bytes()
        .to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "reversed".reversed(),
        ", not anymore".not_reversed(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_8_and_28_toggle_conceal() {
    let bytes = "not, \x1b[8mconcealed\x1b[28m, not anymore"
        .as_bytes()
        .to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "concealed".hidden(),
        ", not anymore".not_hidden(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_9_and_29_toggle_crossed_out() {
    let bytes = "not, \x1b[9mcrossed\x1b[29m, not anymore"
        .as_bytes()
        .to_vec();
    let output = Text::from(Line::from(vec![
        Span::raw("not, "),
        "crossed".crossed_out(),
        ", not anymore".not_crossed_out(),
    ]));
    test_both(bytes, output);
}

#[test]
fn parses_4bit_named_colors_and_backgrounds() {
    const BLACK: &str = "\x1b[30m";
    const RED: &str = "\x1b[31m";
    const GREEN: &str = "\x1b[32m";
    const YELLOW: &str = "\x1b[33m";
    const BLUE: &str = "\x1b[34m";
    const MAGENTA: &str = "\x1b[35m";
    const CYAN: &str = "\x1b[36m";
    const GRAY: &str = "\x1b[37m";

    const BLACK_BG: &str = "\x1b[40m";
    const RED_BG: &str = "\x1b[41m";
    const GREEN_BG: &str = "\x1b[42m";
    const YELLOW_BG: &str = "\x1b[43m";
    const BLUE_BG: &str = "\x1b[44m";
    const MAGENTA_BG: &str = "\x1b[45m";
    const CYAN_BG: &str = "\x1b[46m";
    const GRAY_BG: &str = "\x1b[47m";

    let bytes = format!(
        "{BLACK}black\n\
            {RED}red\n\
            {GREEN}green\n\
            {YELLOW}yellow\n\
            {BLUE}blue\n\
            {MAGENTA}magenta\n\
            {CYAN}cyan\n\
            {GRAY}gray\n\
            {BLACK}{BLACK_BG}black-bg\n\
            {RED_BG}red-bg\n\
            {GREEN_BG}green-bg\n\
            {YELLOW_BG}yellow-bg\n\
            {BLUE_BG}blue-bg\n\
            {MAGENTA_BG}magenta-bg\n\
            {CYAN_BG}cyan-bg\n\
            {GRAY_BG}gray-bg"
    )
    .into_bytes();

    let output = Text::from(vec![
        Line::from("black".black()),
        Line::from("red".red()),
        Line::from("green".green()),
        Line::from("yellow".yellow()),
        Line::from("blue".blue()),
        Line::from("magenta".magenta()),
        Line::from("cyan".cyan()),
        Line::from("gray".gray()),
        Line::from("black-bg".black().on_black()),
        Line::from("red-bg".black().on_red()),
        Line::from("green-bg".black().on_green()),
        Line::from("yellow-bg".black().on_yellow()),
        Line::from("blue-bg".black().on_blue()),
        Line::from("magenta-bg".black().on_magenta()),
        Line::from("cyan-bg".black().on_cyan()),
        Line::from("gray-bg".black().on_gray()),
    ]);

    test_both(bytes, output);
}

#[test]
fn parses_4bit_bright_colors_and_backgrounds() {
    const DARK_GRAY: &str = "\x1b[90m";
    const LIGHT_RED: &str = "\x1b[91m";
    const LIGHT_GREEN: &str = "\x1b[92m";
    const LIGHT_YELLOW: &str = "\x1b[93m";
    const LIGHT_BLUE: &str = "\x1b[94m";
    const LIGHT_MAGENTA: &str = "\x1b[95m";
    const LIGHT_CYAN: &str = "\x1b[96m";
    const WHITE: &str = "\x1b[97m";

    const DARK_GRAY_BG: &str = "\x1b[100m";
    const LIGHT_RED_BG: &str = "\x1b[101m";
    const LIGHT_GREEN_BG: &str = "\x1b[102m";
    const LIGHT_YELLOW_BG: &str = "\x1b[103m";
    const LIGHT_BLUE_BG: &str = "\x1b[104m";
    const LIGHT_MAGENTA_BG: &str = "\x1b[105m";
    const LIGHT_CYAN_BG: &str = "\x1b[106m";
    const WHITE_BG: &str = "\x1b[107m";

    let bytes = format!(
        "{DARK_GRAY}dark-gray\n\
            {LIGHT_RED}light-red\n\
            {LIGHT_GREEN}light-green\n\
            {LIGHT_YELLOW}light-yellow\n\
            {LIGHT_BLUE}light-blue\n\
            {LIGHT_MAGENTA}light-magenta\n\
            {LIGHT_CYAN}light-cyan\n\
            {WHITE}white\n\
            \x1b[30m{DARK_GRAY_BG}dark-gray-bg\n\
            {LIGHT_RED_BG}light-red-bg\n\
            {LIGHT_GREEN_BG}light-green-bg\n\
            {LIGHT_YELLOW_BG}light-yellow-bg\n\
            {LIGHT_BLUE_BG}light-blue-bg\n\
            {LIGHT_MAGENTA_BG}light-magenta-bg\n\
            {LIGHT_CYAN_BG}light-cyan-bg\n\
            {WHITE_BG}white-bg"
    )
    .into_bytes();

    let output = Text::from(vec![
        Line::from("dark-gray".dark_gray()),
        Line::from("light-red".light_red()),
        Line::from("light-green".light_green()),
        Line::from("light-yellow".light_yellow()),
        Line::from("light-blue".light_blue()),
        Line::from("light-magenta".light_magenta()),
        Line::from("light-cyan".light_cyan()),
        Line::from("white".white()),
        Line::from("dark-gray-bg".black().on_dark_gray()),
        Line::from("light-red-bg".black().on_light_red()),
        Line::from("light-green-bg".black().on_light_green()),
        Line::from("light-yellow-bg".black().on_light_yellow()),
        Line::from("light-blue-bg".black().on_light_blue()),
        Line::from("light-magenta-bg".black().on_light_magenta()),
        Line::from("light-cyan-bg".black().on_light_cyan()),
        Line::from("white-bg".black().on_white()),
    ]);

    test_both(bytes, output);
}

#[test]
fn sgr_31_and_39_toggle_foreground_color() {
    let bytes: Vec<u8> = b"\x1b[31;1mred\x1b[39mdefault".to_vec();
    let output = Text::from(Line::from(vec![
        "red".red().bold(),
        "default".bold().fg(Color::Reset),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_44_and_49_toggle_background_color() {
    let bytes: Vec<u8> = b"\x1b[44;1mblue-bg\x1b[49mdefault".to_vec();
    let output = Text::from(Line::from(vec![
        "blue-bg".on_blue().bold(),
        "default".bold().bg(Color::Reset),
    ]));
    test_both(bytes, output);
}

#[test]
fn parses_256color_foreground_palette() {
    for i in 0..256 {
        let bytes = format!("\x1b[38;5;{}mHELLO", i).as_bytes().to_vec();
        let output = Text::from("HELLO".fg(Color::Indexed(i as u8)));
        test_both(bytes, output);
    }
}

#[test]
fn parses_256color_background_palette() {
    for i in 0..256 {
        let bytes = format!("\x1b[48;5;{}mHELLO", i).as_bytes().to_vec();
        let output = Text::from("HELLO".bg(Color::Indexed(i as u8)));
        test_both(bytes, output);
    }
}

#[test]
fn parses_truecolor_foreground() {
    let bytes: Vec<u8> = b"\x1b[38;2;100;100;100mAAABBB".to_vec();
    let output = Text::from("AAABBB".fg(Color::Rgb(100, 100, 100)));
    test_both(bytes, output);
}

#[test]
fn parses_truecolor_foreground_and_background() {
    let test_cases = [
        ((1, 2, 3), (4, 5, 6)),
        ((255, 0, 128), (0, 64, 255)),
        ((17, 34, 51), (68, 85, 102)),
    ];

    for ((fr, fg, fb), (br, bg, bb)) in test_cases {
        let bytes = format!("\x1b[38;2;{fr};{fg};{fb};48;2;{br};{bg};{bb}mHELLO")
            .as_bytes()
            .to_vec();
        let output = Text::from(
            "HELLO"
                .fg(Color::Rgb(fr, fg, fb))
                .bg(Color::Rgb(br, bg, bb)),
        );
        test_both(bytes, output);
    }
}

#[test]
fn carries_style_across_lines_and_handles_resets() {
    let bytes: Vec<u8> = String::from(
        "\u{1b}[32m* \u{1b}[0mRunning before-startup command \u{1b}[1mcommand\u{1b}[0m=make my-simple-package.cabal\n\
            \u{1b}[32m* \u{1b}[0m$ make my-simple-package.cabal\n\
            Build profile: -w ghc-9.0.2 -O1\n",
    )
    .into_bytes();
    let output = Text::from(vec![
        Line::from(vec![
            "* ".green(),
            Span::styled("Running before-startup command ", Style::reset()),
            Span::styled("command", Style::reset()).bold(),
            Span::styled("=make my-simple-package.cabal", Style::reset()),
        ]),
        Line::from(vec![
            Span::styled("* ", Style::reset()).green(),
            Span::styled("$ make my-simple-package.cabal", Style::reset()),
        ]),
        Line::from(vec![Span::styled(
            "Build profile: -w ghc-9.0.2 -O1",
            Style::reset(),
        )]),
    ]);
    test_both(bytes, output);
}

#[track_caller]
fn test_both(bytes: impl AsRef<[u8]>, other: Text) {
    let bytes = bytes.as_ref();

    #[cfg(feature = "zero-copy")]
    let zero_copy = bytes.to_text().unwrap();

    let owned = bytes.into_text().unwrap();

    #[cfg(feature = "zero-copy")]
    assert_eq!(
        zero_copy, owned,
        "zero-copy and owned version of the methods have diverged; this is a bug in the library"
    );

    assert_eq!(
        owned, other,
        "owned and other have diverged; this might be a bug in the library or a ratatui update"
    );

    #[cfg(feature = "zero-copy")]
    assert_eq!(zero_copy, other);
}
