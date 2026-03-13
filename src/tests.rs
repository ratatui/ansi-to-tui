use crate::{IntoText as _, hyperlink::*};
use pretty_assertions::assert_eq;
use ratatui_core::style::{Color, Style, Stylize};

#[test]
fn parses_plain_text_without_styles() {
    let string: Vec<u8> = "FOO".to_string().bytes().collect();
    test_both(string, HyperlinkedText::raw("FOO"));
}

#[test]
fn parses_unicode_text() {
    // These are 8 byte unicode characters.
    // First 4 bytes are for the unicode and the last 4 bytes are for the color / variant.
    let bytes = "AAA🅱️🅱️🅱️".as_bytes().to_vec();
    let output = HyperlinkedText::raw("AAA🅱️🅱️🅱️");
    test_both(bytes, output);
}

#[test]
fn preserves_empty_lines_when_splitting_on_newlines() {
    let bytes = "LINE_1\n\n\n\n\n\n\nLINE_8".as_bytes().to_vec();
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from("LINE_1"),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from("LINE_8"),
    ]);

    test_both(bytes, output);
}

#[test]
fn mixed_cr_and_lf_sequences_are_all_newlines() {
    let bytes = "A\r\n\rB\n\nC\r\r\nD".as_bytes().to_vec();
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from("A"),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from("B"),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from("C"),
        HyperlinkedLine::from(""),
        HyperlinkedLine::from("D"),
    ]);
    test_both(bytes, output);
}

#[test]
/// Treat `\r\n` as a single newline (CRLF).
///
/// This normalizes Windows line endings so the resulting `HyperlinkedText` is stable across platforms.
fn treats_crlf_as_single_newline() {
    let bytes = "LINE_1\r\nLINE_2\r\nLINE_3".as_bytes().to_vec();
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from("LINE_1"),
        HyperlinkedLine::from("LINE_2"),
        HyperlinkedLine::from("LINE_3"),
    ]);
    test_both(bytes, output);
}

#[test]
/// Treat bare `\r` as a newline.
///
/// This avoids embedding carriage returns into spans, and makes the output consistent with LF and
/// CRLF inputs.
fn treats_bare_cr_as_newline() {
    let bytes = "ABC\rDEF".as_bytes().to_vec();
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from("ABC"),
        HyperlinkedLine::from("DEF"),
    ]);
    test_both(bytes, output);
}

#[test]
/// Normalize mixed LF and CRLF into consistent line boundaries.
fn mixed_lf_and_crlf_line_endings_are_normalized() {
    let bytes = "A\nB\r\nC\nD\r\nE".as_bytes().to_vec();
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from("A"),
        HyperlinkedLine::from("B"),
        HyperlinkedLine::from("C"),
        HyperlinkedLine::from("D"),
        HyperlinkedLine::from("E"),
    ]);
    test_both(bytes, output);
}

#[test]
/// A CRLF-only input is a single empty line.
fn crlf_only_input_is_empty_line() {
    let bytes = "\r\n".as_bytes().to_vec();
    let output = HyperlinkedText::raw("");
    test_both(bytes, output);
}

#[test]
/// `\r` acts as a newline, even before non-SGR escape sequences.
///
/// This crate intentionally does not implement cursor movement/erase semantics; it only produces
/// styled text lines.
fn cr_before_non_sgr_escape_sequence_starts_new_line() {
    let bytes: Vec<u8> = b"\r\x1b[KOVERWRITE".to_vec();
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from(""),
        HyperlinkedLine::from("OVERWRITE"),
    ]);
    test_both(bytes, output);
}

#[test]
/// CRLF ends the line and style continues on the next line.
fn crlf_splits_lines_and_carries_style_across_lines() {
    let bytes: Vec<u8> = b"A\x1b[31mB\r\nC".to_vec();
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from(vec![HyperlinkedSpan::raw("A"), "B".red().into()]),
        HyperlinkedLine::from("C".red()),
    ]);
    test_both(bytes, output);
}

#[test]
fn ignores_truncated_escape_sequence() {
    let bytes = b"\x1b[";
    let output = HyperlinkedText::raw("");
    test_both(bytes, output);
}

#[test]
fn ignores_garbage_escape_sequences() {
    let bytes: Vec<u8> = b"\x1b\x1b[0\x1b[m\x1b".to_vec();
    let output = HyperlinkedText::raw("");
    test_both(bytes, output);
}

#[test]
fn ignores_non_sgr_escape_sequences() {
    let bytes: Vec<u8> = b"\x1b[?25hAAABBB".to_vec();
    let output = HyperlinkedText::raw("AAABBB");
    test_both(bytes, output);
}

#[test]
fn ignores_osc_and_other_non_sgr_sequences() {
    // Malformed -> malformed -> empty
    let bytes = b"\x1b[4 q\x1b]12;#fab1ed\x07";
    let output = HyperlinkedText::raw("");
    test_both(bytes, output);
}

#[test]
fn unknown_sgr_codes_are_ignored_and_chained_items_still_apply() {
    let bytes: Vec<u8> = b"\x1b[200;31mred".to_vec();
    let output = HyperlinkedText::from("red".red());
    test_both(bytes, output);
}

#[test]
fn empty_sgr_sequence_is_treated_as_reset() {
    let string = b"\x1b[32mGREEN\x1b[mFOO\nFOO";
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from(vec![
            "GREEN".green().into(),
            HyperlinkedSpan::styled("FOO", Style::reset()),
        ]),
        HyperlinkedLine::from(HyperlinkedSpan::styled("FOO", Style::reset())),
    ]);
    test_both(string, output);
}

#[test]
fn chained_sgr_items_in_single_escape_sequence_are_applied_in_order() {
    let bytes: Vec<u8> = b"\x1b[31;44;1mX".to_vec();
    let output = HyperlinkedText::from("X".red().on_blue().bold());
    test_both(bytes, output);
}

#[test]
fn does_not_emit_empty_spans_for_style_only_changes() {
    // Yellow -> Red -> Green -> "Hello" -> Reset -> "World"
    let bytes: Vec<u8> = b"\x1b[33m\x1b[31m\x1b[32mHello\x1b[0mWorld".to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        "Hello".green().into(),
        HyperlinkedSpan::styled("World", Style::reset()),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_0_resets_style() {
    let string = "\x1b[33mA\x1b[0mB";
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        "A".yellow().into(),
        HyperlinkedSpan::styled("B", Style::reset()),
    ]));
    test_both(string, output);
}

#[test]
fn sgr_1_and_22_toggle_bold() {
    let bytes = "not, \x1b[1mbold\x1b[22m, not anymore".as_bytes().to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "bold".bold().into(),
        ", not anymore".not_bold().not_dim().into(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_2_and_22_toggle_faint() {
    let bytes = "not, \x1b[2mfaint\x1b[22m, not anymore".as_bytes().to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "faint".dim().into(),
        ", not anymore".not_bold().not_dim().into(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_3_and_23_toggle_italic() {
    let bytes = "not, \x1b[3mitalic\x1b[23m, not anymore"
        .as_bytes()
        .to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "italic".italic().into(),
        ", not anymore".not_italic().into(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_4_and_24_toggle_underline() {
    let bytes = "not, \x1b[4munderlined\x1b[24m, not anymore"
        .as_bytes()
        .to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "underlined".underlined().into(),
        ", not anymore".not_underlined().into(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_5_and_25_toggle_slow_blink() {
    let bytes = "not, \x1b[5mblinking\x1b[25m, not anymore"
        .as_bytes()
        .to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "blinking".slow_blink().into(),
        ", not anymore".not_slow_blink().not_rapid_blink().into(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_6_and_25_toggle_rapid_blink() {
    let bytes = "not, \x1b[6mrapid\x1b[25m, not anymore".as_bytes().to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "rapid".rapid_blink().into(),
        ", not anymore".not_slow_blink().not_rapid_blink().into(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_7_and_27_toggle_reverse_video() {
    let bytes = "not, \x1b[7mreversed\x1b[27m, not anymore"
        .as_bytes()
        .to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "reversed".reversed().into(),
        ", not anymore".not_reversed().into(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_8_and_28_toggle_conceal() {
    let bytes = "not, \x1b[8mconcealed\x1b[28m, not anymore"
        .as_bytes()
        .to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "concealed".hidden().into(),
        ", not anymore".not_hidden().into(),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_9_and_29_toggle_crossed_out() {
    let bytes = "not, \x1b[9mcrossed\x1b[29m, not anymore"
        .as_bytes()
        .to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        HyperlinkedSpan::raw("not, "),
        "crossed".crossed_out().into(),
        ", not anymore".not_crossed_out().into(),
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

    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from("black".black()),
        HyperlinkedLine::from("red".red()),
        HyperlinkedLine::from("green".green()),
        HyperlinkedLine::from("yellow".yellow()),
        HyperlinkedLine::from("blue".blue()),
        HyperlinkedLine::from("magenta".magenta()),
        HyperlinkedLine::from("cyan".cyan()),
        HyperlinkedLine::from("gray".gray()),
        HyperlinkedLine::from("black-bg".black().on_black()),
        HyperlinkedLine::from("red-bg".black().on_red()),
        HyperlinkedLine::from("green-bg".black().on_green()),
        HyperlinkedLine::from("yellow-bg".black().on_yellow()),
        HyperlinkedLine::from("blue-bg".black().on_blue()),
        HyperlinkedLine::from("magenta-bg".black().on_magenta()),
        HyperlinkedLine::from("cyan-bg".black().on_cyan()),
        HyperlinkedLine::from("gray-bg".black().on_gray()),
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

    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from("dark-gray".dark_gray()),
        HyperlinkedLine::from("light-red".light_red()),
        HyperlinkedLine::from("light-green".light_green()),
        HyperlinkedLine::from("light-yellow".light_yellow()),
        HyperlinkedLine::from("light-blue".light_blue()),
        HyperlinkedLine::from("light-magenta".light_magenta()),
        HyperlinkedLine::from("light-cyan".light_cyan()),
        HyperlinkedLine::from("white".white()),
        HyperlinkedLine::from("dark-gray-bg".black().on_dark_gray()),
        HyperlinkedLine::from("light-red-bg".black().on_light_red()),
        HyperlinkedLine::from("light-green-bg".black().on_light_green()),
        HyperlinkedLine::from("light-yellow-bg".black().on_light_yellow()),
        HyperlinkedLine::from("light-blue-bg".black().on_light_blue()),
        HyperlinkedLine::from("light-magenta-bg".black().on_light_magenta()),
        HyperlinkedLine::from("light-cyan-bg".black().on_light_cyan()),
        HyperlinkedLine::from("white-bg".black().on_white()),
    ]);

    test_both(bytes, output);
}

#[test]
fn sgr_31_and_39_toggle_foreground_color() {
    let bytes: Vec<u8> = b"\x1b[31;1mred\x1b[39mdefault".to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        "red".red().bold(),
        "default".bold().fg(Color::Reset),
    ]));
    test_both(bytes, output);
}

#[test]
fn sgr_44_and_49_toggle_background_color() {
    let bytes: Vec<u8> = b"\x1b[44;1mblue-bg\x1b[49mdefault".to_vec();
    let output = HyperlinkedText::from(HyperlinkedLine::from(vec![
        "blue-bg".on_blue().bold(),
        "default".bold().bg(Color::Reset),
    ]));
    test_both(bytes, output);
}

#[test]
fn parses_256color_foreground_palette() {
    for i in 0..256 {
        let bytes = format!("\x1b[38;5;{}mHELLO", i).as_bytes().to_vec();
        let output = HyperlinkedText::from("HELLO".fg(Color::Indexed(i as u8)));
        test_both(bytes, output);
    }
}

#[test]
fn parses_256color_background_palette() {
    for i in 0..256 {
        let bytes = format!("\x1b[48;5;{}mHELLO", i).as_bytes().to_vec();
        let output = HyperlinkedText::from("HELLO".bg(Color::Indexed(i as u8)));
        test_both(bytes, output);
    }
}

#[test]
fn parses_truecolor_foreground() {
    let bytes: Vec<u8> = b"\x1b[38;2;100;100;100mAAABBB".to_vec();
    let output = HyperlinkedText::from("AAABBB".fg(Color::Rgb(100, 100, 100)));
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
        let output = HyperlinkedText::from(
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
    let output = HyperlinkedText::from(vec![
        HyperlinkedLine::from(vec![
            "* ".green().into(),
            HyperlinkedSpan::styled("Running before-startup command ", Style::reset()),
            HyperlinkedSpan::styled("command", Style::reset()).bold(),
            HyperlinkedSpan::styled("=make my-simple-package.cabal", Style::reset()),
        ]),
        HyperlinkedLine::from(vec![
            HyperlinkedSpan::styled("* ", Style::reset()).green(),
            HyperlinkedSpan::styled("$ make my-simple-package.cabal", Style::reset()),
        ]),
        HyperlinkedLine::from(vec![HyperlinkedSpan::styled(
            "Build profile: -w ghc-9.0.2 -O1",
            Style::reset(),
        )]),
    ]);
    test_both(bytes, output);
}

fn encode_osc8(label: &str, url: &str) -> String {
    format!("\u{1b}]8;;{url}\u{1b}\\{label}\u{1b}]8;;\u{1b}\\")
}

#[test]
fn integration_test_hyperlinks() {
    use crate::hyperlink::*;
    let bytes = encode_osc8("Google", "https://www.google.com");
    let text = bytes.into_text().expect("Failed to parse hyperlink text");
    let expected = HyperlinkedText::from(vec![HyperlinkedLine::from(vec![
        HyperlinkedSpan::hyperlink("Google", "https://www.google.com"),
    ])]);
    assert_eq!(text, expected);
}

#[track_caller]
fn test_both(bytes: impl AsRef<[u8]>, other: HyperlinkedText) {
    let bytes = bytes.as_ref();
    let owned = bytes.into_text().unwrap();
    assert_eq!(
        owned, other,
        "owned and other have diverged; this might be a bug in the library or a ratatui update"
    );
}
