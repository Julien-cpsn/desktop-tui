use std::{env, fs};
use std::path::PathBuf;
use nestify::nest;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

nest! {
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Shortcut {
        pub name: String,

        pub command: String,

        #[serde(default)]
        pub args: Vec<String>,

        pub taskbar:
            #[derive(Clone, Debug, Serialize, Deserialize)]
            pub struct TaskbarOptions {
                pub position: Option<u32>,
                #[serde(default)]
                pub additional_commands: Vec<
                    #[derive(Clone, Debug, Serialize, Deserialize)]
                    pub struct TaskbarCommand {
                        pub name: String,
                        pub command: String,
                        pub args: Vec<String>,
                    }
                >
            },

        pub window:
            #[derive(Clone, Debug, Serialize, Deserialize)]
            pub struct WindowOptions {
                pub resizable: bool,
                pub close_button: bool,
                pub fixed_position: bool,
                pub size: Option<
                    #[derive(Clone, Debug, Serialize, Deserialize)]
                    pub struct WindowSize {
                        pub width: u32,
                        pub height: u32,
                    }
                >
            },

        pub terminal:
            #[derive(Clone, Debug, Serialize, Deserialize)]
            pub struct TerminalOptions {
                pub padding: Option<(i32, i32)>
            }
    }
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

    desktop_entries
        .sort_by(
            |a, b|
                a.taskbar.position
                .unwrap_or(99)
                .cmp(&b.taskbar.position.unwrap_or(99))
        );

    Ok(desktop_entries)
}