use appcui::prelude::{CharFlags, Character, Color, Surface};

#[derive(Debug, Clone, Copy)]
struct TerminalState {
    foreground: Color,
    background: Color,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    cursor_x: i32,
    cursor_y: i32,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            foreground: Color::White,
            background: Color::Black,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            cursor_x: 0,
            cursor_y: 0,
        }
    }
}

pub struct TerminalParser {
    width: u32,
    height: u32,
    state: TerminalState,
}

impl TerminalParser {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            state: TerminalState::default(),
        }
    }

    pub fn parse_to_surface(&mut self, data: &[u8], mut surface: Surface) -> Surface {
        let text = String::from_utf8_lossy(data);
        let chars: Vec<char> = text.chars().collect();

        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '\u{1b}' && i + 1 < chars.len() && chars[i + 1] == '[' {
                // Re-encode remaining chars into bytes for ANSI parsing
                let slice: String = chars[i..].iter().collect();
                let consumed = self.parse_ansi_sequence(slice.as_bytes(), &mut surface);
                // convert consumed bytes -> consumed chars
                let consumed_chars = String::from_utf8_lossy(&slice.as_bytes()[..consumed])
                    .chars()
                    .count();
                i += consumed_chars;
            }
            else {
                // Handle regular character
                self.write_character(chars[i], &mut surface);
                i += 1;
            }
        }
        
        surface
    }

    fn parse_ansi_sequence(&mut self, data: &[u8], surface: &mut Surface) -> usize {
        if data.len() < 3 {
            return 1; // Skip invalid sequence
        }

        let mut i = 2; // Skip '\x1b['
        let mut params = Vec::new();
        let mut current_param = String::new();
        let mut private_mode = false;

        // Handle private mode prefix '?'
        if i < data.len() && data[i] == b'?' {
            private_mode = true;
            i += 1;
        }

        // Parse parameters
        while i < data.len() {
            let byte = data[i];
            match byte {
                b'0'..=b'9' => current_param.push(byte as char),
                b';' => {
                    params.push(current_param.parse::<u32>().unwrap_or(0));
                    current_param.clear();
                }
                b'A'..=b'Z' | b'a'..=b'z' => {
                    // End of sequence
                    if !current_param.is_empty() {
                        params.push(current_param.parse::<u32>().unwrap_or(0));
                    }
                    if private_mode {
                        self.handle_private_ansi_command(byte as char, &params, surface);
                    }
                    else {
                        self.handle_ansi_command(byte as char, &params, surface);
                    }
                    return i + 1;
                }
                _ => break,
            }
            i += 1;
        }

        1 // Skip if we couldn't parse
    }

    fn handle_ansi_command(&mut self, command: char, params: &[u32], surface: &mut Surface) {
        match command {
            'H' | 'f' => {
                // Cursor position
                let row = params.get(0).unwrap_or(&1).saturating_sub(1) as i32;
                let col = params.get(1).unwrap_or(&1).saturating_sub(1) as i32;
                self.state.cursor_x = col.min(self.width as i32 - 1);
                self.state.cursor_y = row.min(self.height as i32 - 1);
            }
            'A' => {
                // Cursor up
                let count = params.get(0).unwrap_or(&1);
                self.state.cursor_y = (self.state.cursor_y - *count as i32).max(0);
            }
            'B' => {
                // Cursor down
                let count = params.get(0).unwrap_or(&1);
                self.state.cursor_y = (self.state.cursor_y + *count as i32).min(self.height as i32 - 1);
            }
            'C' => {
                // Cursor right
                let count = params.get(0).unwrap_or(&1);
                self.state.cursor_x = (self.state.cursor_x + *count as i32).min(self.width as i32 - 1);
            }
            'D' => {
                // Cursor left
                let count = params.get(0).unwrap_or(&1);
                self.state.cursor_x = (self.state.cursor_x - *count as i32).max(0);
            }
            'm' => {
                // SGR (Select Graphic Rendition) - colors and attributes
                if params.is_empty() {
                    // Reset all attributes
                    self.state = TerminalState::default();
                }
                else {
                    self.handle_sgr_params(params);
                }
            }
            'J' => {
                // Clear screen
                let mode = params.get(0).copied().unwrap_or(0);
                self.handle_erase_display(mode, surface);
            }
            'K' => {
                // Clear line
                let mode = params.get(0).copied().unwrap_or(0);
                self.handle_erase_line(mode, surface);
            }
            _ => {
                // Ignore unknown sequences
            }
        }
    }

    fn handle_private_ansi_command(&mut self, command: char, _params: &[u32], surface: &mut Surface) {
        match command {
            'l' => {
                // Hide cursor
                surface.hide_cursor()
            }
            'h' => {
                // Show cursor
                surface.set_cursor(self.state.cursor_x, self.state.cursor_y);
            }
            _ => {
                // ignore unknown private sequences
            }
        }
    }

    fn handle_erase_display(&mut self, param: u32, surface: &mut Surface) {
        match param {
            0 => {
                // clear from cursor to end of screen
                for y in self.state.cursor_y..self.height as i32 {
                    let start_x = if y == self.state.cursor_y { self.state.cursor_x } else { 0 };
                    for x in start_x..self.width as i32 {
                        surface.write_char(x, y, Character::new(' ', self.state.foreground, self.state.background, CharFlags::None));
                    }
                }
            }
            1 => {
                // clear from beginning of screen to cursor
                for y in 0..=self.state.cursor_y {
                    let end_x = if y == self.state.cursor_y { self.state.cursor_x } else { self.width as i32 - 1 };
                    for x in 0..=end_x {
                        surface.write_char(x, y, Character::new(' ', self.state.foreground, self.state.background, CharFlags::None));
                    }
                }
            }
            2 => {
                // clear entire screen
                surface.clear(Character::new(' ', self.state.foreground, self.state.background, CharFlags::None));
            }
            _ => {}
        }
    }

    fn handle_erase_line(&mut self, param: u32, surface: &mut Surface) {
        match param {
            0 => {
                // clear from cursor to end of line
                for x in self.state.cursor_x..self.width as i32 {
                    surface.write_char(x, self.state.cursor_y, Character::new(' ', self.state.foreground, self.state.background, CharFlags::None));
                }
            }
            1 => {
                // clear from beginning of line to cursor
                for x in 0..=self.state.cursor_x {
                    surface.write_char(x, self.state.cursor_y, Character::new(' ', self.state.foreground, self.state.background, CharFlags::None));
                }
            }
            2 => {
                // clear entire line
                for x in 0..self.width as i32 {
                    surface.write_char(x, self.state.cursor_y, Character::new(' ', self.state.foreground, self.state.background, CharFlags::None));
                }
            }
            _ => {}
        }
    }

    fn handle_sgr_params(&mut self, params: &[u32]) {
        let mut iter = params.iter().copied().peekable();

        while let Some(param) = iter.next() {
            match param {
                0 => self.state = TerminalState::default(), // Reset
                1 => self.state.bold = true,
                2 => self.state.dim = true,
                3 => self.state.italic = true,
                4 => self.state.underline = true,
                22 => {
                    self.state.bold = false;
                    self.state.dim = false;
                }
                23 => self.state.italic = false,
                24 => self.state.underline = false,

                // 16-color standard + bright
                30..=37 => self.state.foreground = ansi_16_color(param - 30, false),
                40..=47 => self.state.background = ansi_16_color(param - 40, false),
                90..=97 => self.state.foreground = ansi_16_color(param - 90, true),
                100..=107 => self.state.background = ansi_16_color(param - 100, true),

                // Extended color sequences
                38 | 48 => {
                    let is_foreground = param == 38;

                    if let Some(mode) = iter.next() {
                        match mode {
                            5 => {
                                // 256-color: 38;5;<idx> or 48;5;<idx>
                                if let Some(idx) = iter.next() {
                                    let color = ansi_256_color(idx);
                                    if is_foreground {
                                        self.state.foreground = color;
                                    } else {
                                        self.state.background = color;
                                    }
                                }
                            }
                            2 => {
                                // Truecolor: 38;2;<r>;<g>;<b> or 48;2;<r>;<g>;<b>
                                let (r, g, b) = (
                                    iter.next().unwrap_or(0),
                                    iter.next().unwrap_or(0),
                                    iter.next().unwrap_or(0),
                                );
                                let color = Color::RGB(r as u8, g as u8, b as u8);
                                if is_foreground {
                                    self.state.foreground = color;
                                } else {
                                    self.state.background = color;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                _ => {
                    // Ignore unknown
                }
            }
        }
    }

    fn write_character(&mut self, ch: char, surface: &mut Surface) {
        match ch {
            '\r' => {
                self.state.cursor_x = 0;
            }
            '\n' => {
                self.state.cursor_x = 0;
                self.state.cursor_y += 1;
                if self.state.cursor_y >= self.height as i32 {
                    self.state.cursor_y = self.height as i32 - 1;
                }
            }
            '\t' => {
                // Tab to next 8-character boundary
                self.state.cursor_x = ((self.state.cursor_x / 8) + 1) * 8;
                self.cursor_forward()
            }
            '\x08' => {
                // Backspace
                if self.state.cursor_x > 0 {
                    self.state.cursor_x -= 1;
                }
            }
            c if c.is_control() => {
                // Ignore other control characters
            }
            c => {
                // Regular printable character
                let mut flags = CharFlags::None;
                if self.state.bold {
                    flags |= CharFlags::Bold;
                }
                if self.state.dim {
                    //flags |= CharFlags::Dim;
                }
                if self.state.italic {
                    flags |= CharFlags::Italic;
                }
                if self.state.underline {
                    flags |= CharFlags::Underline;
                }

                let character = Character::new(c, self.state.foreground, self.state.background, flags);
                surface.write_char(self.state.cursor_x, self.state.cursor_y, character);
                self.cursor_forward();
            }
        }
    }

    pub fn cursor_forward(&mut self) {
        // Advance cursor
        self.state.cursor_x += 1;
        if self.state.cursor_x >= self.width as i32 {
            self.state.cursor_x = 0;
            self.state.cursor_y += 1;
            if self.state.cursor_y >= self.height as i32 {
                self.state.cursor_y = self.height as i32 - 1;
            }
        }
    }
}

/// Map 16 ANSI colors to RGB
fn ansi_16_color(code: u32, bright: bool) -> Color {
    let (r, g, b): (u8, u8, u8) = match code {
        0 => (0, 0, 0),       // Black
        1 => (128, 0, 0),     // Red
        2 => (0, 128, 0),     // Green
        3 => (128, 128, 0),   // Yellow
        4 => (0, 0, 128),     // Blue
        5 => (128, 0, 128),   // Magenta
        6 => (0, 128, 128),   // Cyan
        7 => (192, 192, 192), // White (light gray)
        _ => (255, 255, 255),
    };

    if bright {
        Color::RGB(
            r.saturating_mul(2).min(255),
            g.saturating_mul(2).min(255),
            b.saturating_mul(2).min(255)
        )
    }
    else {
        Color::RGB(r, g, b)
    }
}

/// Map 256-color palette to RGB
fn ansi_256_color(idx: u32) -> Color {
    match idx {
        0..=15 => {
            // Reuse 16 ANSI
            if idx < 8 {
                ansi_16_color(idx, false)
            } else {
                ansi_16_color(idx - 8, true)
            }
        }
        16..=231 => {
            // 6x6x6 color cube
            let n = idx - 16;
            let r = (n / 36) % 6;
            let g = (n / 6) % 6;
            let b = n % 6;
            Color::RGB(
                (r * 51) as u8,
                (g * 51) as u8,
                (b * 51) as u8,
            )
        }
        232..=255 => {
            // Grayscale ramp (24 shades)
            let level = 8 + (idx - 232) * 10;
            Color::RGB(level as u8, level as u8, level as u8)
        }
        _ => Color::RGB(0, 0, 0),
    }
}

