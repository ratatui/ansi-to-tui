# ansi-to-tui

[![drone build](https://img.shields.io/drone/build/uttarayan/ansi-to-tui?server=https%3A%2F%2Fdrone.uttarayan.me)][mirror] [![github build](https://github.com/uttarayan21/ansi-to-tui/actions/workflows/build.yaml/badge.svg)][ansi-to-tui] [![downloads](https://img.shields.io/crates/d/ansi-to-tui)](https://crates.io/crates/ansi-to-tui)

Parse text with ANSI color codes and turn them into [`tui::text::Text`][Text].

|  Color  | Supported |          Examples        |
|   ---   |   :---:   |            ---           |
| 24 bit  |     ✓     | `\x1b[38;2;<R>;<G>;<B>m` |
| 8 bit   |     ✓     | `\x1b[38;5;<N>m`         |
| 4 bit   |     ✓     | `\x1b[30..37;40..47m`    |

## Example

```rust
use ansi_to_tui::ansi_to_text;
use std::io::Read;

let mut input = std::fs::File::open("ascii/text.ascii").unwrap();
let mut buffer: Vec<u8> = Vec::new();
file.read_to_end(&mut buffer);
let output = ansi_to_text(buffer);
```

<!-- A naive, yet relatively fast implementation with lots of room for improvement. -->

[Text]: https://docs.rs/tui/0.16.0/tui/text/struct.Text.html
[ansi-to-tui]: https://github.com/uttarayan21/ansi-to-tui
[mirror]: https://git.uttarayan.me/uttarayan/ansi-to-tui
