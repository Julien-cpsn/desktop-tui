use crate::terminal_emulation::TerminalParser;
use anyhow::anyhow;
use appcui::dialogs::{Location, SelectFolderDialogFlags};
use appcui::graphics::{CharAttribute, CharFlags, Character, Color, Size, Surface};
use appcui::prelude::window::Flags;
use appcui::prelude::{canvas, Alignment, Canvas, EventProcessStatus, Handle, LayoutBuilder, OnResize, TimerEvents, Window};
use async_channel::{Receiver, Sender};
use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;
use virtual_terminal::{Command, Input, Output};

#[CustomControl(overwrite = OnKeyPressed)]
pub struct CustomKeyboardControl {
    pub should_exit: bool,
    pub tx: Sender<Input>,
    pub rx: Receiver<Output>,
}

#[Window(events = TimerEvents)]
pub struct TuiWindow {
    pub canvas: Handle<Canvas>,
    pub terminal_parser: TerminalParser,
    pub custom_keyboard_control: Handle<CustomKeyboardControl>,
    pub horizontal_adjustment: u32,
    pub vertical_adjustment: u32
}

impl TuiWindow {
    pub fn new<S, I>(app_name: &str, program: S, args: I, padding: Option<(i32, i32)>) -> anyhow::Result<Self> where S: AsRef<OsStr>, I: IntoIterator<Item = S> {
        let size = Size {
            width: 100,
            height: 25,
        };

        let mut x = 0;
        let mut y = 0;
        let mut horizontal_adjustment: i32 = 2;
        let mut vertical_adjustment: i32 = 2;

        if let Some(padding) = padding {
            x = padding.0;
            y = padding.1;

            horizontal_adjustment += padding.0;
            vertical_adjustment += padding.1;
        }

        let inner_size = Size {
            width: size.width.saturating_sub(horizontal_adjustment as u32),
            height: size.height.saturating_sub(vertical_adjustment as u32),
        };

        let win = Window::new(
            app_name,
            LayoutBuilder::new()
                .alignment(Alignment::Center)
                .width(size.width)
                .height(size.height)
                .build(),
            Flags::Sizeable
        );

        let modified_program = replace_file_path(program.as_ref().to_str().unwrap().to_string())?;
        let mut modified_args: Vec<String> = Vec::new();

        for arg in args {
            modified_args.push(replace_file_path(arg.as_ref().to_str().unwrap().to_string())?);
        }

        let cmd = Command::new(modified_program)
            .args(modified_args)
            .terminal_size((
                inner_size.width as usize,
                inner_size.height as usize
            ));

        let rx = cmd.out_rx();
        let tx = cmd.in_tx();

        let mut tui_win = Self {
            base: win,
            canvas: Handle::None,
            custom_keyboard_control: Handle::None,
            terminal_parser: TerminalParser::new(size.width, size.height),
            horizontal_adjustment: horizontal_adjustment  as u32,
            vertical_adjustment: vertical_adjustment as u32,
        };

        tui_win.canvas = tui_win.add(Canvas::new(
            Size::new(inner_size.width, inner_size.height),
            LayoutBuilder::new()
                .width(inner_size.width)
                .height(inner_size.height)
                .x(x)
                .y(y)
                .build(),
            canvas::Flags::None
        ));

        tui_win.clear_canva();

        let c = tui_win.canvas;
        if let Some(cv) = tui_win.control_mut(c) {
            let surface = cv.drawing_surface_mut();
            surface.write_string(0, 0, "Loading...", CharAttribute::default(), false);
        }

        let timer = match tui_win.timer() {
            Some(t) => t,
            None => return Err(anyhow!("Failed to get timer"))
        };
        timer.start(Duration::from_millis(25));

        tui_win.custom_keyboard_control = tui_win.add(CustomKeyboardControl {
            should_exit: false,
            base: ControlBase::new(Layout::fill(), true),
            tx,
            rx,
        });

        tokio::spawn(cmd.run());

        let c = tui_win.canvas;
        if let Some(cv) = tui_win.control_mut(c) {
            let surface = cv.drawing_surface_mut();
            surface.clear(Character::new(' ', Color::Transparent, Color::Transparent, CharFlags::None));
        }

        Ok(tui_win)
    }

    pub fn clear_canva(&mut self) {
        let c = self.canvas;
        if let Some(cv) = self.control_mut(c) {
            let surface = cv.drawing_surface_mut();
            surface.clear(Character::with_attributes(' ', CharAttribute::new(Color::White, Color::Black, CharFlags::None)));
        }
    }
}

impl TimerEvents for TuiWindow {
    fn on_update(&mut self, _: u64) -> EventProcessStatus {
        let (should_close, (rx_clone, tx_clone)) = {
            let ckc = self.control(self.custom_keyboard_control).unwrap();

            (ckc.should_exit, (ckc.rx.clone(), ckc.tx.clone()))
        };

        if should_close {
            self.close();
            return EventProcessStatus::Processed;
        }

        match rx_clone.try_recv() {
            Ok(msg) => match msg {
                Output::Pid(_) => EventProcessStatus::Ignored,
                Output::Stdout(command_output) => {
                    let size = self.size();
                    let inner_size = Size {
                        width: size.width.saturating_sub(self.horizontal_adjustment),
                        height: size.height.saturating_sub(self.vertical_adjustment),
                    };

                    let (old_surface, should_resize) = {
                        let c = self.canvas;
                        let cv = self.control_mut(c).unwrap();

                        let should_resize = cv.size() != inner_size;
                        let surface = cv.drawing_surface_mut();

                        let mut buffer = Vec::new();
                        surface.serialize_to_buffer(&mut buffer);

                        (Surface::from_buffer(&buffer).unwrap(), should_resize)
                    };

                    let new_surface = self.terminal_parser.parse_to_surface(&command_output, old_surface);

                    let c = self.canvas;
                    let cv = self.control_mut(c).unwrap();
                    let surface = cv.drawing_surface_mut();
                    *surface = new_surface;

                    if should_resize {
                        tx_clone
                            .send_blocking(Input::Resize((
                                inner_size.width as usize,
                                inner_size.height as usize
                            )))
                            .ok();
                        cv.set_size(inner_size.width as u16, inner_size.height as u16);
                        cv.resize_surface(inner_size);
                        self.clear_canva();
                    }

                    EventProcessStatus::Processed
                }
                Output::Error(error) => {
                    dialogs::error("An error occurred", &error);

                    self.close();
                    EventProcessStatus::Processed
                },
                Output::Terminated(_) => {
                    self.close();
                    EventProcessStatus::Processed
                }
            }
            Err(_) => EventProcessStatus::Ignored
        }
    }
}

fn replace_file_path(arg: String) -> anyhow::Result<String> {
    match arg.contains("<file_path>") {
        false => Ok(arg),
        true => match dialogs::select_folder("Select file", Location::Path(Path::new(env!("HOME"))), SelectFolderDialogFlags::Icons) {
            None => Err(anyhow!("No file selected")),
            Some(file_path) => Ok(arg.replace("<file_path>", file_path.to_str().unwrap()))
        }
    }
}