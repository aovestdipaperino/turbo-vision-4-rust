// (C) 2025 - Enzo Lombardi

//! Event system - keyboard and mouse event handling with Borland-compatible key codes.

use super::command::CommandId;
use super::geometry::Point;
use crossterm::event::{KeyCode as CKC, KeyEvent, KeyModifiers};
use std::fmt;
use std::time::{Duration, Instant};

/// Keyboard code (scan code + character)
pub type KeyCode = u16;

// Special key codes (high byte = scan code, low byte = char)
pub const KB_ESC: KeyCode = 0x011B;
pub const KB_ENTER: KeyCode = 0x1C0D;
pub const KB_BACKSPACE: KeyCode = 0x0E08;
pub const KB_TAB: KeyCode = 0x0F09;
pub const KB_SHIFT_TAB: KeyCode = 0x0F00; // Shift+Tab for reverse focus

// Function keys
pub const KB_F1: KeyCode = 0x3B00;
pub const KB_F2: KeyCode = 0x3C00;
pub const KB_F3: KeyCode = 0x3D00;
pub const KB_F4: KeyCode = 0x3E00;
pub const KB_F5: KeyCode = 0x3F00;
pub const KB_F6: KeyCode = 0x4000;
pub const KB_F7: KeyCode = 0x4100;
pub const KB_F8: KeyCode = 0x4200;
pub const KB_F9: KeyCode = 0x4300;
pub const KB_F10: KeyCode = 0x4400;
pub const KB_F11: KeyCode = 0x8500;
pub const KB_F12: KeyCode = 0x8600;
pub const KB_SHIFT_F12: KeyCode = 0x8601; // Shift+F12 for active view dump

// Arrow keys
pub const KB_UP: KeyCode = 0x4800;
pub const KB_DOWN: KeyCode = 0x5000;
pub const KB_LEFT: KeyCode = 0x4B00;
pub const KB_RIGHT: KeyCode = 0x4D00;

pub const KB_HOME: KeyCode = 0x4700;
pub const KB_END: KeyCode = 0x4F00;
pub const KB_PGUP: KeyCode = 0x4900;
pub const KB_PGDN: KeyCode = 0x5100;
pub const KB_INS: KeyCode = 0x5200;
pub const KB_DEL: KeyCode = 0x5300;

// Alt + letter
pub const KB_ALT_X: KeyCode = 0x2D00;
pub const KB_ALT_F: KeyCode = 0x2100;
pub const KB_ALT_E: KeyCode = 0x1200;
pub const KB_ALT_H: KeyCode = 0x2300;
pub const KB_ALT_O: KeyCode = 0x1800;
pub const KB_ALT_A: KeyCode = 0x1E00;
pub const KB_ALT_F3: KeyCode = 0x6A00;

// ESC + letter (for macOS Alt emulation)
pub const KB_ESC_F: KeyCode = 0x2101; // ESC+F
pub const KB_ESC_H: KeyCode = 0x2301; // ESC+H
pub const KB_ESC_X: KeyCode = 0x2D01; // ESC+X
pub const KB_ESC_A: KeyCode = 0x1E01; // ESC+A
pub const KB_ESC_O: KeyCode = 0x1801; // ESC+O
pub const KB_ESC_E: KeyCode = 0x1201; // ESC+E (Edit menu)
pub const KB_ESC_S: KeyCode = 0x1F01; // ESC+S (Search menu)
pub const KB_ESC_V: KeyCode = 0x2F01; // ESC+V (View menu)

pub const KB_CTRL_A: KeyCode = 0x0001; // CTRL+A
pub const KB_CTRL_B: KeyCode = 0x0002; // CTRL+B
pub const KB_CTRL_C: KeyCode = 0x0003; // CTRL+C
pub const KB_CTRL_D: KeyCode = 0x0004; // CTRL+D
pub const KB_CTRL_E: KeyCode = 0x0005; // CTRL+E
pub const KB_CTRL_F: KeyCode = 0x0006; // CTRL+F
pub const KB_CTRL_G: KeyCode = 0x0007; // CTRL+G
pub const KB_CTRL_H: KeyCode = 0x0008; // CTRL+H
pub const KB_CTRL_I: KeyCode = 0x0009; // CTRL+I
pub const KB_CTRL_J: KeyCode = 0x000a; // CTRL+J
pub const KB_CTRL_K: KeyCode = 0x000b; // CTRL+K
pub const KB_CTRL_L: KeyCode = 0x000c; // CTRL+L
pub const KB_CTRL_M: KeyCode = 0x000d; // CTRL+M
pub const KB_CTRL_N: KeyCode = 0x000e; // CTRL+N
pub const KB_CTRL_O: KeyCode = 0x000f; // CTRL+O
pub const KB_CTRL_P: KeyCode = 0x0010; // CTRL+P
pub const KB_CTRL_Q: KeyCode = 0x0011; // CTRL+Q
pub const KB_CTRL_R: KeyCode = 0x0012; // CTRL+R
pub const KB_CTRL_S: KeyCode = 0x0013; // CTRL+S
pub const KB_CTRL_T: KeyCode = 0x0014; // CTRL+T
pub const KB_CTRL_U: KeyCode = 0x0015; // CTRL+U
pub const KB_CTRL_V: KeyCode = 0x0016; // CTRL+V
pub const KB_CTRL_W: KeyCode = 0x0017; // CTRL+W
pub const KB_CTRL_X: KeyCode = 0x0018; // CTRL+X
pub const KB_CTRL_Y: KeyCode = 0x0019; // CTRL+Y
pub const KB_CTRL_Z: KeyCode = 0x001a; // CTRL+Z

// Double ESC for closing dialogs
pub const KB_ESC_ESC: KeyCode = 0x011C; // Double ESC

/// Event types (matching original Turbo Vision)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Nothing,
    Keyboard,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseAuto,
    MouseWheelUp,   // Mouse wheel scrolled up
    MouseWheelDown, // Mouse wheel scrolled down
    Command,
    Broadcast,
}

// Event masks (for filtering)
pub const EV_NOTHING: u16 = 0x0000;
pub const EV_MOUSE_DOWN: u16 = 0x0001;
pub const EV_MOUSE_UP: u16 = 0x0002;
pub const EV_MOUSE_MOVE: u16 = 0x0004;
pub const EV_MOUSE_AUTO: u16 = 0x0008;
pub const EV_MOUSE_WHEEL_UP: u16 = 0x0010;
pub const EV_MOUSE_WHEEL_DOWN: u16 = 0x0020;
pub const EV_MOUSE: u16 = 0x003F; // All mouse events (including wheel)
pub const EV_KEYBOARD: u16 = 0x0040;
pub const EV_COMMAND: u16 = 0x0100;
pub const EV_BROADCAST: u16 = 0x0200;
pub const EV_MESSAGE: u16 = 0xFF00; // Command | Broadcast

// Mouse button masks
pub const MB_LEFT_BUTTON: u8 = 0x01;
pub const MB_MIDDLE_BUTTON: u8 = 0x02;
pub const MB_RIGHT_BUTTON: u8 = 0x04;

/// Mouse event data
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub pos: Point,
    pub buttons: u8, // button state (bit flags)
    pub double_click: bool,
}

/// A unified event structure
///
/// # Examples
///
/// ```
/// use turbo_vision::core::event::{Event, EventType, KB_ESC, KB_ENTER};
/// use turbo_vision::core::command::CM_QUIT;
///
/// // Create keyboard event
/// let esc_event = Event::keyboard(KB_ESC);
/// assert_eq!(esc_event.key_code, KB_ESC);
///
/// // Create command event
/// let quit_cmd = Event::command(CM_QUIT);
/// assert_eq!(quit_cmd.command, CM_QUIT);
///
/// // Clear an event to mark it as handled
/// let mut event = Event::keyboard(KB_ENTER);
/// event.clear();
/// assert_eq!(event.what, EventType::Nothing);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub what: EventType,
    pub key_code: KeyCode,
    pub key_modifiers: KeyModifiers,
    pub mouse: MouseEvent,
    pub command: CommandId,
}

impl Event {
    pub fn nothing() -> Self {
        Self {
            what: EventType::Nothing,
            key_code: 0,
            key_modifiers: KeyModifiers::empty(),
            mouse: MouseEvent {
                pos: Point::zero(),
                buttons: 0,
                double_click: false,
            },
            command: 0,
        }
    }

    pub fn keyboard(key_code: KeyCode) -> Self {
        Self {
            what: EventType::Keyboard,
            key_code,
            key_modifiers: KeyModifiers::empty(),
            ..Self::nothing()
        }
    }

    pub fn command(cmd: CommandId) -> Self {
        Self {
            what: EventType::Command,
            command: cmd,
            ..Self::nothing()
        }
    }

    pub fn broadcast(cmd: CommandId) -> Self {
        Self {
            what: EventType::Broadcast,
            command: cmd,
            ..Self::nothing()
        }
    }

    pub fn mouse(event_type: EventType, pos: Point, buttons: u8, double_click: bool) -> Self {
        Self {
            what: event_type,
            mouse: MouseEvent { pos, buttons, double_click },
            ..Self::nothing()
        }
    }

    pub fn from_crossterm_key(key_event: KeyEvent) -> Self {
        let key_code = crossterm_to_keycode(key_event);
        Self {
            what: EventType::Keyboard,
            key_code,
            key_modifiers: key_event.modifiers,
            ..Self::nothing()
        }
    }

    /// Mark this event as handled (clear it)
    pub fn clear(&mut self) {
        self.what = EventType::Nothing;
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::nothing()
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.what {
            EventType::Nothing => write!(f, "Event::Nothing"),
            EventType::Keyboard => {
                write!(f, "Event::Keyboard(key_code={:#06x}", self.key_code)?;
                if !self.key_modifiers.is_empty() {
                    write!(f, ", modifiers={:?}", self.key_modifiers)?;
                }
                write!(f, ")")
            }
            EventType::MouseDown => write!(
                f,
                "Event::MouseDown({}, buttons={:#04x}{})",
                self.mouse.pos,
                self.mouse.buttons,
                if self.mouse.double_click { ", double_click" } else { "" }
            ),
            EventType::MouseUp => write!(f, "Event::MouseUp({}, buttons={:#04x})", self.mouse.pos, self.mouse.buttons),
            EventType::MouseMove => write!(f, "Event::MouseMove({}, buttons={:#04x})", self.mouse.pos, self.mouse.buttons),
            EventType::MouseAuto => write!(f, "Event::MouseAuto({}, buttons={:#04x})", self.mouse.pos, self.mouse.buttons),
            EventType::MouseWheelUp => write!(f, "Event::MouseWheelUp({})", self.mouse.pos),
            EventType::MouseWheelDown => write!(f, "Event::MouseWheelDown({})", self.mouse.pos),
            EventType::Command => write!(f, "Event::Command({:#06x})", self.command),
            EventType::Broadcast => write!(f, "Event::Broadcast({:#06x})", self.command),
        }
    }
}

/// ESC sequence tracker for macOS Alt emulation
#[derive(Default)]
pub struct EscSequenceTracker {
    last_esc_time: Option<Instant>,
    waiting_for_char: bool,
}

impl EscSequenceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a key event, handling ESC sequences
    /// Returns the appropriate KeyCode
    pub fn process_key(&mut self, key: KeyEvent) -> KeyCode {
        // Check if this is ESC
        if matches!(key.code, CKC::Esc) {
            let now = Instant::now();

            // Check if this is a second ESC within 500ms
            if let Some(last_time) = self.last_esc_time {
                if now.duration_since(last_time) < Duration::from_millis(500) {
                    // Double ESC!
                    self.last_esc_time = None;
                    self.waiting_for_char = false;
                    return KB_ESC_ESC;
                }
            }

            // First ESC - wait for next character
            self.last_esc_time = Some(now);
            self.waiting_for_char = true;
            return 0; // Don't generate event yet
        }

        // If we're waiting for a character after ESC
        if self.waiting_for_char {
            self.waiting_for_char = false;
            self.last_esc_time = None;

            // Map ESC+letter to Alt codes
            if let CKC::Char(c) = key.code {
                return match c.to_ascii_lowercase() {
                    'f' => KB_ESC_F,
                    'h' => KB_ESC_H,
                    'x' => KB_ESC_X,
                    'a' => KB_ESC_A,
                    'o' => KB_ESC_O,
                    'e' => KB_ESC_E,
                    's' => KB_ESC_S,
                    'v' => KB_ESC_V,
                    _ => crossterm_to_keycode(key),
                };
            }
        }

        // Check if ESC timeout expired (user pressed ESC but waited too long)
        if let Some(last_time) = self.last_esc_time {
            if Instant::now().duration_since(last_time) > Duration::from_millis(500) {
                self.last_esc_time = None;
                self.waiting_for_char = false;
                // Too late, treat as single ESC
                if matches!(key.code, CKC::Char(_)) {
                    return crossterm_to_keycode(key);
                }
            }
        }

        crossterm_to_keycode(key)
    }
}

/// Convert crossterm KeyEvent to our KeyCode
fn crossterm_to_keycode(key: KeyEvent) -> KeyCode {
    match key.code {
        CKC::Char(c) => {
            // Check for Ctrl modifier first (Ctrl+letter generates ASCII control codes)
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl + letter produces ASCII control codes (0x01-0x1A for A-Z)
                let c_lower = c.to_ascii_lowercase();
                if c_lower >= 'a' && c_lower <= 'z' {
                    return (c_lower as u16) - ('a' as u16) + 1; // Ctrl+A = 0x01, Ctrl+B = 0x02, etc.
                }
            }

            // Check for Alt modifier
            if key.modifiers.contains(KeyModifiers::ALT) {
                // Alt + letter
                match c.to_ascii_lowercase() {
                    'a' => return KB_ALT_A,
                    'e' => return KB_ALT_E,
                    'f' => return KB_ALT_F,
                    'h' => return KB_ALT_H,
                    'o' => return KB_ALT_O,
                    'x' => return KB_ALT_X,
                    _ => {}
                }
            }

            c as u16
        }
        CKC::Enter => KB_ENTER,
        CKC::Backspace => KB_BACKSPACE,
        CKC::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                KB_SHIFT_TAB
            } else {
                KB_TAB
            }
        }
        CKC::BackTab => KB_SHIFT_TAB, // Some terminals send BackTab for Shift+Tab
        CKC::Esc => KB_ESC,
        CKC::Up => KB_UP,
        CKC::Down => KB_DOWN,
        CKC::Left => KB_LEFT,
        CKC::Right => KB_RIGHT,
        CKC::Home => KB_HOME,
        CKC::End => KB_END,
        CKC::PageUp => KB_PGUP,
        CKC::PageDown => KB_PGDN,
        CKC::Insert => KB_INS,
        CKC::Delete => KB_DEL,
        CKC::F(1) => KB_F1,
        CKC::F(2) => KB_F2,
        CKC::F(3) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                KB_ALT_F3
            } else {
                KB_F3
            }
        }
        CKC::F(4) => KB_F4,
        CKC::F(5) => KB_F5,
        CKC::F(6) => KB_F6,
        CKC::F(7) => KB_F7,
        CKC::F(8) => KB_F8,
        CKC::F(9) => KB_F9,
        CKC::F(10) => KB_F10,
        CKC::F(11) => KB_F11,
        CKC::F(12) => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                KB_SHIFT_F12
            } else {
                KB_F12
            }
        }
        _ => 0,
    }
}
