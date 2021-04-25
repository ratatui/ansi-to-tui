# ansi-to-tui

Parse text with ansi color codes and turn them into [`tui::text::Text`](https://docs.rs/tui/0.14.0/tui/text/struct.Text.html).

Supports TrueColor ( RGB ) ( `\x1b[38;2;<r>;<g>;<b>m`)  
Supports 8 - Bit Color ( 0..256 ) ( `\x1b[38;5;<n>m` )  
Supports 4 - Bit Color Pallete ( `\x1b[30..37;40..47m` )

A naive implementation, relatively fast.  
Only dependency is the tui crate.  
Lots of room for improvement.
