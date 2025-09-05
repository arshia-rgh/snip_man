use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snippet {
    pub id: String,
    pub description: String,
    pub tags: Vec<String>,
    pub code: String,
}

impl Snippet {
    pub fn new(description: String, tags: Vec<String>, code: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            tags,
            code,
        }
    }
}

pub fn get_snippets_dir() -> PathBuf {
    let mut path = std::env::current_dir().expect("Failed to get current directory");
    path.push("snippets");
    path
}

pub fn save_snippet(snippet: &Snippet) -> std::io::Result<()> {
    let snippets_dir = get_snippets_dir();

    fs::create_dir_all(&snippets_dir)?;

    let file_path = snippets_dir.join(format!("{}.json", snippet.id));
    let json_data = serde_json::to_string_pretty(snippet).expect("Failed to serialize snippet");

    fs::write(file_path, json_data)?;

    println!("Snippet '{}' saved successfully!", snippet.description);
    Ok(())
}

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
