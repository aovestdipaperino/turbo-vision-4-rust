// (C) 2025 - Enzo Lombardi

//! Crossterm-based backend implementation.
//!
//! This module provides the [`CrosstermBackend`] which implements the [`Backend`]
//! trait using crossterm for local terminal I/O. This is the default backend
//! used when running turbo-vision applications locally.

use std::io::{self, Write, stdout};
use std::sync::Once;
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event as CTEvent, KeyEventKind, MouseButton, MouseEventKind},
    execute,
    terminal::{self, window_size},
};

use super::backend::{Backend, Capabilities};
use crate::core::event::{
    EscSequenceTracker, Event, EventType, MB_LEFT_BUTTON, MB_MIDDLE_BUTTON, MB_RIGHT_BUTTON,
};
use crate::core::geometry::Point;

/// Escape sequences that take the terminal out of every mouse reporting mode
/// this crate can turn on, newest protocol first.
///
/// Emitting all of them is deliberate: a terminal that never had a given mode
/// enabled ignores the matching reset, while a terminal left in an older mode
/// (because it silently declined SGR) still gets switched off.
const MOUSE_OFF: &[u8] = b"\x1b[?1006l\x1b[?1015l\x1b[?1003l\x1b[?1002l\x1b[?1000l";

/// Restore the terminal from anywhere, without needing a [`CrosstermBackend`].
///
/// Writes the mouse-reporting resets *before* leaving the alternate screen and
/// *before* dropping out of raw mode, so no report can reach the shell that
/// inherits the terminal afterwards. Every step is attempted even if an earlier
/// one fails, which is what makes this usable from a panic hook.
pub fn restore_terminal() {
    let mut stdout = stdout();

    // Mouse first: while still in raw mode and on the alternate screen, so a
    // report generated mid-teardown is swallowed by the TUI, not the shell.
    let _ = stdout.write_all(MOUSE_OFF);
    // Re-enable autowrap (DECAWM) and show the cursor.
    let _ = stdout.write_all(b"\x1b[?7h\x1b[?25h");
    let _ = stdout.flush();

    let _ = execute!(stdout, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
    let _ = stdout.flush();
}

static PANIC_HOOK: Once = Once::new();

/// Install a panic hook that restores the terminal before the normal hook
/// prints its message, so the backtrace lands on a usable screen and the shell
/// is not left reporting mouse movement. Installing it more than once is a
/// no-op.
fn install_panic_hook() {
    PANIC_HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            previous(info);
        }));
    });
}

/// Crossterm-based terminal backend for local terminal I/O.
///
/// This backend uses the crossterm crate to interact with the terminal,
/// providing support for:
/// - Raw mode and alternate screen
/// - Keyboard input with modifiers
/// - Mouse events (clicks, scroll, drag)
/// - Terminal resize detection
/// - ANSI color output
///
/// # Example
///
/// ```rust,no_run
/// use turbo_vision::terminal::CrosstermBackend;
/// use turbo_vision::terminal::Backend;
///
/// let mut backend = CrosstermBackend::new().unwrap();
/// backend.init().unwrap();
/// // ... use backend ...
/// backend.cleanup().unwrap();
/// ```
pub struct CrosstermBackend {
    esc_tracker: EscSequenceTracker,
    last_mouse_pos: Point,
    last_mouse_buttons: u8,
    last_click_time: Option<Instant>,
    last_click_pos: Point,
    capabilities: Capabilities,
}

impl CrosstermBackend {
    /// Create a new crossterm backend.
    ///
    /// This does not initialize the terminal - call [`init`](Self::init) to
    /// enter raw mode and set up the terminal for TUI operation.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal capabilities cannot be queried.
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            esc_tracker: EscSequenceTracker::new(),
            last_mouse_pos: Point::zero(),
            last_mouse_buttons: 0,
            last_click_time: None,
            last_click_pos: Point::zero(),
            capabilities: Capabilities {
                mouse: true,
                colors_256: true,
                true_color: true, // crossterm supports true color
                bracketed_paste: false,
                focus_events: false,
                kitty_keyboard: false,
            },
        })
    }

    /// Set the ESC timeout in milliseconds.
    ///
    /// This controls how long the backend waits after ESC to detect
    /// ESC+letter sequences (for macOS Alt emulation).
    pub fn set_esc_timeout(&mut self, timeout_ms: u64) {
        self.esc_tracker.set_timeout(timeout_ms);
    }

    /// Convert crossterm mouse event to turbo-vision Event.
    fn convert_mouse_event(&mut self, mouse: event::MouseEvent) -> Option<Event> {
        let pos = Point::new(mouse.column as i16, mouse.row as i16);

        // Handle scroll wheel events separately
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                return Some(Event::mouse(EventType::MouseWheelUp, pos, 0, false));
            }
            MouseEventKind::ScrollDown => {
                return Some(Event::mouse(EventType::MouseWheelDown, pos, 0, false));
            }
            _ => {}
        }

        // Convert button state to our format
        let buttons =
            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left)
                | MouseEventKind::Drag(MouseButton::Left) => MB_LEFT_BUTTON,
                MouseEventKind::Down(MouseButton::Right)
                | MouseEventKind::Drag(MouseButton::Right) => MB_RIGHT_BUTTON,
                MouseEventKind::Down(MouseButton::Middle)
                | MouseEventKind::Drag(MouseButton::Middle) => MB_MIDDLE_BUTTON,
                MouseEventKind::Up(_) => 0,
                MouseEventKind::Moved => self.last_mouse_buttons,
                _ => return None,
            };

        // Determine event type and detect double-clicks
        let (event_type, is_double_click) = match mouse.kind {
            MouseEventKind::Down(_) => {
                // Check for double-click: same position, within 500ms
                let is_double = if let Some(last_time) = self.last_click_time {
                    let elapsed = last_time.elapsed();
                    elapsed.as_millis() <= 500 && pos == self.last_click_pos
                } else {
                    false
                };

                // Update click tracking
                self.last_click_time = Some(Instant::now());
                self.last_click_pos = pos;
                self.last_mouse_buttons = buttons;
                self.last_mouse_pos = pos;

                (EventType::MouseDown, is_double)
            }
            MouseEventKind::Up(_) => {
                self.last_mouse_buttons = 0;
                (EventType::MouseUp, false)
            }
            MouseEventKind::Drag(_) | MouseEventKind::Moved => {
                self.last_mouse_pos = pos;
                (EventType::MouseMove, false)
            }
            _ => return None,
        };

        // Carry the keyboard modifiers held during the mouse event (e.g. Alt for
        // rectangular block selection). `Event::mouse` defaults them to empty.
        let mut ev = Event::mouse(event_type, pos, buttons, is_double_click);
        ev.key_modifiers = mouse.modifiers;
        Some(ev)
    }
}

impl Backend for CrosstermBackend {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn init(&mut self) -> io::Result<()> {
        install_panic_hook();
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            event::EnableMouseCapture
        )?;

        // Disable autowrap (DECAWM) to prevent scrolling when writing to bottom-right corner
        write!(stdout, "\x1b[?7l")?;
        stdout.flush()?;

        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        restore_terminal();
        Ok(())
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        if event::poll(timeout)? {
            match event::read()? {
                CTEvent::Key(key) => {
                    // On Windows, crossterm sends both Press and Release events
                    // Filter to only process Press events to avoid duplicates
                    if key.kind != KeyEventKind::Press {
                        return Ok(None);
                    }

                    let key_code = self.esc_tracker.process_key(key);
                    if key_code == 0 {
                        // ESC sequence in progress, don't generate event yet
                        return Ok(None);
                    }

                    // Create event preserving modifiers from original crossterm event
                    Ok(Some(Event {
                        what: EventType::Keyboard,
                        key_code,
                        key_modifiers: key.modifiers,
                        ..Event::nothing()
                    }))
                }
                CTEvent::Mouse(mouse) => Ok(self.convert_mouse_event(mouse)),
                CTEvent::Resize(_, _) => {
                    // Emit a broadcast so the application can re-layout
                    Ok(Some(Event::broadcast(crate::core::command::CM_REDRAW)))
                }
                _ => Ok(None),
            }
        } else {
            // No crossterm event — check if a pending ESC has timed out
            if let Some(key_code) = self.esc_tracker.check_timeout() {
                return Ok(Some(Event::keyboard(key_code)));
            }
            Ok(None)
        }
    }

    fn write_raw(&mut self, data: &[u8]) -> io::Result<()> {
        stdout().write_all(data)
    }

    fn flush(&mut self) -> io::Result<()> {
        stdout().flush()
    }

    fn show_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, cursor::MoveTo(x, y), cursor::Show)?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, cursor::Hide)?;
        Ok(())
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn suspend(&mut self) -> io::Result<()> {
        restore_terminal();
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            event::EnableMouseCapture
        )?;

        // Disable autowrap (DECAWM) to prevent scrolling when writing to bottom-right corner
        write!(stdout, "\x1b[?7l")?;
        stdout.flush()?;

        Ok(())
    }

    fn cell_aspect_ratio(&self) -> (i16, i16) {
        if let Ok(ws) = window_size() {
            // WindowSize has: columns, rows, width (pixels), height (pixels)
            if ws.width > 0 && ws.height > 0 && ws.columns > 0 && ws.rows > 0 {
                let cell_width = ws.width as f32 / ws.columns as f32;
                let cell_height = ws.height as f32 / ws.rows as f32;

                if cell_width > 0.0 {
                    // Calculate ratio: how many horizontal cells equal one vertical cell
                    let ratio = (cell_height / cell_width).round() as i16;
                    return (ratio.max(1), 1);
                }
            }
        }
        // Fallback: typical terminal fonts are ~10x16 pixels (1.6:1 ratio)
        (2, 1)
    }
}

impl Default for CrosstermBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create CrosstermBackend")
    }
}

#[cfg(test)]
mod tests {
    use super::MOUSE_OFF;

    /// Every mouse mode the local and SSH backends can enable must have a
    /// matching reset here, or the shell inherits a terminal that keeps
    /// reporting (issue #109).
    #[test]
    fn mouse_off_resets_every_mode_we_enable() {
        for mode in ["1000", "1002", "1003", "1006", "1015"] {
            let reset = format!("\x1b[?{mode}l");
            assert!(
                MOUSE_OFF
                    .windows(reset.len())
                    .any(|w| w == reset.as_bytes()),
                "mouse reset for mode {mode} is missing"
            );
        }
    }

    /// SGR must be reset before the older modes: a terminal still in SGR would
    /// otherwise answer the X10 reset with an SGR-encoded report.
    #[test]
    fn mouse_off_resets_sgr_first() {
        let seq = String::from_utf8(MOUSE_OFF.to_vec()).expect("ASCII escapes");
        assert!(seq.find("1006l").unwrap() < seq.find("1000l").unwrap());
    }
}
