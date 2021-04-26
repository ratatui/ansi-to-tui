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

#[test]
fn test_bytes() {
    use crate::ansi_to_text;
    let bytes = vec![27_u8, 91, 51, 49, 109, 65, 65, 65];
    println!("{:#?}", ansi_to_text(&bytes))
}

#[test]
fn archlinux_ascii() {
    use crate::ansi_to_text;
    use std::{fs::File, io::Read};
    let mut ascii = File::open("tests/archlinux.ascii").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    ascii.read_to_end(&mut buffer).unwrap();
    let text = ansi_to_text(&buffer).unwrap();
    for line in text.lines {
        println!("{:?}", line.width());
    }
}
#[test]
fn ascii_test() {
    use crate::ansi_to_text;
    use std::{fs::File, io::Read};
    let mut ascii = File::open("tests/ascii").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    ascii.read_to_end(&mut buffer).unwrap();
    let text = ansi_to_text(&buffer).unwrap();
    println!("lines {:#?}", text);
}
