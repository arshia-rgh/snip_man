# SnipMan ✂️ - A Blazing Fast TUI Snippet Manager

SnipMan is a simple, fast, and intuitive command-line snippet manager built with Rust. It lets you save, search,
and manage code snippets right from your terminal.

## Features

- Add Snippets: Quickly save new snippets with descriptions and tags.
- Interactive TUI: Fuzzy-search, preview, copy, and delete.
- Copy to Clipboard: Select a snippet, press Enter, and the code is automatically in your clipboard.
- Cross-Platform: Linux, macOS, and Windows.

## Installation

Pick one of the following:

- Using Cargo (Rust toolchain):

  ```bash
  cargo install snipman
  ```

- Prebuilt binaries (GitHub Releases):
    - Linux (x86_64):
      ```bash
      curl -LO https://github.com/arshia-rgh/snipman/releases/latest/download/snipman-x86_64-unknown-linux-gnu.tar.gz
      tar xzf snipman-x86_64-unknown-linux-gnu.tar.gz
      sudo mv snipman-x86_64-unknown-linux-gnu /usr/local/bin/snipman
      ```
    - macOS (x86_64):
      ```bash
      curl -LO https://github.com/arshia-rgh/snipman/releases/latest/download/snipman-x86_64-apple-darwin.tar.gz
      tar xzf snipman-x86_64-apple-darwin.tar.gz
      sudo mv snipman-x86_64-apple-darwin /usr/local/bin/snipman
      ```
    - Windows (x86_64):
        1) Download: https://github.com/arshia-rgh/snipman/releases/latest/download/snipman-x86_64-pc-windows-msvc.zip
        2) Extract, rename the file to `snipman.exe`, and place it in a folder on your PATH (e.g., `%USERPROFILE%\bin`).

## Initialize (one-time)

Before using other commands, run the installer to set up manual pages and shell completions. Other commands are gated until this completes.

```bash
snipman install
```

Notes:
- After install, open a new shell so completions and man pages are picked up.
- Try: `man snipman`
- The installer writes a per-user stamp to mark completion; without it, running other commands prints a helpful message to run `snipman install`.

Options:
- Choose target shell(s):
  ```bash
  snipman install --shell auto     # default; detect $SHELL, else install for bash,zsh,fish
  snipman install --shell bash
  snipman install --shell zsh
  snipman install --shell fish
  snipman install --shell all
  ```
- Avoid modifying your shell rc (Zsh):
  ```bash
  snipman install --no-modify-rc
  ```
  By default on Zsh, SnipMan appends a small idempotent block to your ~/.zshrc (or $ZDOTDIR/.zshrc) to add the completion path and run `compinit`.

Where things go (Unix):
- Man page: `~/.local/share/man/man1/snipman.1` (then `mandb -q` is attempted quietly)
- Bash completion: `~/.local/share/bash-completion/completions/snipman`
- Zsh completion: `~/.local/share/zsh/site-functions/_snipman` (name decided by clap_complete)
- Fish completion: `~/.config/fish/completions/snipman.fish`

## Usage

Note: ensure you ran `snipman install` first (see above).

Show help:

```bash
snipman --help
```

### Add

Create a new snippet. Provide the code body via one of: `--code`, `--file`, `--stdin`, or `--editor`.

Precedence (if multiple are provided): `--code` > `--file` > `--stdin` > `--editor`.

- Inline code:
  ```bash
  snipman add -d "Open file" -t fs,io --code 'std::fs::read_to_string("path")?;'
  ```
- From a file:
  ```bash
  snipman add -d "HTTP GET" -t http,req --file examples/get.rs
  ```
- From stdin (pipe):
  ```bash
  cat snippet.rs | snipman add -d "Read file" -t fs --stdin
  ```
- Open your editor ($VISUAL or $EDITOR; flags supported, e.g., `export VISUAL="code -w"`; falls back to nano/vi on Unix,
  Notepad on Windows):
  ```bash
  snipman add -d "Regex replace" -t text,regex --editor
  ```

Flags:

- -d, --description <TEXT>  required
- -t, --tags <LIST>         comma-separated (e.g., fs,io,read)
- --code <TEXT>             inline code body
- --file <PATH>             read code from file
- --stdin read code from stdin
- --editor open $VISUAL/$EDITOR to compose

### List

Print all snippets.

```bash
snipman list
```

### Remove

Remove a snippet by its description (as shown in `list`).

```bash
snipman remove --description "Open file"
# or
snipman remove -d "Open file"
```

### Interactive

Open the interactive picker with fuzzy search, preview, copy, and delete.

```bash
snipman interactive
```

Key bindings:

- Type: refine fuzzy search
- Up/Down: move selection
- Enter: copy selected snippet code to clipboard and exit
- q: quit
- p: toggle compact/full preview
- d: delete selected snippet (confirm with y/n)
- PgUp/PgDn: scroll preview up/down
- Backspace: delete last character in query

## Roadmap

- Configurable colors for the interactive UI
- Richer search over tags and code body

## License

MIT. See `LICENSE`.
