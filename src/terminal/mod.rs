// (C) 2025 - Enzo Lombardi

//! Terminal abstraction layer providing crossterm-based rendering.
//!
//! This module provides the [`Terminal`] type which handles all interaction
//! with the physical terminal including:
//! - Raw mode management and alternate screen
//! - Double-buffered rendering for flicker-free updates
//! - Event polling (keyboard, mouse, resize)
//! - Mouse capture and tracking
//! - Clipping region management
//! - ANSI dump support for debugging
//!
//! # Examples
//!
//! Basic terminal usage:
//!
//! ```rust,no_run
//! use turbo_vision::Terminal;
//! use turbo_vision::core::error::Result;
//!
//! fn main() -> Result<()> {
//!     let mut terminal = Terminal::init()?;
//!
//!     // Use terminal for rendering...
//!
//!     terminal.shutdown()?;
//!     Ok(())
//! }
//! ```

use crate::core::draw::Cell;
use crate::core::event::{Event, EventType, EscSequenceTracker, MB_LEFT_BUTTON, MB_MIDDLE_BUTTON, MB_RIGHT_BUTTON, KB_F12, KB_SHIFT_F12};
use crate::core::geometry::Point;
use crate::core::palette::Attr;
use crate::core::ansi_dump;
use crate::core::error::Result;
use crossterm::{
    cursor, execute, queue, style,
    terminal::{self},
    event::{self, Event as CTEvent, MouseEventKind, MouseButton},
};
use std::io::{self, Write, stdout};
use std::time::{Duration, Instant};

/// Terminal abstraction for crossterm backend
pub struct Terminal {
    buffer: Vec<Vec<Cell>>,
    prev_buffer: Vec<Vec<Cell>>,
    width: u16,
    height: u16,
    esc_tracker: EscSequenceTracker,
    last_mouse_pos: Point,
    last_mouse_buttons: u8,
    last_click_time: Option<Instant>,
    last_click_pos: Point,
    clip_stack: Vec<crate::core::geometry::Rect>,
    active_view_bounds: Option<crate::core::geometry::Rect>,
    pending_event: Option<Event>,  // Event queue for putEvent() - matches Borland's TProgram::pending
}

impl Terminal {
    /// Initializes a new terminal instance in raw mode.
    ///
    /// This function sets up the terminal for full-screen TUI operation by:
    /// - Enabling raw mode (no line buffering, no echo)
    /// - Entering alternate screen buffer
    /// - Hiding the cursor
    /// - Enabling mouse capture
    /// - Creating double buffers for flicker-free rendering
    ///
    /// The terminal is automatically restored to normal mode when dropped,
    /// but it's recommended to call [`shutdown()`](Self::shutdown) explicitly
    /// for better error handling.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Terminal capabilities cannot be queried
    /// - Raw mode cannot be enabled
    /// - Alternate screen cannot be entered
    /// - Mouse capture cannot be enabled
    ///
    /// Common causes include:
    /// - Running in a non-terminal environment (e.g., redirected output)
    /// - Terminal doesn't support required capabilities
    /// - Permission denied for terminal operations
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use turbo_vision::Terminal;
    /// use turbo_vision::core::error::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let mut terminal = Terminal::init()?;
    ///     // Terminal is now in raw mode with alternate screen
    ///     terminal.shutdown()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn init() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            event::EnableMouseCapture  // Enable mouse support
        )?;

        let (width, height) = terminal::size()?;

        let empty_cell = Cell::new(' ', Attr::from_u8(0x07));
        let buffer = vec![vec![empty_cell; width as usize]; height as usize];
        let prev_buffer = vec![vec![empty_cell; width as usize]; height as usize];

        Ok(Self {
            buffer,
            prev_buffer,
            width,
            height,
            esc_tracker: EscSequenceTracker::new(),
            last_mouse_pos: Point::zero(),
            last_mouse_buttons: 0,
            last_click_time: None,
            last_click_pos: Point::zero(),
            clip_stack: Vec::new(),
            active_view_bounds: None,
            pending_event: None,
        })
    }

    /// Shuts down the terminal and restores normal mode.
    ///
    /// This function restores the terminal to its original state by:
    /// - Disabling mouse capture
    /// - Showing the cursor
    /// - Leaving alternate screen buffer
    /// - Disabling raw mode
    ///
    /// # Errors
    ///
    /// Returns an error if terminal restoration fails. In most cases, the
    /// terminal will still be usable even if an error occurs.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use turbo_vision::Terminal;
    /// # use turbo_vision::core::error::Result;
    /// # fn main() -> Result<()> {
    /// let mut terminal = Terminal::init()?;
    /// // Use terminal...
    /// terminal.shutdown()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn shutdown(&mut self) -> Result<()> {
        let mut stdout = stdout();
        execute!(
            stdout,
            event::DisableMouseCapture,  // Disable mouse support
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// Get terminal size
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Set the bounds of the currently active view (for F11 screen dumps)
    pub fn set_active_view_bounds(&mut self, bounds: crate::core::geometry::Rect) {
        self.active_view_bounds = Some(bounds);
    }

    /// Clear the active view bounds
    pub fn clear_active_view_bounds(&mut self) {
        self.active_view_bounds = None;
    }

    /// Push a clipping region onto the stack
    pub fn push_clip(&mut self, rect: crate::core::geometry::Rect) {
        self.clip_stack.push(rect);
    }

    /// Pop a clipping region from the stack
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Get the current effective clipping region (intersection of all regions on stack)
    fn get_clip_rect(&self) -> Option<crate::core::geometry::Rect> {
        if self.clip_stack.is_empty() {
            None
        } else {
            let mut result = self.clip_stack[0];
            for clip in &self.clip_stack[1..] {
                result = result.intersect(clip);
            }
            Some(result)
        }
    }

    /// Check if a point is within the current clipping region
    fn is_clipped(&self, x: i16, y: i16) -> bool {
        if let Some(clip) = self.get_clip_rect() {
            !clip.contains(Point::new(x, y))
        } else {
            false
        }
    }

    /// Write a cell at the given position
    pub fn write_cell(&mut self, x: u16, y: u16, cell: Cell) {
        let x_i16 = x as i16;
        let y_i16 = y as i16;

        // Check terminal bounds
        if (x as usize) >= self.width as usize || (y as usize) >= self.height as usize {
            return;
        }

        // Check clipping
        if self.is_clipped(x_i16, y_i16) {
            return;
        }

        self.buffer[y as usize][x as usize] = cell;
    }

    /// Write a line from a draw buffer
    pub fn write_line(&mut self, x: u16, y: u16, cells: &[Cell]) {
        let y_i16 = y as i16;

        if (y as usize) >= self.height as usize {
            return;
        }

        let max_width = (self.width as usize).saturating_sub(x as usize);
        let len = cells.len().min(max_width);

        for (i, cell) in cells.iter().enumerate().take(len) {
            let cell_x = (x as usize) + i;
            let cell_x_i16 = cell_x as i16;

            // Check clipping for each cell
            if !self.is_clipped(cell_x_i16, y_i16) {
                self.buffer[y as usize][cell_x] = *cell;
            }
        }
    }

    /// Clear the entire screen
    pub fn clear(&mut self) {
        let empty_cell = Cell::new(' ', Attr::from_u8(0x07));
        for row in &mut self.buffer {
            for cell in row {
                *cell = empty_cell;
            }
        }
    }

    /// Flush changes to the terminal
    pub fn flush(&mut self) -> io::Result<()> {
        let mut stdout = stdout();

        for y in 0..self.height as usize {
            let mut x = 0;
            while x < self.width as usize {
                // Find the start of a changed region
                if self.buffer[y][x] == self.prev_buffer[y][x] {
                    x += 1;
                    continue;
                }

                // Find the end of the changed region
                let start_x = x;
                let current_attr = self.buffer[y][x].attr;

                while x < self.width as usize
                    && self.buffer[y][x] != self.prev_buffer[y][x]
                    && self.buffer[y][x].attr == current_attr
                {
                    x += 1;
                }

                // Move cursor and set colors
                queue!(
                    stdout,
                    cursor::MoveTo(start_x as u16, y as u16),
                    style::SetForegroundColor(current_attr.fg.to_crossterm()),
                    style::SetBackgroundColor(current_attr.bg.to_crossterm())
                )?;

                // Write the changed characters
                for i in start_x..x {
                    write!(stdout, "{}", self.buffer[y][i].ch)?;
                }
            }
        }

        stdout.flush()?;

        // Copy current buffer to previous buffer
        self.prev_buffer.clone_from(&self.buffer);

        Ok(())
    }

    /// Show the cursor at the specified position
    pub fn show_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(
            stdout,
            cursor::MoveTo(x, y),
            cursor::Show
        )?;
        Ok(())
    }

    /// Hide the cursor
    pub fn hide_cursor(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, cursor::Hide)?;
        Ok(())
    }

    /// Put an event in the queue for next iteration
    /// Matches Borland's TProgram::putEvent() - allows re-queuing events
    pub fn put_event(&mut self, event: Event) {
        self.pending_event = Some(event);
    }

    /// Poll for an event with timeout
    pub fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        // Check for pending event first (matches Borland's TProgram::getEvent)
        if let Some(event) = self.pending_event.take() {
            return Ok(Some(event));
        }

        if event::poll(timeout)? {
            match event::read()? {
                CTEvent::Key(key) => {
                    let key_code = self.esc_tracker.process_key(key);
                    if key_code == 0 {
                        // ESC sequence in progress, don't generate event yet
                        return Ok(None);
                    }

                    // Handle global screen dump shortcuts at the lowest level
                    if key_code == KB_F12 {
                        let _ = self.flash();
                        let _ = self.dump_screen("screen-dump.txt");
                        return Ok(None);  // Don't propagate event, it's been handled
                    }

                    // Handle active view dump shortcut (Shift+F12)
                    if key_code == KB_SHIFT_F12 {
                        let _ = self.flash();
                        if let Some(bounds) = self.active_view_bounds {
                            let _ = self.dump_region(
                                bounds.a.x as u16,
                                bounds.a.y as u16,
                                (bounds.b.x - bounds.a.x) as u16,
                                (bounds.b.y - bounds.a.y) as u16,
                                "active-view-dump.txt"
                            );
                        }
                        return Ok(None);  // Don't propagate event, it's been handled
                    }

                    // Create event preserving modifiers from original crossterm event
                    Ok(Some(Event {
                        what: EventType::Keyboard,
                        key_code,
                        key_modifiers: key.modifiers,
                        ..Event::nothing()
                    }))
                }
                CTEvent::Mouse(mouse) => {
                    Ok(self.convert_mouse_event(mouse))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Read an event (blocking)
    pub fn read_event(&mut self) -> io::Result<Event> {
        loop {
            match event::read()? {
                CTEvent::Key(key) => {
                    let key_code = self.esc_tracker.process_key(key);
                    if key_code == 0 {
                        // ESC sequence in progress, wait for next key
                        continue;
                    }

                    // Handle global screen dump shortcuts at the lowest level
                    if key_code == KB_F12 {
                        let _ = self.flash();
                        let _ = self.dump_screen("screen-dump.txt");
                        continue;  // Don't return event, it's been handled - wait for next event
                    }

                    // Handle active view dump shortcut (Shift+F12)
                    if key_code == KB_SHIFT_F12 {
                        let _ = self.flash();
                        if let Some(bounds) = self.active_view_bounds {
                            let _ = self.dump_region(
                                bounds.a.x as u16,
                                bounds.a.y as u16,
                                (bounds.b.x - bounds.a.x) as u16,
                                (bounds.b.y - bounds.a.y) as u16,
                                "active-view-dump.txt"
                            );
                        }
                        continue;  // Don't return event, it's been handled - wait for next event
                    }

                    return Ok(Event::keyboard(key_code));
                }
                CTEvent::Mouse(mouse) => {
                    if let Some(event) = self.convert_mouse_event(mouse) {
                        return Ok(event);
                    }
                }
                _ => continue,
            }
        }
    }

    /// Convert crossterm mouse event to our Event type
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
        let buttons = match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => MB_LEFT_BUTTON,
            MouseEventKind::Down(MouseButton::Right) | MouseEventKind::Drag(MouseButton::Right) => MB_RIGHT_BUTTON,
            MouseEventKind::Down(MouseButton::Middle) | MouseEventKind::Drag(MouseButton::Middle) => MB_MIDDLE_BUTTON,
            MouseEventKind::Up(_) => 0, // No buttons pressed on release
            MouseEventKind::Moved => self.last_mouse_buttons, // Maintain button state during move
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

        Some(Event::mouse(event_type, pos, buttons, is_double_click))
    }

    /// Dump the entire screen buffer to an ANSI text file for debugging
    pub fn dump_screen(&self, path: &str) -> io::Result<()> {
        ansi_dump::dump_buffer_to_file(&self.buffer, self.width as usize, self.height as usize, path)
    }

    /// Dump a rectangular region of the screen to an ANSI text file
    pub fn dump_region(&self, x: u16, y: u16, width: u16, height: u16, path: &str) -> io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        ansi_dump::dump_buffer_region(
            &mut file,
            &self.buffer,
            x as usize,
            y as usize,
            width as usize,
            height as usize,
        )
    }

    /// Get a reference to the internal buffer for custom dumping
    pub fn buffer(&self) -> &[Vec<Cell>] {
        &self.buffer
    }

    /// Flash the screen by inverting all colors briefly
    pub fn flash(&mut self) -> io::Result<()> {
        use std::thread;

        // Save current buffer
        let saved_buffer = self.buffer.clone();

        // Invert all colors
        for row in &mut self.buffer {
            for cell in row {
                // Swap foreground and background colors
                let temp_fg = cell.attr.fg;
                cell.attr.fg = cell.attr.bg;
                cell.attr.bg = temp_fg;
            }
        }

        // Flush inverted screen
        self.flush()?;

        // Wait briefly (50ms)
        thread::sleep(Duration::from_millis(50));

        // Restore original buffer
        self.buffer = saved_buffer;

        // Flush restored screen
        self.flush()?;

        Ok(())
    }

    /// Emit a terminal beep (bell) sound
    /// Matches Borland: TScreen::makeBeep() which calls beep() + refresh()
    /// Outputs the terminal bell character and flushes immediately
    pub fn beep(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        write!(stdout, "\x07")?;  // Terminal bell character
        stdout.flush()?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
