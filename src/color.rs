use tui::style::Color;
#[derive(Debug, PartialEq)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,     // white
    DarkGray, // lightblack
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,           // lightwhite
    RGB(u8, u8, u8), // TrueColor
    Indexed(u8),     // 8-bit color
}

impl From<AnsiColor> for Color {
    fn from(ansi: AnsiColor) -> Self {
        match ansi {
            AnsiColor::Black => Color::Black,
            AnsiColor::Red => Color::Red,
            AnsiColor::Green => Color::Green,
            AnsiColor::Yellow => Color::Yellow,
            AnsiColor::Blue => Color::Blue,
            AnsiColor::Magenta => Color::Magenta,
            AnsiColor::Cyan => Color::Cyan,
            AnsiColor::Gray => Color::Gray,
            AnsiColor::DarkGray => Color::DarkGray,
            AnsiColor::LightRed => Color::LightRed,
            AnsiColor::LightGreen => Color::LightGreen,
            AnsiColor::LightYellow => Color::LightYellow,
            AnsiColor::LightBlue => Color::LightBlue,
            AnsiColor::LightMagenta => Color::LightMagenta,
            AnsiColor::LightCyan => Color::LightCyan,
            AnsiColor::White => Color::White,
            AnsiColor::RGB(r, g, b) => Color::Rgb(r, g, b),
            AnsiColor::Indexed(c) => Color::Indexed(c),
        }
    }
}
