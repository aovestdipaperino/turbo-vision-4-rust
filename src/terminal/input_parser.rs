// (C) 2025 - Enzo Lombardi

//! Input parser for raw terminal byte streams.
//!
//! This module provides the [`InputParser`] type which converts raw terminal
//! input bytes (ANSI escape sequences) into turbo-vision [`Event`] structures.
//! This is primarily used by the SSH backend to parse input from remote
//! terminal clients.
//!
//! # Supported Input
//!
//! - Regular ASCII characters
//! - UTF-8 multi-byte characters
//! - Control characters (Ctrl+A through Ctrl+Z)
//! - Function keys (F1-F12)
//! - Arrow keys and navigation keys
//! - Mouse events (X10 and SGR formats)
//! - Modifier combinations (Shift, Alt, Ctrl)

use crate::core::event::{
    Event, EventType,
    KB_ESC, KB_ENTER, KB_BACKSPACE, KB_TAB, KB_SHIFT_TAB,
    KB_UP, KB_DOWN, KB_LEFT, KB_RIGHT,
    KB_HOME, KB_END, KB_PGUP, KB_PGDN, KB_INS, KB_DEL,
    KB_F1, KB_F2, KB_F3, KB_F4, KB_F5, KB_F6, KB_F7, KB_F8, KB_F9, KB_F10, KB_F11, KB_F12,
    KB_ALT_A, KB_ALT_B, KB_ALT_C, KB_ALT_D, KB_ALT_E, KB_ALT_F, KB_ALT_G, KB_ALT_H,
    KB_ALT_I, KB_ALT_J, KB_ALT_K, KB_ALT_L, KB_ALT_M, KB_ALT_N, KB_ALT_O, KB_ALT_P,
    KB_ALT_Q, KB_ALT_R, KB_ALT_S, KB_ALT_T, KB_ALT_U, KB_ALT_V, KB_ALT_W, KB_ALT_X,
    KB_ALT_Y, KB_ALT_Z,
    MB_LEFT_BUTTON, MB_MIDDLE_BUTTON, MB_RIGHT_BUTTON,
};
use crate::core::geometry::Point;

/// Parser for raw terminal input bytes.
///
/// Maintains an internal buffer to handle multi-byte sequences and
/// incomplete escape sequences that may arrive across multiple reads.
///
/// # Example
///
/// ```rust
/// use turbo_vision::terminal::InputParser;
///
/// let mut parser = InputParser::new();
///
/// // Feed raw bytes and extract events
/// let events = parser.parse(b"\x1b[A"); // Up arrow
/// assert_eq!(events.len(), 1);
/// ```
pub struct InputParser {
    buffer: Vec<u8>,
}

impl InputParser {
    /// Create a new input parser.
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(64),
        }
    }

    /// Feed raw bytes and extract events.
    ///
    /// Returns a vector of events parsed from the input. Incomplete
    /// sequences are buffered for the next call.
    pub fn parse(&mut self, data: &[u8]) -> Vec<Event> {
        self.buffer.extend_from_slice(data);
        let mut events = Vec::new();

        while !self.buffer.is_empty() {
            match self.try_parse() {
                Some((event, consumed)) => {
                    events.push(event);
                    self.buffer.drain(..consumed);
                }
                None => break, // Need more data
            }
        }
        events
    }

    /// Clear the internal buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Try to parse a single event from the buffer.
    ///
    /// Returns `Some((event, bytes_consumed))` if successful, `None` if
    /// more data is needed.
    fn try_parse(&self) -> Option<(Event, usize)> {
        let buf = &self.buffer;
        if buf.is_empty() {
            return None;
        }

        // ESC sequences
        if buf[0] == 0x1b {
            return self.parse_escape_sequence();
        }

        // Control characters and regular input
        match buf[0] {
            0x0d => Some((Event::keyboard(KB_ENTER), 1)),
            0x09 => Some((Event::keyboard(KB_TAB), 1)),
            0x7f | 0x08 => Some((Event::keyboard(KB_BACKSPACE), 1)),
            0x01..=0x1a => {
                // Control characters (Ctrl+A = 0x01, Ctrl+B = 0x02, etc.)
                let ctrl_code = buf[0] as u16;
                Some((Event::keyboard(ctrl_code), 1))
            }
            c if c >= 0x20 => self.parse_utf8(),
            _ => Some((Event::keyboard(0), 1)), // Null/unknown
        }
    }

    /// Parse an escape sequence starting with ESC (0x1b).
    fn parse_escape_sequence(&self) -> Option<(Event, usize)> {
        let buf = &self.buffer;
        if buf.len() < 2 {
            return None; // Need more data
        }

        match buf[1] {
            b'[' => self.parse_csi(),
            b'O' => self.parse_ss3(),
            c if c.is_ascii_alphabetic() => {
                // ESC + letter = Alt+letter
                if let Some(alt_code) = char_to_alt_code((c as char).to_ascii_lowercase()) {
                    Some((Event::keyboard(alt_code), 2))
                } else {
                    Some((Event::keyboard(KB_ESC), 1))
                }
            }
            _ => Some((Event::keyboard(KB_ESC), 1)),
        }
    }

    /// Parse CSI (Control Sequence Introducer) sequences: ESC [
    fn parse_csi(&self) -> Option<(Event, usize)> {
        let buf = &self.buffer;
        if buf.len() < 3 {
            return None;
        }

        // Check for mouse sequences first
        if buf[2] == b'<' {
            return self.parse_mouse_sgr();
        }
        if buf[2] == b'M' && buf.len() >= 6 {
            return self.parse_mouse_normal();
        }

        // Find final byte (0x40..=0x7E)
        let end = buf[2..]
            .iter()
            .position(|&b| (0x40..=0x7E).contains(&b))
            .map(|i| i + 3)?;

        let params = &buf[2..end - 1];
        let final_byte = buf[end - 1];
        let modifiers = self.parse_modifiers(params);

        let key_code = match final_byte {
            b'A' => apply_modifiers(KB_UP, modifiers),
            b'B' => apply_modifiers(KB_DOWN, modifiers),
            b'C' => apply_modifiers(KB_RIGHT, modifiers),
            b'D' => apply_modifiers(KB_LEFT, modifiers),
            b'H' => apply_modifiers(KB_HOME, modifiers),
            b'F' => apply_modifiers(KB_END, modifiers),
            b'Z' => KB_SHIFT_TAB,
            b'~' => self.parse_tilde(params),
            _ => 0,
        };

        Some((Event::keyboard(key_code), end))
    }

    /// Parse SS3 (Single Shift 3) sequences: ESC O
    fn parse_ss3(&self) -> Option<(Event, usize)> {
        if self.buffer.len() < 3 {
            return None;
        }

        let key_code = match self.buffer[2] {
            b'P' => KB_F1,
            b'Q' => KB_F2,
            b'R' => KB_F3,
            b'S' => KB_F4,
            b'A' => KB_UP,
            b'B' => KB_DOWN,
            b'C' => KB_RIGHT,
            b'D' => KB_LEFT,
            b'H' => KB_HOME,
            b'F' => KB_END,
            _ => 0,
        };

        Some((Event::keyboard(key_code), 3))
    }

    /// Parse tilde-terminated sequences: ESC [ number ~
    fn parse_tilde(&self, params: &[u8]) -> u16 {
        let num: u8 = params
            .iter()
            .take_while(|&&b| b.is_ascii_digit())
            .fold(0, |acc, &b| acc.saturating_mul(10).saturating_add(b - b'0'));

        match num {
            1 | 7 => KB_HOME,
            2 => KB_INS,
            3 => KB_DEL,
            4 | 8 => KB_END,
            5 => KB_PGUP,
            6 => KB_PGDN,
            11 => KB_F1,
            12 => KB_F2,
            13 => KB_F3,
            14 => KB_F4,
            15 => KB_F5,
            17 => KB_F6,
            18 => KB_F7,
            19 => KB_F8,
            20 => KB_F9,
            21 => KB_F10,
            23 => KB_F11,
            24 => KB_F12,
            _ => 0,
        }
    }

    /// Parse modifier parameters from CSI sequences.
    fn parse_modifiers(&self, params: &[u8]) -> u8 {
        let s = std::str::from_utf8(params).unwrap_or("");
        let mod_code: u8 = s
            .split(';')
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let mut mods = 0u8;
        if mod_code & 2 != 0 { mods |= 1; } // Shift
        if mod_code & 4 != 0 { mods |= 2; } // Alt
        if mod_code & 8 != 0 { mods |= 4; } // Control
        mods
    }

    /// Parse a UTF-8 character.
    fn parse_utf8(&self) -> Option<(Event, usize)> {
        if let Ok(s) = std::str::from_utf8(&self.buffer) {
            if let Some(ch) = s.chars().next() {
                return Some((Event::keyboard(ch as u16), ch.len_utf8()));
            }
        }
        if self.buffer.len() < 4 {
            None // Incomplete UTF-8
        } else {
            Some((Event::keyboard(0), 1)) // Invalid - skip byte
        }
    }

    /// Parse X10 mouse protocol: ESC [ M Cb Cx Cy
    fn parse_mouse_normal(&self) -> Option<(Event, usize)> {
        if self.buffer.len() < 6 {
            return None;
        }

        let cb = self.buffer[3].wrapping_sub(32);
        let cx = self.buffer[4].wrapping_sub(32).saturating_sub(1) as i16;
        let cy = self.buffer[5].wrapping_sub(32).saturating_sub(1) as i16;
        let pos = Point::new(cx, cy);

        let event = if cb & 0x40 != 0 {
            // Scroll wheel
            let event_type = if cb & 0x01 != 0 {
                EventType::MouseWheelDown
            } else {
                EventType::MouseWheelUp
            };
            Event::mouse(event_type, pos, 0, false)
        } else if cb & 0x03 == 3 {
            // Button release
            Event::mouse(EventType::MouseUp, pos, 0, false)
        } else {
            let button = match cb & 0x03 {
                0 => MB_LEFT_BUTTON,
                1 => MB_MIDDLE_BUTTON,
                2 => MB_RIGHT_BUTTON,
                _ => MB_LEFT_BUTTON,
            };
            Event::mouse(EventType::MouseDown, pos, button, false)
        };

        Some((event, 6))
    }

    /// Parse SGR mouse protocol: ESC [ < Cb ; Cx ; Cy M/m
    fn parse_mouse_sgr(&self) -> Option<(Event, usize)> {
        // Find the final M or m
        let end = self.buffer[3..]
            .iter()
            .position(|&b| b == b'M' || b == b'm')
            .map(|i| i + 4)?;

        let params = std::str::from_utf8(&self.buffer[3..end - 1]).ok()?;
        let mut parts = params.split(';');

        let cb: u8 = parts.next()?.parse().ok()?;
        let cx: i16 = parts.next()?.parse::<i16>().ok()?.saturating_sub(1);
        let cy: i16 = parts.next()?.parse::<i16>().ok()?.saturating_sub(1);
        let pressed = self.buffer[end - 1] == b'M';
        let pos = Point::new(cx, cy);

        let event = if cb & 64 != 0 {
            // Scroll wheel
            let event_type = if cb & 1 != 0 {
                EventType::MouseWheelDown
            } else {
                EventType::MouseWheelUp
            };
            Event::mouse(event_type, pos, 0, false)
        } else if cb & 32 != 0 {
            // Motion event (drag)
            let button = match cb & 0x03 {
                0 => MB_LEFT_BUTTON,
                1 => MB_MIDDLE_BUTTON,
                2 => MB_RIGHT_BUTTON,
                _ => 0,
            };
            Event::mouse(EventType::MouseMove, pos, button, false)
        } else {
            let button = match cb & 0x03 {
                0 => MB_LEFT_BUTTON,
                1 => MB_MIDDLE_BUTTON,
                2 => MB_RIGHT_BUTTON,
                _ => MB_LEFT_BUTTON,
            };
            let event_type = if pressed {
                EventType::MouseDown
            } else {
                EventType::MouseUp
            };
            Event::mouse(event_type, pos, button, false)
        };

        Some((event, end))
    }
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a lowercase letter to its Alt+letter key code.
fn char_to_alt_code(c: char) -> Option<u16> {
    match c {
        'a' => Some(KB_ALT_A),
        'b' => Some(KB_ALT_B),
        'c' => Some(KB_ALT_C),
        'd' => Some(KB_ALT_D),
        'e' => Some(KB_ALT_E),
        'f' => Some(KB_ALT_F),
        'g' => Some(KB_ALT_G),
        'h' => Some(KB_ALT_H),
        'i' => Some(KB_ALT_I),
        'j' => Some(KB_ALT_J),
        'k' => Some(KB_ALT_K),
        'l' => Some(KB_ALT_L),
        'm' => Some(KB_ALT_M),
        'n' => Some(KB_ALT_N),
        'o' => Some(KB_ALT_O),
        'p' => Some(KB_ALT_P),
        'q' => Some(KB_ALT_Q),
        'r' => Some(KB_ALT_R),
        's' => Some(KB_ALT_S),
        't' => Some(KB_ALT_T),
        'u' => Some(KB_ALT_U),
        'v' => Some(KB_ALT_V),
        'w' => Some(KB_ALT_W),
        'x' => Some(KB_ALT_X),
        'y' => Some(KB_ALT_Y),
        'z' => Some(KB_ALT_Z),
        _ => None,
    }
}

/// Apply modifier bits to a key code (placeholder - modifiers are complex).
fn apply_modifiers(base: u16, _modifiers: u8) -> u16 {
    // For now, just return the base code
    // Full modifier support would require more complex key code handling
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_regular_chars() {
        let mut parser = InputParser::new();
        let events = parser.parse(b"abc");
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].key_code, 'a' as u16);
        assert_eq!(events[1].key_code, 'b' as u16);
        assert_eq!(events[2].key_code, 'c' as u16);
    }

    #[test]
    fn test_parse_arrow_keys() {
        let mut parser = InputParser::new();

        let events = parser.parse(b"\x1b[A");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key_code, KB_UP);

        let events = parser.parse(b"\x1b[B");
        assert_eq!(events[0].key_code, KB_DOWN);

        let events = parser.parse(b"\x1b[C");
        assert_eq!(events[0].key_code, KB_RIGHT);

        let events = parser.parse(b"\x1b[D");
        assert_eq!(events[0].key_code, KB_LEFT);
    }

    #[test]
    fn test_parse_function_keys() {
        let mut parser = InputParser::new();

        let events = parser.parse(b"\x1bOP");
        assert_eq!(events[0].key_code, KB_F1);

        let events = parser.parse(b"\x1b[15~");
        assert_eq!(events[0].key_code, KB_F5);
    }

    #[test]
    fn test_parse_enter_and_backspace() {
        let mut parser = InputParser::new();

        let events = parser.parse(b"\r");
        assert_eq!(events[0].key_code, KB_ENTER);

        let events = parser.parse(b"\x7f");
        assert_eq!(events[0].key_code, KB_BACKSPACE);
    }

    #[test]
    fn test_parse_control_chars() {
        let mut parser = InputParser::new();

        // Ctrl+A = 0x01
        let events = parser.parse(b"\x01");
        assert_eq!(events[0].key_code, 0x01);

        // Ctrl+C = 0x03
        let events = parser.parse(b"\x03");
        assert_eq!(events[0].key_code, 0x03);
    }

    #[test]
    fn test_parse_alt_letters() {
        let mut parser = InputParser::new();

        let events = parser.parse(b"\x1bx");
        assert_eq!(events[0].key_code, KB_ALT_X);

        let events = parser.parse(b"\x1bf");
        assert_eq!(events[0].key_code, KB_ALT_F);
    }

    #[test]
    fn test_parse_mouse_sgr() {
        let mut parser = InputParser::new();

        // Left button down at (10, 5)
        let events = parser.parse(b"\x1b[<0;11;6M");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].what, EventType::MouseDown);
        assert_eq!(events[0].mouse.pos.x, 10);
        assert_eq!(events[0].mouse.pos.y, 5);
        assert_eq!(events[0].mouse.buttons, MB_LEFT_BUTTON);
    }

    #[test]
    fn test_incomplete_sequence() {
        let mut parser = InputParser::new();

        // Incomplete escape sequence
        let events = parser.parse(b"\x1b[");
        assert_eq!(events.len(), 0);

        // Complete it
        let events = parser.parse(b"A");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key_code, KB_UP);
    }
}
