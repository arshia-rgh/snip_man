//! snipman CLI entry point.
//!
//! Commands:
//! - add: create a new snippet with description, tags, and code
//! - list: print all saved snippets
//! - interactive: open the interactive TUI to fuzzy-search and copy a snippet

mod init;
mod os;
mod shell;
mod snippets;
mod tui;

use crate::os::OsKind;
use crate::shell::ShellTarget;
use crate::snippets::{Snippet, load_snippets, save_snippet};
use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs, io};

/// Command-line interface for Snipman.
#[derive(Parser)]
#[command(author, version, about = "Fast TUI snippet manager", long_about = None)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands.
#[derive(Subcommand)]
enum Commands {
    /// Add a new snippet
    Add {
        /// A short, searchable description
        #[arg(short, long)]
        description: String,

        /// Comma-separated tags, e.g. "fs,io,read"
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Inline code (use quotes). For large/multi-line, prefer --file/--stdin/--editor
        #[arg(long)]
        code: Option<String>,

        /// Read the snippet body from a file path
        #[arg(long)]
        file: Option<PathBuf>,

        /// Read the snippet body from stdin (e.g., via pipe or here-doc)
        #[arg(long)]
        stdin: bool,

        /// Open editor to write the snippet body
        #[arg(long)]
        editor: bool,
    },
    /// List all snippets
    List,
    /// Remove the given snippet by its description
    Remove {
        /// The description of the snippet to remove
        #[arg(short, long)]
        description: String,
    },
    /// Enter the interactive TUI to search, copy and remove snippets
    Interactive,

    /// Install man page and shell completions into user directories and mark as installed
    Install {
        /// Which shell to target (auto detects current shell)
        #[arg(value_enum, default_value_t = ShellTarget::Auto)]
        shell: ShellTarget,
        /// Do not modify shell rc files (e.g., zsh fpath)
        #[arg(long)]
        no_modify_rc: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if requires_install_gate(&cli.command) && !init::state::is_installed() {
        eprintln!(
            "snipman is not initialized. Run: snipman install\n\
             After that, open a new shell to use completions. `man snipman` will also be available."
        );
        std::process::exit(2);
    }

    match cli.command {
        Commands::Add {
            description,
            tags,
            code,
            file,
            stdin,
            editor,
        } => {
            let code_body = match resolve_code_input(code, file, stdin, editor) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "Provide snippet code via --code, --file, --stdin or --edit. Error: {}",
                        e
                    );
                    return;
                }
            };

            let new_snippet = Snippet::new(description, tags, code_body);
            if let Err(e) = save_snippet(&new_snippet) {
                eprintln!("Error saving snippet: {}", e);
            }
        }
        Commands::List => match load_snippets() {
            Ok(snippets) => {
                if snippets.is_empty() {
                    println!("No snippets found.");
                } else {
                    println!("Found {} snippets:", snippets.len());
                    for snippet in snippets {
                        println!("- {} (Tags: {:?})", snippet.description, snippet.tags);
                    }
                }
            }
            Err(e) => eprintln!("Error loading snippets: {}", e),
        },
        Commands::Remove { description } => match load_snippets() {
            Ok(snippets) => {
                let snippet_opt = snippets.iter().find(|s| s.description == description);
                if let Some(snippet) = snippet_opt {
                    if let Err(e) = snippets::delete_snippet(&snippet.id) {
                        eprintln!("Error deleting snippet: {}", e);
                    } else {
                        println!("Snippet '{}' deleted successfully.", description);
                    }
                } else {
                    println!("No snippet found with description '{}'.", description);
                }
            }
            Err(e) => eprintln!("Error loading snippets: {}", e),
        },
        Commands::Interactive => {
            let all_snippets = match load_snippets() {
                Ok(snippets) => snippets,
                Err(e) => {
                    eprintln!("Failed to load snippets: {}", e);
                    return;
                }
            };

            match tui::run_tui(all_snippets) {
                Ok(Some(_)) => {
                    println!("✅ Snippet copied to clipboard!");
                }
                Ok(None) => {
                    println!("No snippet selected.");
                }
                Err(e) => {
                    eprintln!("TUI Error: {}", e);
                }
            }
        }
        Commands::Install {
            shell,
            no_modify_rc,
        } => {
            if let Err(e) = init::install_user_assets(shell, no_modify_rc) {
                eprintln!("Install failed: {}", e);
                std::process::exit(1);
            } else {
                println!("Install completed. Open a new shell. Try: man snipman");
            }
        }
    }
}

fn requires_install_gate(cmd: &Commands) -> bool {
    match cmd {
        Commands::Install { .. } => false,
        _ => true,
    }
}

/// Resolve the snippet code input from command-line options.
///
/// Precedence:
/// 1. --code: Inline code provided as a string.
/// 2. --file: File path given, read the file contents.
/// 3. --stdin: Read from stdin if true.
/// 4. --editor: Open an editor to compose the snippet body if true.
///
/// # Errors
/// Returns an error if no valid code source is provided, or if file/stdin/editor operations fail.
fn resolve_code_input(
    inline: Option<String>,
    file: Option<PathBuf>,
    from_stdin: bool,
    editor: bool,
) -> io::Result<String> {
    if let Some(s) = inline {
        return Ok(s);
    }
    if let Some(path) = file {
        return fs::read_to_string(path);
    }
    if from_stdin {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    if editor {
        return open_editor();
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "no code source provided",
    ))
}

/// Open a text editor to compose a snippet body and return its contents.
///
/// Editor resolution order:
/// - $VISUAL, then $EDITOR if set (parsed with a minimal shell-like splitter)
/// - Windows: notepad.exe
/// - macOS: `open -W -t`
/// - Other Unix: prefers `nano` if available, otherwise `vi`
///
/// Returns the edited text, or an error if the editor fails to launch or exits non-zero.
fn open_editor() -> io::Result<String> {
    let mut path = env::temp_dir();
    path.push(format!("snipman_{}.txt", std::process::id()));
    fs::write(&path, "")?;

    // Prefer $VISUAL, then $EDITOR
    let editor_spec = env::var("VISUAL").or_else(|_| env::var("EDITOR")).ok();
    let mut cmd;

    if let Some(spec) = editor_spec {
        let mut parts = parse_cmdline(&spec);
        if parts.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "empty $VISUAL/$EDITOR",
            ));
        }
        let prog = parts.remove(0);
        cmd = Command::new(prog);
        cmd.args(parts).arg(&path);
    } else if OsKind::current() == OsKind::Windows {
        cmd = Command::new("notepad.exe");
        cmd.arg(&path);
    } else if OsKind::current() == OsKind::Macos {
        cmd = Command::new("open");
        cmd.args(["-W", "-t"]).arg(&path);
    } else {
        let prefer_nano = Command::new("nano").arg("--version").status().is_ok();
        cmd = if prefer_nano {
            Command::new("nano")
        } else {
            Command::new("vi")
        };
        cmd.arg(&path);
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "editor exited with non-zero status",
        ));
    }

    let contents = fs::read_to_string(&path)?;
    let _ = fs::remove_file(&path);
    Ok(contents)
}

/// Minimal shell‑like splitter for `$VISUAL`/`$EDITOR`.
///
/// Splits a command string into argv without invoking a shell.
///
/// Behavior:
/// - Whitespace outside quotes separates arguments.
/// - Single quotes `'...'` take text literally; backslashes have no special meaning inside.
/// - Double quotes `"..."` group text; backslash `\` escapes the next character inside.
/// - Outside single quotes, a backslash `\` escapes the next character (including space and quotes).
/// - Quote characters are not included in results unless escaped inside double quotes.
/// - Unclosed quotes are tolerated: remaining text goes into the current token.
/// - A trailing standalone backslash is ignored.
///
/// Not a full shell parser:
/// - No variable expansion, globbing, pipelines, or command substitution.
///
/// # Examples:
/// ```rust,ignore
/// assert_eq!(parse_cmdline(r#"code -w"#), ["code", "-w"]);
/// assert_eq!(parse_cmdline(r#"my\ editor --flag"#), ["my editor", "--flag"]);
/// assert_eq!(parse_cmdline(r#"nvim "+set ft=rust""#), ["nvim", "+set ft=rust"]);
/// assert_eq!(
///     parse_cmdline(r#"sh -c "echo \"hi\" 'and bye'""#),
///     ["sh", "-c", r#"echo "hi" 'and bye'"#]
/// );
/// assert_eq!(
///     parse_cmdline(r#"--ext=\*.rs 'path with space'/file"#),
///     ["--ext=*.rs", "path with space/file"]
/// );
/// ```
fn parse_cmdline(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut buf = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;

    for ch in s.chars() {
        if escape {
            buf.push(ch);
            escape = false;
            continue;
        }
        match ch {
            '\\' if !in_single => {
                escape = true;
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !buf.is_empty() {
                    args.push(std::mem::take(&mut buf));
                }
            }
            _ => buf.push(ch),
        }
    }
    if !buf.is_empty() {
        args.push(buf);
    }
    args
}
