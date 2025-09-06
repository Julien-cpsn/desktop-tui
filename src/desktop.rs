use crate::desktop::mydesktop::Commands;
use crate::shortcut::Shortcut;
use crate::tui_window::TuiWindow;
use crate::utils::time_to_string;
use appcui::prelude::menu::{Command, SingleChoice};
use appcui::prelude::*;
use std::collections::HashMap;

#[Desktop(
    events = [MenuEvents, DesktopEvents],
    overwrite = OnPaint,
    commands = [Exit, NoArrange, Cascade, Vertical, Horizontal, Grid, AppVisibilityToggle, OpenApp, CloseApp, None]
)]
pub struct MyDesktop {
    pub arrange_method: Option<desktop::ArrangeWindowsMethod>,
    pub desktop_menu: Handle<Menu>,
    pub arrange_menu: Handle<Menu>,
    pub sep: Handle<Menu>,
    pub app_menues: Vec<Handle<Menu>>,
    pub shortcuts: Vec<Shortcut>,
    pub app_windows: HashMap<usize, Handle<TuiWindow>>,
}

impl MyDesktop {
    pub fn new(shortcuts: Vec<Shortcut>) -> Self {
        Self {
            base: Desktop::new(),
            arrange_method: None,
            desktop_menu: Handle::None,
            sep: Handle::None,
            arrange_menu: Handle::None,
            app_menues: vec![Handle::None; shortcuts.len()],
            app_windows: HashMap::new(),
            shortcuts,
        }
    }
    
    pub fn create_window(&mut self, index: usize) -> anyhow::Result<()> {
        let app_name = self.shortcuts[index].name.clone();
        let command = self.shortcuts[index].binary_path.clone();
        let args = self.shortcuts[index].args.clone();
        let padding = self.shortcuts[index].padding.clone();

        let win = TuiWindow::new(
            &app_name,
            command,
            args,
            padding
        )?;

        let win_handle = self.add_window(win);
        self.app_windows.insert(index, win_handle);

        Ok(())
    }
}

impl OnPaint for MyDesktop {
    fn on_paint(&self, surface: &mut Surface, theme: &Theme) {
        surface.clear(theme.desktop.character);

        surface.write_string(
            surface.size().width as i32 - 5,
            1,
            &time_to_string(),
            CharAttribute::new(theme.menu.text.normal.foreground, theme.menu.text.normal.background, CharFlags::None),
            true
        );
        /*
        for (index, shortcut) in self.shortcuts.iter().enumerate() {
            let x = 1 + 29 * index as i32;

            surface.fill_rect(
                Rect::new(x, 2, x + 23, 2 + 12 + 1),
                Character::new(' ', Color::Transparent, Color::Gray, CharFlags::None)
            );

            surface.write_string(
                x,
                2,
                shortcut.logo.as_str(),
                CharAttribute::new(Color::White, Color::Gray, CharFlags::None),
                true
            );

            let mut text_format = TextFormat::default();

            text_format.set_position(x + 25, 2 + 12 + 1);
            text_format.set_chars_count(30);
            text_format.set_align(TextAlignment::Center);
            text_format.set_attribute(CharAttribute::new(Color::White, Color::Gray, CharFlags::None));
            //text_format.set_wrap_type(WrapType::SingleLineWrap(24));

            surface.write_text(
                shortcut.name.as_str(),
                &text_format
            );
        }*/
    }
}

impl DesktopEvents for MyDesktop {
    fn on_start(&mut self) {
        let mut menu = Menu::new("Desktop");

        menu.add(Command::new("Exit", Key::None, Commands::Exit));

        self.desktop_menu = self.register_menu(menu);

        let mut menu = Menu::new("Tilling");

        menu.add(SingleChoice::new("No arrangement", Key::None, Commands::NoArrange, true));
        menu.add(SingleChoice::new("Cascade", Key::None, Commands::Cascade, false));
        menu.add(SingleChoice::new("Vertical", Key::None, Commands::Vertical, false));
        menu.add(SingleChoice::new("Horizontal", Key::None, Commands::Horizontal, false));
        menu.add(SingleChoice::new("Grid", Key::None, Commands::Grid, false));

        self.arrange_menu = self.register_menu(menu);

        let mut menu = Menu::new("|");

        menu.add(Command::new("", Key::None, Commands::None));

        self.sep = self.register_menu(menu);

        let shortcuts = self.shortcuts.clone();
        for (index, shortcut) in shortcuts.iter().enumerate() {
            let mut menu = Menu::new(&shortcut.name);

            menu.add(Command::new("Hide", Key::None, Commands::AppVisibilityToggle));
            menu.add(Command::new("Start", Key::None, Commands::OpenApp));
            menu.add(Command::new("Close", Key::None, Commands::CloseApp));

            self.app_menues[index] = self.register_menu(menu);
        }
    }

    fn on_update_window_count(&mut self, _count: usize) {
        let m = self.arrange_method;

        if let Some(method) = m {
            self.arrange_windows(method);
        }
    }

}

impl MenuEvents for MyDesktop {
    fn on_command(&mut self, menu: Handle<Menu>, item: Handle<Command>, command: Commands) {
        match command {
            Commands::Exit => self.close(),
            Commands::OpenApp | Commands::CloseApp | Commands::AppVisibilityToggle => {
                let mut app = None;

                for (index, app_menu) in self.app_menues.iter().enumerate() {
                    if &menu == app_menu {
                        app = Some(index);
                    }
                }

                if let Some(index) = app {
                    let win_handle = match self.app_windows.get(&index) {
                        Some(win_handle) => Some(*win_handle),
                        None => {
                            match command {
                                Commands::OpenApp => {
                                    self.create_window(index).ok();
                                },
                                _ => {}
                            }

                            None
                        }
                    };

                    let mut visibility_item = None;

                    if let Some(win_handle) = win_handle {
                        if let Some(window) = self.window_mut(win_handle) {
                            match command {
                                Commands::AppVisibilityToggle => match window.is_visible() {
                                    true => {
                                        window.set_visible(false);
                                        visibility_item = Some("Show");
                                    },
                                    false => {
                                        window.set_visible(true);
                                        visibility_item = Some("Hide");
                                    }
                                }
                                Commands::OpenApp => {}
                                Commands::CloseApp => {
                                    window.close();
                                    self.app_windows.remove(&index);
                                }
                                _ => {}
                            }
                        }
                        else {
                            match command {
                                Commands::OpenApp => {
                                    self.create_window(index).ok();
                                },
                                Commands::CloseApp => {},
                                _ => {}
                            }
                        }
                    }

                    if let Some(name) = visibility_item {
                        let item = self.menuitem_mut(menu, item).unwrap();
                        item.set_caption(name);
                    }
                }
            }
            _ => {}
        }
    }

    fn on_select(&mut self, _menu: Handle<Menu>, _item: Handle<SingleChoice>, command: Commands) {
        match command {
            Commands::Exit => self.close(),
            Commands::NoArrange => self.arrange_method = None,
            Commands::Cascade => self.arrange_method = Some(desktop::ArrangeWindowsMethod::Cascade),
            Commands::Vertical => self.arrange_method = Some(desktop::ArrangeWindowsMethod::Vertical),
            Commands::Horizontal => self.arrange_method = Some(desktop::ArrangeWindowsMethod::Horizontal),
            Commands::Grid => self.arrange_method = Some(desktop::ArrangeWindowsMethod::Grid),
            _ => {}
        }
        let m = self.arrange_method;

        if let Some(method) = m {
            self.arrange_windows(method);
        }
    }

    fn on_update_menubar(&self, menubar: &mut MenuBar) {
        menubar.add(self.desktop_menu, 0);
        menubar.add(self.arrange_menu, 1);
        menubar.add(self.sep, 2);

        for (index, app_menu) in self.app_menues.iter().enumerate() {
            menubar.add(*app_menu, index as u8 + 3);
        }
    }
}