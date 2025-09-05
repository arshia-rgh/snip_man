//! snipman CLI entry point.
//!
//! Commands:
//! - add: create a new snippet with description, tags, and code
//! - list: print all saved snippets
//! - search: open the interactive TUI to fuzzy-search and copy a snippet

mod os;
mod snippets;
mod tui;

use crate::snippets::{Snippet, load_snippets, save_snippet};
use clap::{Parser, Subcommand};

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

        /// The snippet body/code
        #[arg(short, long)]
        code: String,
    },
    /// List all snippets
    List,
    /// Remove the given snippet by its description
    Remove {
        /// The description of the snippet to remove
        #[arg(short, long)]
        description: String,
    },
    /// Search snippets with an interactive TUI; copies selection to clipboard
    Search,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add {
            description,
            tags,
            code,
        } => {
            let new_snippet = Snippet::new(description.clone(), tags.clone(), code.clone());
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
        Commands::Remove { description } => {
            match load_snippets() {
                Ok(snippets) => {
                    let snippet_opt = snippets.iter().find(|s| s.description == *description);
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
            }
        }
        Commands::Search => {
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
    }
}
