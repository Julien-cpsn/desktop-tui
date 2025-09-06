use std::{env, fs};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shortcut {
    pub name: String,
    pub binary_path: String,
    pub args: Vec<String>,
    pub padding: Option<(i32, i32)>,
    pub position: Option<u32>
}

pub fn parse_shortcut_dir(shortcut_path: PathBuf) -> anyhow::Result<Vec<Shortcut>> {
    let mut desktop_entries = Vec::<Shortcut>::new();

    for entry in WalkDir::new(env::current_dir()?.join(shortcut_path)).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_dir() || entry_path.extension().is_none() || !entry_path.extension().unwrap().to_str().unwrap().ends_with("toml") {
            continue;
        }

        let file_content = fs::read_to_string(entry.path())?;
        let desktop_entry = toml::from_str::<Shortcut>(&file_content)?;

        let exists = desktop_entries.iter().find(|entry| entry.name == desktop_entry.name);

        if exists.is_some() {
            continue;
        }

        desktop_entries.push(desktop_entry);
    }

    desktop_entries.sort_by(|a, b| a.position.unwrap_or(99).cmp(&b.position.unwrap_or(99)));

    Ok(desktop_entries)
}