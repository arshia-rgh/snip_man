# SnipMan ✂️ - A Blazing Fast TUI Snippet Manager

SnipMan is a simple, fast, and intuitive command-line snippet manager built with Rust. It allows you to save, search, and manage your code snippets directly from your terminal, keeping you in the flow.

## Features

- Add Snippets: Quickly save new snippets with descriptions and tags.
- TUI Search: A fast, fuzzy-search interface to find snippets instantly.
- Copy to Clipboard: Select a snippet, press Enter, and the code is automatically in your clipboard.
- Cross-Platform: Works on Linux, macOS, and Windows.

## Installation

You can install SnipMan in two ways:

### 1. Pre-built Binary (Recommended)

Download the latest pre-compiled binary from GitHub Releases and place it in your executable path.

#### Linux & macOS

```bash
curl -sSL https://api.github.com/repos/arshia-rgh/snipman/releases/latest \
  | grep "browser_download_url.*-unknown-linux-gnu" \
  | cut -d '"' -f 4 \
  | xargs -I {} curl -sSL {} \
  | tar -xz -C /usr/local/bin snipman
```

*For macOS, replace `-unknown-linux-gnu` with `-apple-darwin` if a Mac build is available.*

#### Windows (PowerShell)

```powershell
Invoke-WebRequest -Uri "https://api.github.com/repos/arshia-rgh/snipman/releases/latest" |
  Select-Object -ExpandProperty Content |
  ConvertFrom-Json |
  Select-Object -ExpandProperty assets |
  Where-Object { $_.name -like "*-pc-windows-msvc.zip" } |
  Select-Object -ExpandProperty browser_download_url |
  ForEach-Object { Invoke-WebRequest -Uri $_ -OutFile "snipman.zip" }; `
  Expand-Archive -Path "snipman.zip" -DestinationPath "."; `
Move-Item -Path ".\snipman.exe" -Destination "C:\Windows\System32\snipman.exe"; `
Remove-Item "snipman.zip"
```

### 2. For Rust Developers

If you have the Rust toolchain installed, you can install directly from crates.io.

```bash
cargo install snipman
```

## Usage

Show help:

```bash
snipman --help
```

### Add

Add a new snippet (fields: description, tags, code).

```bash
snipman add --description "Open file" --tags fs,io --code 'std::fs::read_to_string("path")?;'
snipman add -d "Open file" -t fs,io -c 'std::fs::read_to_string("path")?;'
```

### List

Print all snippets.

```bash
snipman list
```

### Search (interactive)

Open the interactive picker with fuzzy search on descriptions.

```bash
snipman search
```

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
