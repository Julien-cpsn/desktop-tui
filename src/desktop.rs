use crate::desktop::mydesktop::Commands;
use crate::shortcut::Shortcut;
use crate::tui_window::TuiWindow;
use crate::utils::time_to_string;
use appcui::prelude::appbar::MenuButton;
use appcui::prelude::menu::{Command, SingleChoice};
use appcui::prelude::*;
use appcui::ui::appbar::Side;
use std::collections::HashMap;
use std::time::Duration;

#[Desktop(
    events = [AppBarEvents, MenuEvents, DesktopEvents, TimerEvents],
    overwrite = OnPaint,
    commands = [Exit, NoArrange, Cascade, Vertical, Horizontal, Grid, AppVisibilityToggle, OpenApp, CloseApp, AppCommand, None]
)]
pub struct MyDesktop {
    pub arrange_method: Option<desktop::ArrangeWindowsMethod>,
    pub desktop_menu: Handle<MenuButton>,
    pub arrange_menu: Handle<MenuButton>,
    pub separator: Handle<appbar::Separator>,
    pub app_menues: Vec<Handle<Menu>>,
    pub app_menu_buttons: Vec<Handle<MenuButton>>,
    pub shortcuts: Vec<Shortcut>,
    pub app_windows: HashMap<usize, Handle<TuiWindow>>,
    pub time_label: Handle<appbar::Label>,
}

impl MyDesktop {
    pub fn new(shortcuts: Vec<Shortcut>) -> Self {
        Self {
            base: Desktop::new(),
            arrange_method: None,
            desktop_menu: Handle::None,
            separator: Handle::None,
            arrange_menu: Handle::None,
            app_menues: vec![Handle::None; shortcuts.len()],
            app_menu_buttons: vec![Handle::None; shortcuts.len()],
            app_windows: HashMap::new(),
            time_label: Handle::None,
            shortcuts,
        }
    }
    
    pub fn create_window(&mut self, index: usize, command: String, args: Vec<String>) -> anyhow::Result<()> {
        let app_name = self.shortcuts[index].name.clone();
        let window = self.shortcuts[index].window.clone();
        let terminal = self.shortcuts[index].terminal.clone();

        let window = TuiWindow::new(
            &app_name,
            command,
            args,
            window,
            terminal,
        )?;

        let win_handle = self.add_window(window);
        self.app_windows.insert(index, win_handle);

        Ok(())
    }
}

impl OnPaint for MyDesktop {
    fn on_paint(&self, surface: &mut Surface, theme: &Theme) {
        surface.clear(theme.desktop.character);
    }
}

impl DesktopEvents for MyDesktop {
    fn on_start(&mut self) {
        let shortcuts = self.shortcuts.clone();
        let mut desktop_menu = Menu::new();

        desktop_menu.add(Command::new("Exit", Key::None, Commands::Exit));

        let desktop_menu_button = self.appbar().add(MenuButton::new("Desktop", desktop_menu, 0, Side::Left));

        let mut tilling_menu = Menu::new();

        tilling_menu.add(SingleChoice::new("No arrangement", Key::None, Commands::NoArrange, true));
        tilling_menu.add(SingleChoice::new("Cascade", Key::None, Commands::Cascade, false));
        tilling_menu.add(SingleChoice::new("Vertical", Key::None, Commands::Vertical, false));
        tilling_menu.add(SingleChoice::new("Horizontal", Key::None, Commands::Horizontal, false));
        tilling_menu.add(SingleChoice::new("Grid", Key::None, Commands::Grid, false));

        let arrange_menu_button = self.appbar().add(MenuButton::new("Tilling", tilling_menu, 1, Side::Left));

        let separator = self.appbar().add(appbar::Separator::new(2, Side::Left));

        let mut app_menues = vec![Handle::<Menu>::None; shortcuts.len()];
        let mut app_menu_buttons = vec![Handle::<MenuButton>::None; shortcuts.len()];
        for (index, shortcut) in shortcuts.iter().enumerate() {
            let mut menu = Menu::new();

            menu.add(Command::new("Hide", Key::None, Commands::AppVisibilityToggle));
            menu.add(Command::new("Start", Key::None, Commands::OpenApp));
            menu.add(Command::new("Close", Key::None, Commands::CloseApp));

            if !shortcut.taskbar.additional_commands.is_empty() {
                menu.add(menu::Separator::new());
            }

            for command in &shortcut.taskbar.additional_commands {
                menu.add(Command::new(&command.name, Key::None, Commands::AppCommand));
            }

            app_menues[index] = self.register_menu(menu);
            app_menu_buttons[index] = self.appbar().add(MenuButton::with_handle(&shortcut.name, app_menues[index], 2 + index as u8, Side::Left));
        }

        self.time_label = self.appbar().add(appbar::Label::new(&time_to_string(), 0, Side::Right));

        self.desktop_menu = desktop_menu_button;
        self.arrange_menu = arrange_menu_button;
        self.separator = separator;
        self.app_menues = app_menues;
        self.app_menu_buttons = app_menu_buttons;

        let timer = self.timer().expect("Failed to get timer");
        timer.start(Duration::from_millis(2000));
    }

    fn on_update_window_count(&mut self, _count: usize) {
        let m = self.arrange_method;

        if let Some(method) = m {
            self.arrange_windows(method);
        }
    }
}

impl AppBarEvents for MyDesktop {
    fn on_update(&self, app_bar: &mut AppBar) {
        app_bar.show(self.desktop_menu);
        app_bar.show(self.arrange_menu);
        app_bar.show(self.separator);

        for app_menu in self.app_menu_buttons.iter() {
            app_bar.show(*app_menu);
        }

        app_bar.show(self.time_label);
    }
}

impl MenuEvents for MyDesktop {
    fn on_command(&mut self, menu: Handle<Menu>, item: Handle<Command>, command: Commands) {
        match command {
            Commands::Exit => {
                for window in self.app_windows.clone().values() {
                    if let Some(win) = self.window_mut(*window) {
                        win.close_command();
                    }
                }

                self.close()
            },
            Commands::OpenApp | Commands::CloseApp | Commands::AppVisibilityToggle | Commands::AppCommand => {
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
                                    let command = self.shortcuts[index].command.clone();
                                    let args = self.shortcuts[index].args.clone();
                                    self.create_window(index, command, args).ok();
                                },
                                Commands::AppCommand => {
                                    let shortcut = self.shortcuts[index].clone();
                                    let item = self.menuitem_mut(menu, item).unwrap();

                                    for command in shortcut.taskbar.additional_commands {
                                        if item.caption() == command.name {
                                            self.create_window(index, command.command, command.args).ok();
                                            break;
                                        }
                                    }
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
                                    window.close_command();
                                    self.app_windows.remove(&index);
                                }
                                _ => {}
                            }
                        }
                        else {
                            match command {
                                Commands::OpenApp => {
                                    let command = self.shortcuts[index].command.clone();
                                    let args = self.shortcuts[index].args.clone();
                                    self.create_window(index, command, args).ok();
                                },
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
}

impl TimerEvents for MyDesktop {
    fn on_update(&mut self, _: u64) -> EventProcessStatus {
        let time_label_handle = self.time_label;
        let time_label = self.appbar().get_mut(time_label_handle).unwrap();

        time_label.set_caption(&time_to_string());

        EventProcessStatus::Processed
    }
}