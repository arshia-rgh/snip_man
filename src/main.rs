mod snippets;
mod tui;

use crate::snippets::{Snippet, load_snippets, save_snippet};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new snippet
    Add {
        #[arg(short, long)]
        description: String,

        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        #[arg(short, long)]
        code: String,
    },
    /// List all snippets
    List,
    /// search snippets with a TUI
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
