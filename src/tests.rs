#[cfg(test)]
#[test]
fn test_color() {
    use crate::stack::Stack;
    use tui::style::Color;
    let mut stack: Stack<u8> = Stack::new();
    stack.push(30);
    assert_eq!(stack.parse_color().unwrap(), Color::Indexed(30));
    stack.push(30);
    stack.push(3);
    stack.push(55);
    assert_eq!(stack.parse_color().unwrap(), Color::Rgb(30, 3, 55));
}
