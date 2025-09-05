//! Snippet data model and persistence utilities.
//!
//! Snippets are stored as prettified JSON files on disk in a per-user
//! application data directory:
//! - Linux:   $XDG_DATA_HOME (or ~/.local/share)/.snipman/snippets
//! - macOS:   ~/Library/Application Support/.snipman/snippets
//! - Windows: %APPDATA%/.snipman/snippets

use crate::os;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// A single code snippet, with description, tags, and code body.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snippet {
    /// Unique identifier (UUID v4) used as filename on disk.
    pub id: String,
    /// Short, searchable description.
    pub description: String,
    /// Optional tags used for filtering.
    pub tags: Vec<String>,
    /// The snippet body/code.
    pub code: String,
}

impl Snippet {
    /// Create a new snippet with a random UUID.
    pub fn new(description: String, tags: Vec<String>, code: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            tags,
            code,
        }
    }
}

fn get_snippets_dir() -> PathBuf {
    let path: PathBuf = match os::current_os() {
        os::OsKind::Windows => std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("USERPROFILE")
                    .map(|p| PathBuf::from(p).join("AppData").join("Roaming"))
            })
            .unwrap_or_else(|| PathBuf::from(".")),
        os::OsKind::Macos => std::env::var_os("HOME")
            .map(|home| {
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
            })
            .unwrap_or_else(|| PathBuf::from(".")),
        os::OsKind::Linux | os::OsKind::Unknown(_) => std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(|home| PathBuf::from(home).join(".local").join("share"))
            })
            .unwrap_or_else(|| PathBuf::from(".")),
    };

    let path = path.join(".snipman").join("snippets");
    path
}

/// Persist a snippet to disk as `<id>.json` in the snippets directory.
/// Creates the directory if it doesn't exist.
pub fn save_snippet(snippet: &Snippet) -> std::io::Result<()> {
    let snippets_dir = get_snippets_dir();

    fs::create_dir_all(&snippets_dir)?;

    let file_path = snippets_dir.join(format!("{}.json", snippet.id));
    let json_data = serde_json::to_string_pretty(snippet).expect("Failed to serialize snippet");

    fs::write(file_path, json_data)?;

    println!("Snippet '{}' saved successfully!", snippet.description);
    Ok(())
}

/// Load all snippets from disk, ignoring malformed entries with a warning.
pub fn load_snippets() -> std::io::Result<Vec<Snippet>> {
    let snippet_dir = get_snippets_dir();
    fs::create_dir_all(&snippet_dir)?;

    let mut snippets = Vec::new();
    let entries = fs::read_dir(snippet_dir)?.filter_map(std::io::Result::ok);

    for entry in entries {
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            let data = fs::read_to_string(&path)?;
            match serde_json::from_str(&data) {
                Ok(snippet) => snippets.push(snippet),
                Err(e) => eprintln!("Failed to parse {}: {}", path.display(), e),
            }
        }
    }

    Ok(snippets)
}
