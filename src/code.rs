use crate::color::AnsiColor;

/// This enum stores most types of ansi escape sequences  
///
/// You can turn an escape sequence to this enum variant using
/// from(u8) trait.  
/// This doesn't support all of them but does support most of them.  

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum AnsiCode {
    Reset,
    Bold,
    Faint,
    Italic,
    Underline,
    SlowBlink,
    RapidBlink,
    Reverse,
    Conceal,
    CrossedOut,
    PrimaryFont,
    AlternateFont,
    AlternateFonts(u8), // = 11..19, // from 11 to 19
    Fraktur,
    BoldOff,
    Normal,
    NotItalic,
    UnderlineOff,
    BlinkOff,
    // 26 ?
    InvertOff,
    Reveal,
    CrossedOutOff,

    ForegroundColor(AnsiColor), //, 31..37//Issue 60553 https://github.com/rust-lang/rust/issues/60553
    SetForegroundColor,
    DefaultForegroundColor,
    BackgroundColor(AnsiColor), // 41..47
    SetBackgroundColor,
    DefaultBackgroundColor, // 49
}

impl From<u8> for AnsiCode {
    fn from(code: u8) -> Self {
        match code {
            0 => AnsiCode::Reset,
            1 => AnsiCode::Bold,
            2 => AnsiCode::Faint,
            3 => AnsiCode::Italic,
            4 => AnsiCode::Underline,
            5 => AnsiCode::SlowBlink,
            6 => AnsiCode::RapidBlink,
            7 => AnsiCode::Reverse,
            8 => AnsiCode::Conceal,
            9 => AnsiCode::CrossedOut,
            10 => AnsiCode::PrimaryFont,
            11 => AnsiCode::AlternateFont,
            // AnsiCode::// AlternateFont = 11..19, // from 11 to 19
            20 => AnsiCode::Fraktur,
            21 => AnsiCode::BoldOff,
            22 => AnsiCode::Normal,
            23 => AnsiCode::NotItalic,
            24 => AnsiCode::UnderlineOff,
            25 => AnsiCode::BlinkOff,
            // 26 ?
            27 => AnsiCode::InvertOff,
            28 => AnsiCode::Reveal,
            29 => AnsiCode::CrossedOutOff,
            30 => AnsiCode::ForegroundColor(AnsiColor::Black),
            31 => AnsiCode::ForegroundColor(AnsiColor::Red),
            32 => AnsiCode::ForegroundColor(AnsiColor::Green),
            33 => AnsiCode::ForegroundColor(AnsiColor::Yellow),
            34 => AnsiCode::ForegroundColor(AnsiColor::Blue),
            35 => AnsiCode::ForegroundColor(AnsiColor::Magenta),
            36 => AnsiCode::ForegroundColor(AnsiColor::Cyan),
            37 => AnsiCode::ForegroundColor(AnsiColor::Gray),
            38 => AnsiCode::SetForegroundColor,
            39 => AnsiCode::DefaultForegroundColor,
            40 => AnsiCode::BackgroundColor(AnsiColor::Black),
            41 => AnsiCode::BackgroundColor(AnsiColor::Red),
            42 => AnsiCode::BackgroundColor(AnsiColor::Green),
            43 => AnsiCode::BackgroundColor(AnsiColor::Yellow),
            44 => AnsiCode::BackgroundColor(AnsiColor::Blue),
            45 => AnsiCode::BackgroundColor(AnsiColor::Magenta),
            46 => AnsiCode::BackgroundColor(AnsiColor::Cyan),
            47 => AnsiCode::BackgroundColor(AnsiColor::Gray),
            48 => AnsiCode::SetBackgroundColor,
            49 => AnsiCode::DefaultBackgroundColor,
            _ => AnsiCode::Reset,
        }
    }
}
