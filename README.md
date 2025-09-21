Desktop-TUI 🖥️
===

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![GitHub Release](https://img.shields.io/github/v/release/julien-cpsn/desktop-tui?link=https%3A%2F%2Fgithub.com%2FJulien-cpsn%2Fdesktop-tuiC%2Freleases%2Flatest)
[![Crates.io](https://repology.org/badge/version-for-repo/crates_io/desktop-tui.svg)](https://crates.io/crates/desktop-tui)

A desktop environment without graphics (tmux-like).

Features:
- [x] Parse shortcut files containing apps
  - [x] Custom additional commands
  - [x] Custom window options
  - [x] Custom terminal options
- [x] Display any application or command that uses stdout
  - [x] Move and resize windows
  - [x] Handle and display application error
- [x] Change tilling options
- [x] Can let the user select a file or a folder to use its path as a command argument
- [x] Clock

![demo](./demo.gif)

## How to use

### Install

```shell
cargo install desktop-tui
```

### Compile

```shell
cargo build
```

```shell
cargo build --release
```

### Run

You can replace `cargo run --` with `desktop-tui`

```shell
cargo run -- <shortcut_folder_path>
```

Or in release :

```shell
cargo run --release -- <shortcut_folder_path>
```

## Shortcut file

Example `helix.toml` shortcut file:

```toml
# Window name
name = "Text editor"

# Command to execute
command = "hx"
# Each command argument
args = []

[taskbar]
# Shortcut position on the action bar
# Optional
position = 3

# Optional
[[taskbar.additional_commands]]
# Command name
name = "Open folder"
# Command to execute
command = "hx"
# <FILE_PATH> or <FOLDER_PATH> will be replaced by a path selected in a dialog
args = ["<FOLDER_PATH>"]

[[taskbar.additional_commands]]
name = "Open file"
command = "hx"
args = ["<FILE_PATH>"]

[window]
resizable = true
close_button = true
fixed_position = false
# Optional
size = { width = 10, height = 5 }

[terminal]
# Pad inner window
padding = [0, 0]
# Optional
background_color = { r = 30, g = 30, b = 30 }
```

## Star history

<a href="https://www.star-history.com/#julien-cpsn/desktop-tui&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=julien-cpsn/desktop-tui&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=julien-cpsn/desktop-tui&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=julien-cpsn/desktop-tui&type=Date" />
 </picture>
</a>

## License

The MIT license for this project can be seen [here](./LICENSE)
