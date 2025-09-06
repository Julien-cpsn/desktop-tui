use crate::tui_window::CustomKeyboardControl;
use appcui::input::{Key, KeyModifier};
use appcui::prelude::{EventProcessStatus, KeyCode, OnKeyPressed};
use virtual_terminal::Input;

impl OnKeyPressed for CustomKeyboardControl {
    fn on_key_pressed(&mut self, key: Key, _character: char) -> EventProcessStatus {
        if !self.has_focus() {
            return EventProcessStatus::Ignored;
        }

        if key.modifier == KeyModifier::Ctrl && key.code == KeyCode::C {
            self.tx.send_blocking(Input::Terminate).ok();
            self.should_exit = true;
        }
        else {
            if let Some(data) = to_escape_sequence_vec(key) {
                self.tx
                    .send_blocking(Input::Data(data))
                    .ok();
            }
        }

        EventProcessStatus::Processed
    }
}

pub fn to_escape_sequence_vec(key: Key) -> Option<Vec<u8>> {
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

        KeyCode::None => return None,

        _ => {
            use appcui::prelude::KeyCode::*;

            let c = match key.code {
                A => b'a', B => b'b', C => b'c', D => b'd', E => b'e',
                F => b'f', G => b'g', H => b'h', I => b'i', J => b'j',
                K => b'k', L => b'l', M => b'm', N => b'n', O => b'o',
                P => b'p', Q => b'q', R => b'r', S => b's', T => b't',
                U => b'u', V => b'v', W => b'w', X => b'x', Y => b'y', Z => b'z',
                N0 => b'0', N1 => b'1', N2 => b'2', N3 => b'3', N4 => b'4',
                N5 => b'5', N6 => b'6', N7 => b'7', N8 => b'8', N9 => b'9',
                _ => unreachable!(),
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