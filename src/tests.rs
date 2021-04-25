#[cfg(test)]
#[test]
fn test_color() {
    use crate::ansi_to_color;
    use tui::style::Color;
    assert_eq!(ansi_to_color(31_u8), Color::Red);
}
