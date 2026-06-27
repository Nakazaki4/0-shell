# 0-shell

A minimalist Unix shell written from scratch in Rust. It reads commands from an interactive prompt, parses them, and runs a set of built-in commands directly — without relying on an existing shell like `bash` or `sh` to do the work.

The goal of the project is to understand what really happens between pressing a key and seeing a command run: raw terminal input, tokenizing, parsing into a syntax tree, and executing.

## Features

- **Interactive prompt** showing the current working directory.
- **Raw-mode line editor** built on `termios`, with:
  - Command **history** navigation using the ↑ / ↓ arrow keys.
  - **Backspace / Delete** editing.
  - **Ctrl+D** to exit on an empty line.
  - **Multi-line input**: a trailing `\` or an unclosed double quote drops you into a continuation prompt (`>` and `dquote>`).
- **Command sequencing** with `;` (e.g. `mkdir test ; cd test ; pwd`).
- **Quoting and escaping**: double quotes group arguments, and `\` escapes the next character.
- A pipeline of **lexer → parser → executor**, with parse errors reported without crashing the shell.

> Note: TAB auto-completion is stubbed out and not yet implemented.

## Built-in commands

| Command | Description | Supported flags |
|---------|-------------|-----------------|
| `cd`    | Change the current directory | — |
| `pwd`   | Print the working directory | — |
| `ls`    | List directory contents | `-a` (show hidden), `-l` (long format), `-F` (classify entries) |
| `cat`   | Concatenate and print files | — |
| `cp`    | Copy files | — |
| `mv`    | Move / rename files | — |
| `rm`    | Remove files and directories | `-r` (recursive) |
| `mkdir` | Create directories | — |
| `echo`  | Print arguments | — |
| `exit`  | Exit the shell | — |

Unknown commands return a `command not found` message instead of terminating the shell.

## Getting started

### Prerequisites

- A recent [Rust toolchain](https://www.rust-lang.org/tools/install) (the project uses Rust **edition 2024**).
- A Unix-like environment (Linux / macOS), since the shell relies on terminal and POSIX APIs.

### Build

```bash
git clone https://github.com/Nakazaki4/0-shell.git
cd 0-shell
cargo build --release
```

### Run

```bash
cargo run
```

Or run the compiled binary directly:

```bash
./target/release/O-shell
```

## Usage

```text
/home/user $ pwd
/home/user
/home/user $ mkdir demo ; cd demo ; pwd
/home/user/demo
/home/user/demo $ echo "hello world"
hello world
/home/user/demo $ ls -la
...
/home/user/demo $ exit
```

Use the arrow keys to scroll through previous commands, and press **Ctrl+D** on an empty line (or type `exit`) to quit.

## Project structure

```
src/
├── main.rs          # Entry point: raw-mode setup, prompt, line editor, REPL loop
├── lexer.rs         # tokenize(): splits input into tokens, handles quotes/escapes/`;`
├── parser.rs        # parse(): builds the AST (SimpleCommand, Sequence)
├── executor.rs      # execute(): walks the AST and dispatches to built-ins
├── error.rs         # Error types
└── builtins/        # One module per built-in command
    ├── mod.rs
    ├── cat.rs
    ├── cd.rs
    ├── cp.rs
    ├── echo.rs
    ├── exit.rs
    ├── ls.rs
    ├── mkdir.rs
    ├── mv.rs
    ├── pwd.rs
    └── rm.rs
```

### How it works

1. **Input** is read byte-by-byte in raw mode so the shell controls editing, history, and multi-line continuation.
2. **`tokenize`** turns the raw command string into a list of tokens, respecting quotes, escapes, and `;` separators.
3. **`parse`** builds an abstract syntax tree — a single `SimpleCommand`, or a `Sequence` of commands joined by `;`.
4. **`execute`** walks the tree and dispatches each command to its built-in implementation. Raw mode is toggled off around execution so commands behave normally, then restored afterwards.

## Dependencies

- [`nix`](https://crates.io/crates/nix) — safe bindings to POSIX/Unix APIs (process, fs, term)
- [`libc`](https://crates.io/crates/libc) — low-level system bindings
- [`termios`](https://crates.io/crates/termios) — terminal mode control (raw mode)
- [`chrono`](https://crates.io/crates/chrono) — timestamps for `ls -l`
- [`users`](https://crates.io/crates/users) — user/group lookup for `ls -l`

## Roadmap

- TAB auto-completion
- Additional flags for the existing built-ins
- More built-in commands

## License

No license file is currently provided. Add one (e.g. MIT) if you intend others to reuse the code.
