use crate::tui_window::CustomKeyboardControl;
use appcui::input::{Key, KeyModifier};
use appcui::prelude::{EventProcessStatus, KeyCode, OnKeyPressed};
use virtual_terminal::Input;

impl OnKeyPressed for CustomKeyboardControl {
    fn on_key_pressed(&mut self, key: Key, character: char) -> EventProcessStatus {
        if !self.has_focus() {
            return EventProcessStatus::Ignored;
        }

        if key.modifier == KeyModifier::Ctrl && key.code == KeyCode::C {
            self.tx.send_blocking(Input::Terminate).ok();
            self.should_exit = true;
        }
        else {
            if let Some(data) = to_escape_sequence_vec(key, character) {
                self.tx
                    .send_blocking(Input::Data(data))
                    .ok();
            }
        }

        EventProcessStatus::Processed
    }
}

pub fn to_escape_sequence_vec(key: Key, character: char) -> Option<Vec<u8>> {
    use KeyModifier as KM;

    let mut seq = Vec::new();

    // Calculate modifier parameter for CSI sequences
    // (1 = no modifier, 2=Shift, 3=Alt, 4=Alt+Shift, 5=Ctrl, ...)
    let mut mod_param = 1;
    if key.modifier.contains(KM::Shift) {
        mod_param += 1;
    }
    if key.modifier.contains(KM::Alt) {
        mod_param += 2;
    }
    if key.modifier.contains(KM::Ctrl) {
        mod_param += 4;
    }

    match key.code {
        // ----- Single-byte ASCII -----
        KeyCode::Space => seq.push(b' '),
        KeyCode::Enter => seq.push(b'\r'),
        KeyCode::Escape => seq.push(0x1B),
        KeyCode::Tab => return match key.modifier.contains(KM::Shift) {
            true => Some(b"\x1B[Z".to_vec()),
            false => Some(vec![b'\t'])
        },
        KeyCode::Backspace => seq.push(0x7F),

        // ----- Arrows -----
        KeyCode::Up => return Some(csi_mod(b"A", mod_param)),
        KeyCode::Down => return Some(csi_mod(b"B", mod_param)),
        KeyCode::Right => return Some(csi_mod(b"C", mod_param)),
        KeyCode::Left => return Some(csi_mod(b"D", mod_param)),

        // ----- Navigation -----
        KeyCode::Home => return Some(csi_mod(b"H", mod_param)),
        KeyCode::End => return Some(csi_mod(b"F", mod_param)),
        KeyCode::PageUp => return Some(csi_mod_tilde(5, mod_param)),
        KeyCode::PageDown => return Some(csi_mod_tilde(6, mod_param)),
        KeyCode::Insert => return Some(csi_mod_tilde(2, mod_param)),
        KeyCode::Delete => return Some(csi_mod_tilde(3, mod_param)),

        // ----- Function keys -----
        KeyCode::F1 => return Some(ss3_or_csi(b"P", 11, mod_param)),
        KeyCode::F2 => return Some(ss3_or_csi(b"Q", 12, mod_param)),
        KeyCode::F3 => return Some(ss3_or_csi(b"R", 13, mod_param)),
        KeyCode::F4 => return Some(ss3_or_csi(b"S", 14, mod_param)),
        KeyCode::F5 => return Some(csi_mod_tilde(15, mod_param)),
        KeyCode::F6 => return Some(csi_mod_tilde(17, mod_param)),
        KeyCode::F7 => return Some(csi_mod_tilde(18, mod_param)),
        KeyCode::F8 => return Some(csi_mod_tilde(19, mod_param)),
        KeyCode::F9 => return Some(csi_mod_tilde(20, mod_param)),
        KeyCode::F10 => return Some(csi_mod_tilde(21, mod_param)),
        KeyCode::F11 => return Some(csi_mod_tilde(23, mod_param)),
        KeyCode::F12 => return Some(csi_mod_tilde(24, mod_param)),

        KeyCode::A | KeyCode::B | KeyCode::C | KeyCode::D | KeyCode::E |
        KeyCode::F | KeyCode::G | KeyCode::H | KeyCode::I | KeyCode::J |
        KeyCode::K | KeyCode::L | KeyCode::M | KeyCode::N | KeyCode::O |
        KeyCode::P | KeyCode::Q | KeyCode::R | KeyCode::S | KeyCode::T |
        KeyCode::U | KeyCode::V | KeyCode::W | KeyCode::X | KeyCode::Y | KeyCode::Z |
        KeyCode::N0 | KeyCode::N1 | KeyCode::N2 | KeyCode::N3 | KeyCode::N4 |
        KeyCode::N5 | KeyCode::N6 | KeyCode::N7 | KeyCode::N8 | KeyCode::N9 | KeyCode::None => {
            let c = match key.code {
                KeyCode::A => b'a', KeyCode::B => b'b', KeyCode::C => b'c', KeyCode::D => b'd', KeyCode::E => b'e',
                KeyCode::F => b'f', KeyCode::G => b'g', KeyCode::H => b'h', KeyCode::I => b'i', KeyCode::J => b'j',
                KeyCode::K => b'k', KeyCode::L => b'l', KeyCode::M => b'm', KeyCode::N => b'n', KeyCode::O => b'o',
                KeyCode::P => b'p', KeyCode::Q => b'q', KeyCode::R => b'r', KeyCode::S => b's', KeyCode::T => b't',
                KeyCode::U => b'u', KeyCode::V => b'v', KeyCode::W => b'w', KeyCode::X => b'x', KeyCode::Y => b'y', KeyCode::Z => b'z',
                KeyCode::N0 => b'0', KeyCode::N1 => b'1', KeyCode::N2 => b'2', KeyCode::N3 => b'3', KeyCode::N4 => b'4',
                KeyCode::N5 => b'5', KeyCode::N6 => b'6', KeyCode::N7 => b'7', KeyCode::N8 => b'8', KeyCode::N9 => b'9',
                _ => character as u8,
            };

            if key.modifier.contains(KM::Ctrl) {
                // Ctrl+A → 0x01, etc.
                let ctrl = (c & 0x1F) as u8;
                seq.push(ctrl);
            }
            else {
                seq.push(c);
            }

            if key.modifier.contains(KM::Alt) {
                let mut with_alt = vec![0x1B];
                with_alt.extend_from_slice(&seq);
                return Some(with_alt);
            }

            return Some(seq);
        }
    }

    Some(seq)
}

fn csi_mod(final_byte: &[u8], mod_param: u8) -> Vec<u8> {
    if mod_param == 1 {
        // No modifier
        let mut v = vec![0x1B, b'['];
        v.extend_from_slice(final_byte);
        v
    } else {
        format!("\x1B[1;{}{}", mod_param, std::str::from_utf8(final_byte).unwrap())
            .into_bytes()
    }
}

fn csi_mod_tilde(code: u8, mod_param: u8) -> Vec<u8> {
    if mod_param == 1 {
        format!("\x1B[{}~", code).into_bytes()
    } else {
        format!("\x1B[{};{}~", code, mod_param).into_bytes()
    }
}

fn ss3_or_csi(final_byte: &[u8], base_code: u8, mod_param: u8) -> Vec<u8> {
    if mod_param == 1 {
        // Traditional SS3 sequence
        let mut v = vec![0x1B, b'O'];
        v.extend_from_slice(final_byte);
        v
    } else {
        // CSI form with modifiers
        format!("\x1B[{};{}~", base_code, mod_param).into_bytes()
    }
}