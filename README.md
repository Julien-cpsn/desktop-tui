# Desktop-TUI

A desktop environment without graphics (tmux-like).

Features:
- [x] Parse shortcut files containing apps
- [x] Display any application or command that uses stdout
- [x] Move and resize windows
- [x] Change tilling options
- [x] Handle application error
- [x] Select a file or a folder to then use it as an application or command argument
- [ ] Use the Crossterm backend (currently using ncurses but colors are wrong)
- [ ] Handle GNU applications and commands crash

![demo](./demo.gif)

## How to use

### Compile

```shell
cargo build
```

```shell
cargo build --release
```

### Run

```shell
cargo run <shortcut_folder_path>
```

Or in release :

```shell
cargo run --release <shortcut_folder_path>
```

## Shortcut file

Example `helix.toml` shortcut file:

```toml
# Window name
name = "Text editor"

# Command to execute
binary_path = "hx"

# Each command argument
# <file_path> will be replaced by a path selected in a dialog
args = ["<file_path>"]

# Pad inner window
padding = [0, 0]

# Shortcut position on the action bar
position = 3
```