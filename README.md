# ansi-to-tui

[![Crate Badge]][Crate] [![Repo Badge]][Repo]  \
[![Docs Badge]][Docs] [![License Badge]][License]  \
[![CI Badge]][CI] [![Codecov Badge]][Codecov]

Convert ANSI color and style codes into Ratatui [`Text`][Text].

This crate parses bytes containing ANSI SGR escape sequences (like `\x1b[31m`).
It produces a Ratatui [`Text`][Text] with equivalent foreground/background [`Color`][Color] and
[`Modifier`][Modifier] settings via [`Style`][Style].

Unknown or malformed escape sequences are ignored so you can feed it real terminal output without
having to pre-clean it.

| Color Mode                  | Supported | SGR Example              | Ratatui `Color` Example |
| --------------------------- | :-------: | ------------------------ | ----------------------- |
| Named (3/4-bit, 8/16-color) |     ✓     | `\x1b[30..37;40..47m`    | `Color::Blue`           |
| Indexed (8-bit, 256-color)  |     ✓     | `\x1b[38;5;<N>m`         | `Color::Indexed(1)`     |
| Truecolor (24-bit RGB)      |     ✓     | `\x1b[38;2;<R>;<G>;<B>m` | `Color::Rgb(255, 0, 0)` |

## Example

```rust
use ansi_to_tui::IntoText as _;
let buffer = std::fs::read("ascii/text.ascii")?;
let output = buffer.into_text()?;
```

Contributing and CI details are in [`CONTRIBUTING.md`][Contributing].

[Text]: https://docs.rs/ratatui-core/latest/ratatui_core/text/struct.Text.html
[Color]: https://docs.rs/ratatui-core/latest/ratatui_core/style/enum.Color.html
[Style]: https://docs.rs/ratatui-core/latest/ratatui_core/style/struct.Style.html
[Modifier]: https://docs.rs/ratatui-core/latest/ratatui_core/style/struct.Modifier.html
[Crate Badge]: https://img.shields.io/crates/v/ansi-to-tui?logo=rust
[Crate]: https://crates.io/crates/ansi-to-tui
[Repo Badge]: https://img.shields.io/badge/repo-ansi--to--tui-blue?logo=github
[Repo]: https://github.com/ratatui/ansi-to-tui
[Docs Badge]: https://img.shields.io/badge/docs-ansi--to--tui-blue?logo=rust
[Docs]: https://docs.rs/ansi-to-tui
[Contributing]: CONTRIBUTING.md
[License Badge]: https://img.shields.io/crates/l/ansi-to-tui
[License]: LICENSE
[CI Badge]: https://github.com/ratatui/ansi-to-tui/actions/workflows/build.yml/badge.svg
[CI]: https://github.com/ratatui/ansi-to-tui/actions/workflows/build.yml
[Codecov Badge]: https://codecov.io/gh/ratatui/ansi-to-tui/branch/main/graph/badge.svg
[Codecov]: https://codecov.io/gh/ratatui/ansi-to-tui
