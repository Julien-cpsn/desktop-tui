mod terminal_emulation;
mod tui_window;
mod keyboard;
mod desktop;
mod shortcut;
mod utils;
mod args;

use crate::desktop::MyDesktop;
use crate::shortcut::parse_shortcut_dir;
use appcui::backend::Type;
use appcui::prelude::{App, Theme};
use appcui::system::Themes;
use clap::Parser;
use crate::args::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let desktop_shortcuts = parse_shortcut_dir(args.shortcut_dir)?;

    let theme = Theme::new(Themes::Default);

    //theme.menu.text.normal = CharAttribute::new(Color::Blue, Color::Black, CharFlags::None);
    //theme.text.enphasized_2 = CharAttribute::new(Color::Red, Color::Green, CharFlags::None);
    //theme.desktop.character = Character::new(' ', Color::RGB(255, 255, 255), Color::RGB(85, 85, 85), CharFlags::None);

    // TODO: Fix Crossterm backend
    let app = App::with_backend(Type::NcursesTerminal)
        .desktop(MyDesktop::new(desktop_shortcuts))
        .menu_bar()
        .theme(theme)
        .color_schema(true)
        .build()?;

    app.run();

    Ok(())
}