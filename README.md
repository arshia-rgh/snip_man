# snip_man

A tiny Rust CLI to manage code snippets. Search interactively with fuzzy matching and copy the selected snippet to the clipboard.

## Features

- Fuzzy search over snippet descriptions
- Interactive TUI (type to filter, arrow keys to navigate)
- Auto copy selected snippet to clipboard on Enter
- Simple commands: `add`, `list`, `search

## Installation

- With Rust installed:
  - Build: `cargo build --release
  - Install to `$HOME/.cargo/bin`: `cargo install --path .

## Usage

Show help:

    snip_man --help

### Add

Add a new snippet (fields: description, tags, code).

    snip_man add --description "Open file" --tags fs,io --code 'std::fs::read_to_string("path")?;'
    snip_man add -d "Open file" -t fs,io -c 'std::fs::read_to_string("path")?;'

### List

Print all snippets.

    snip_man list

### Search (interactive)

Open the interactive picker with fuzzy search on descriptions.

    snip_man search

Key bindings:
- Type: refine fuzzy search
- Up/Down: move selection
- Enter: copy selected snippet code to clipboard and exit
- q: quit without copying
- Backspace: delete last character in query

## Data model

Each snippet has:
- description: short text label
- tags: comma-separated tags
- code: the snippet body

## Roadmap

- Configurable colors for the interactive UI
- More subcommands (edit, remove, import/export)
- Better file handling and storage robustness
- Richer search over tags and code body

## License

MIT. See `LICENSE`.
