# ansi-to-tui

[![Build & Tests](https://github.com/uttarayan21/ansi-to-tui/actions/workflows/build.yaml/badge.svg)][ansi-to-tui]

Parse text with ANSI color codes and turn them into [`tui::text::Text`][Text].

|  Color  | Supported |          Examples        |
|   ---   |    ---    |            ---           |
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

A naive, yet relatively fast implementation with lots of room for improvement.

[Text]: https://docs.rs/tui/0.16.0/tui/text/struct.Text.html
[ansi-to-tui]: https://github.com/uttarayan21/ansi-to-tui