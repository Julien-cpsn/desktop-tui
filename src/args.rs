use clap::Parser;
use std::path::PathBuf;
use std::env;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(default_value_t = default_shortcut_dir())]
    pub shortcut_dir: PathBuf
}

fn default_shortcut_dir() -> PathBuf {
    let config_home = env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| String::from("."));
            format!("{}/.config", home)
        });
    PathBuf::from(config_home).join("desktop-tui/shortcuts")
}
