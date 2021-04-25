// use crate::ansi::AnsiGraphicsStack;
// use crate::code::AnsiCode;
// use crate::color::AnsiColor;
// use tui::style::Style;

// pub struct AnsiStyle {
//     foreground: Option<AnsiColor>,
//     background: Option<AnsiColor>,
//     effect: Vec<u8>,
// }

// impl AnsiStyle {
//     pub fn new() -> Self {
//         Self {
//             foreground: None,
//             background: None,
//             effect: Vec::<u8>::new(),
//         }
//     }
// }
// impl From<AnsiStyle> for Style {
//     fn from(ansi_style: AnsiStyle) -> Self {
//         let style = Style::default();
//         todo!();
//     }
// }

// impl From<AnsiGraphicsStack> for AnsiStyle {
//     fn from(stack: AnsiGraphicsStack) -> Self {
//         AnsiStyle::new()
//         // for style in stack.iter() {
//         //     match style {
//         //         1..=11 | 20..=25 | 27..=29 | 38 | 39 => {
//         //             *style as AnsiCode;
//         //         }
//         //         _ => (),
//         //     }
//         // }
//     }
// }
