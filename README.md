# ansi-to-tui

[![Build & Tests](https://github.com/uttarayan21/ansi-to-tui/actions/workflows/build.yaml/badge.svg)][ansi-to-tui]

Parse text with ansi color codes and turn them into [`tui::text::Text`][Text].

Supports TrueColor ( RGB ) ( `\x1b[38;2;<r>;<g>;<b>m`)  
Supports 8 - Bit Color ( 0..256 ) ( `\x1b[38;5;<n>m` )  
Supports 4 - Bit Color Pallete ( `\x1b[30..37;40..47m` )

Example

```rust
use ansi_to_tui::ansi_to_text;
use std::io::Read;

let mut file = std::fs::File::open("ascii/text.ascii").unwrap();
let mut buffer: Vec<u8> = Vec::new();
file.read_to_end(&mut buffer);
let text = ansi_to_text(buffer);
```

A naive implementation, relatively fast.  
Probably lots of room for improvement.  


[Text]: https://docs.rs/tui/0.15.0/tui/text/struct.Text.html
[ansi-to-tui]: https://github.com/uttarayan21/ansi-to-tui
