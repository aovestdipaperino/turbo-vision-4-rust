// (C) 2025 - Enzo Lombardi

//! Terminal abstraction layer for turbo-vision.
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
//! # Backend Architecture
//!
//! The terminal uses a [`Backend`] trait to abstract low-level I/O operations,
//! allowing turbo-vision to work with different terminal transports:
//!
//! - [`CrosstermBackend`] - Local terminal via crossterm (default)
//! - `SshBackend` - Remote terminal via SSH (requires `ssh` feature)
//!
//! # Examples
//!
//! Basic terminal usage:
//!
//! ```rust,no_run
//! use turbo_vision::terminal::Terminal;
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
//!
//! Using a custom backend:
//!
//! ```rust,no_run
//! use turbo_vision::terminal::{Terminal, CrosstermBackend};
//! use turbo_vision::core::error::Result;
//!
//! fn main() -> Result<()> {
//!     let backend = CrosstermBackend::new()?;
//!     let mut terminal = Terminal::with_backend(Box::new(backend))?;
//!     // ...
//!     terminal.shutdown()?;
//!     Ok(())
//! }
//! ```

mod backend;
mod crossterm_backend;
pub mod remote_input;

#[cfg(feature = "ssh")]
mod input_parser;
#[cfg(feature = "ssh")]
mod ssh_backend;

pub use backend::{Backend, Capabilities};
pub use crossterm_backend::CrosstermBackend;

#[cfg(feature = "ssh")]
pub use input_parser::InputParser;
#[cfg(feature = "ssh")]
pub use ssh_backend::{SshBackend, SshSessionBuilder, SshSessionHandle};

use crate::core::ansi_dump;
use crate::core::draw::Cell;
use crate::core::error::Result;
use crate::core::event::Event;
use crate::core::geometry::{Point, Rect};
use crate::core::palette::Attr;
use std::io::{self, Write};
use std::sync::mpsc::Receiver;
use std::time::Duration;

/// Build the full SGR escape for a cell run: a reset, truecolor fg/bg, then any
/// style flags. The leading `0` reset prevents a previous run's style from
/// bleeding, since each run re-emits absolute colors.
/// Style order: bold(1), dim(2), italic(3), underline(4), reverse(7), strikethrough(9).
fn attr_to_sgr(attr: Attr) -> String {
    use crate::core::palette::Style;
    let (fg_r, fg_g, fg_b) = attr.fg.to_rgb();
    let (bg_r, bg_g, bg_b) = attr.bg.to_rgb();
    let mut s = format!(
        "\x1b[0;38;2;{};{};{};48;2;{};{};{}",
        fg_r, fg_g, fg_b, bg_r, bg_g, bg_b
    );
    if attr.style.contains(Style::BOLD) { s.push_str(";1"); }
    if attr.style.contains(Style::DIM) { s.push_str(";2"); }
    if attr.style.contains(Style::ITALIC) { s.push_str(";3"); }
    if attr.style.contains(Style::UNDERLINE) { s.push_str(";4"); }
    if attr.style.contains(Style::REVERSE) { s.push_str(";7"); }
    if attr.style.contains(Style::STRIKETHROUGH) { s.push_str(";9"); }
    s.push('m');
    s
}

/// Terminal abstraction for rendering and input handling.
///
/// The Terminal provides a high-level interface for TUI applications,
/// managing double-buffered rendering, clipping regions, and event handling.
/// Low-level I/O is delegated to a [`Backend`] implementation.
pub struct Terminal {
    backend: Box<dyn Backend>,
    buffer: Vec<Vec<Cell>>,
    prev_buffer: Vec<Vec<Cell>>,
    width: u16,
    height: u16,
    clip_stack: Vec<Rect>,
    pending_event: Option<Event>,
    /// Receiver for keyboard events injected by the remote-input listener.
    /// `None` unless [`enable_remote_input`](Self::enable_remote_input) was called.
    injected_rx: Option<Receiver<Event>>,
}

impl Terminal {
    /// Initializes a new terminal instance using the default crossterm backend.
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
    /// # Examples
    ///
    /// ```rust,no_run
    /// use turbo_vision::terminal::Terminal;
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
        let backend = CrosstermBackend::new()?;
        Self::with_backend(Box::new(backend))
    }

    /// Initializes a new terminal instance with a custom backend.
    ///
    /// This allows using alternative backends such as SSH for remote
    /// terminal access.
    ///
    /// # Arguments
    ///
    /// * `backend` - The backend implementation to use for I/O.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use turbo_vision::terminal::{Terminal, CrosstermBackend};
    /// use turbo_vision::core::error::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let backend = CrosstermBackend::new()?;
    ///     let mut terminal = Terminal::with_backend(Box::new(backend))?;
    ///     terminal.shutdown()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn with_backend(mut backend: Box<dyn Backend>) -> Result<Self> {
        backend.init()?;

        let (width, height) = backend.size()?;
        let empty_cell = Cell::new(' ', Attr::from_u8(0x07));
        let buffer = vec![vec![empty_cell; width as usize]; height as usize];
        let prev_buffer = vec![vec![empty_cell; width as usize]; height as usize];

        Ok(Self {
            backend,
            buffer,
            prev_buffer,
            width,
            height,
            clip_stack: Vec::new(),
            pending_event: None,
            injected_rx: None,
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
    /// # use turbo_vision::terminal::Terminal;
    /// # use turbo_vision::core::error::Result;
    /// # fn main() -> Result<()> {
    /// let mut terminal = Terminal::init()?;
    /// // Use terminal...
    /// terminal.shutdown()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn shutdown(&mut self) -> Result<()> {
        self.backend.cleanup()?;
        Ok(())
    }

    /// Suspend the terminal (for Ctrl+Z handling).
    ///
    /// Restores terminal to normal mode while keeping the Terminal struct alive.
    /// Call [`resume()`](Self::resume) to return to TUI mode.
    pub fn suspend(&mut self) -> Result<()> {
        self.backend.suspend()?;
        Ok(())
    }

    /// Resume the terminal after suspension.
    ///
    /// Re-initializes terminal state and forces full screen redraw.
    pub fn resume(&mut self) -> Result<()> {
        self.backend.resume()?;

        // Force full screen redraw by clearing prev_buffer
        let empty_cell = Cell::new(' ', Attr::from_u8(0x07));
        for row in &mut self.prev_buffer {
            for cell in row {
                *cell = empty_cell;
            }
        }

        Ok(())
    }

    /// Get terminal size.
    pub fn size(&self) -> (i16, i16) {
        (self.width as i16, self.height as i16)
    }

    /// Query actual terminal size from the system.
    ///
    /// This is useful for detecting manual resizes. Prefer
    /// [`backend_size`](Self::backend_size) when a Terminal instance is
    /// available: this static version always asks crossterm, which is wrong
    /// for non-local backends (e.g. SSH sessions).
    pub fn query_size() -> io::Result<(i16, i16)> {
        let (width, height) = crossterm::terminal::size()?;
        Ok((width as i16, height as i16))
    }

    /// Query the size reported by this terminal's backend.
    ///
    /// Unlike [`query_size`](Self::query_size), this respects the active
    /// backend, so SSH sessions report their client's window size.
    pub fn backend_size(&self) -> io::Result<(i16, i16)> {
        let (width, height) = self.backend.size()?;
        Ok((width as i16, height as i16))
    }

    /// Query terminal cell aspect ratio for shadow proportions (static version).
    ///
    /// Returns `(horizontal, vertical)` shadow multipliers to make shadows
    /// appear visually proportional. This static version can be called before
    /// a Terminal instance is created.
    pub fn query_cell_aspect_ratio() -> (i16, i16) {
        use crossterm::terminal::window_size;

        if let Ok(ws) = window_size() {
            if ws.width > 0 && ws.height > 0 && ws.columns > 0 && ws.rows > 0 {
                let cell_width = ws.width as f32 / ws.columns as f32;
                let cell_height = ws.height as f32 / ws.rows as f32;

                if cell_width > 0.0 {
                    let ratio = (cell_height / cell_width).round() as i16;
                    return (ratio.max(1), 1);
                }
            }
        }
        // Fallback: typical terminal fonts are ~10x16 pixels (1.6:1 ratio)
        (2, 1)
    }

    /// Query terminal cell aspect ratio for shadow proportions (instance version).
    ///
    /// Returns `(horizontal, vertical)` shadow multipliers to make shadows
    /// appear visually proportional.
    pub fn cell_aspect_ratio(&self) -> (i16, i16) {
        self.backend.cell_aspect_ratio()
    }

    /// Resize the terminal buffers.
    ///
    /// Recreates buffers and forces a complete redraw.
    pub fn resize(&mut self, new_width: u16, new_height: u16) {
        self.width = new_width;
        self.height = new_height;

        // Recreate buffers with new size
        let empty_cell = Cell::new(' ', Attr::from_u8(0x07));
        self.buffer = vec![vec![empty_cell; new_width as usize]; new_height as usize];

        // Use a different cell for prev_buffer to force complete redraw
        let force_redraw_cell = Cell::new('\0', Attr::from_u8(0xFF));
        self.prev_buffer = vec![vec![force_redraw_cell; new_width as usize]; new_height as usize];

        // Clear the screen
        let _ = self.backend.clear_screen();
    }

    /// Set the ESC timeout in milliseconds.
    ///
    /// This controls how long the terminal waits after ESC to detect
    /// ESC+letter sequences.
    pub fn set_esc_timeout(&mut self, timeout_ms: u64) {
        // Only the crossterm backend has an ESC-disambiguation timeout;
        // other backends ignore the setting
        if let Some(ct_backend) = self.backend_as_crossterm_mut() {
            ct_backend.set_esc_timeout(timeout_ms);
        }
    }

    /// Get a mutable reference to the backend as CrosstermBackend, if applicable.
    fn backend_as_crossterm_mut(&mut self) -> Option<&mut CrosstermBackend> {
        self.backend.as_any_mut().downcast_mut::<CrosstermBackend>()
    }

    /// Force a full screen redraw on the next flush.
    ///
    /// This clears the internal prev_buffer, forcing all cells to be resent
    /// to the terminal on the next [`flush()`](Self::flush) call.
    pub fn force_full_redraw(&mut self) {
        // Use a cell that will never match any real content, forcing every
        // cell to be resent on the next flush. Must NOT use 0x07 (the default
        // empty cell) because views that fill with LightGray-on-Black spaces
        // would match and be silently skipped by the diff.
        let force_cell = Cell::new('\0', Attr::from_u8(0xFF));
        for row in &mut self.prev_buffer {
            for cell in row {
                *cell = force_cell;
            }
        }
    }

    /// Push a clipping region onto the stack.
    pub fn push_clip(&mut self, rect: Rect) {
        self.clip_stack.push(rect);
    }

    /// Pop a clipping region from the stack.
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Get the current effective clipping region (intersection of all regions on stack).
    fn get_clip_rect(&self) -> Option<Rect> {
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

    /// Check if a point is within the current clipping region.
    fn is_clipped(&self, x: i16, y: i16) -> bool {
        if let Some(clip) = self.get_clip_rect() {
            !clip.contains(Point::new(x, y))
        } else {
            false
        }
    }

    /// Write a cell at the given position.
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

    /// Write a line from a draw buffer.
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

    /// Read a cell from the buffer at the given position.
    ///
    /// Returns `None` if coordinates are out of bounds.
    pub fn read_cell(&self, x: i16, y: i16) -> Option<Cell> {
        if x < 0 || y < 0 || x >= self.width as i16 || y >= self.height as i16 {
            return None;
        }
        Some(self.buffer[y as usize][x as usize])
    }

    /// Clear the entire screen buffer.
    pub fn clear(&mut self) {
        let empty_cell = Cell::new(' ', Attr::from_u8(0x07));
        for row in &mut self.buffer {
            for cell in row {
                *cell = empty_cell;
            }
        }
    }

    /// Flush changes to the terminal.
    ///
    /// This performs differential rendering, only sending changed cells
    /// to the terminal for optimal performance.
    pub fn flush(&mut self) -> io::Result<()> {
        // Build output in a buffer, then send through backend
        let mut output = Vec::new();

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

                // Move cursor: ESC[row;colH (1-indexed)
                write!(output, "\x1b[{};{}H", y + 1, start_x + 1)?;

                // Full SGR for this run (reset + truecolor + style).
                output.extend_from_slice(attr_to_sgr(current_attr).as_bytes());

                // Write the changed characters
                for i in start_x..x {
                    let ch = self.buffer[y][i].ch;
                    // Skip zero-width padding cells (placed after wide characters)
                    if ch == '\0' {
                        continue;
                    }
                    // Encode character as UTF-8
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    output.extend_from_slice(encoded.as_bytes());
                }
            }
        }

        // Send through backend
        if !output.is_empty() {
            self.backend.write_raw(&output)?;
        }
        self.backend.flush()?;

        // Copy current buffer to previous buffer
        self.prev_buffer.clone_from(&self.buffer);

        Ok(())
    }

    /// Show the cursor at the specified position.
    pub fn show_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        self.backend.show_cursor(x, y)
    }

    /// Hide the cursor.
    pub fn hide_cursor(&mut self) -> io::Result<()> {
        self.backend.hide_cursor()
    }

    /// Put an event in the queue for next iteration.
    ///
    /// This allows re-queuing events, matching Borland's `TProgram::putEvent()`.
    pub fn put_event(&mut self, event: Event) {
        self.pending_event = Some(event);
    }

    /// Enable the remote keyboard-input listener on the given TCP port.
    ///
    /// This is **off by default**. Once enabled, the terminal listens on
    /// `127.0.0.1:port` and injects key chords received over the socket into the
    /// event stream returned by [`poll_event`](Self::poll_event), as if they had
    /// been typed. See [`remote_input`] for the wire format. Intended for
    /// testing and automation (for example, triggering Ctrl+F12 screenshots on
    /// terminals that do not forward that chord).
    ///
    /// # Errors
    ///
    /// Returns an error if the port cannot be bound.
    pub fn enable_remote_input(&mut self, port: u16) -> io::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        remote_input::spawn(port, tx)?;
        self.injected_rx = Some(rx);
        Ok(())
    }

    /// Poll for an event with timeout.
    pub fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<Event>> {
        // Check for pending event first
        if let Some(event) = self.pending_event.take() {
            return Ok(Some(event));
        }

        // Then any event injected by the remote-input listener.
        if let Some(rx) = &self.injected_rx {
            if let Ok(event) = rx.try_recv() {
                return Ok(Some(event));
            }
        }

        self.backend.poll_event(timeout)
    }

    /// Read an event (blocking).
    pub fn read_event(&mut self) -> io::Result<Event> {
        loop {
            if let Some(event) = self.poll_event(Duration::from_secs(60))? {
                return Ok(event);
            }
        }
    }

    /// Query the pixel size of a single character cell ("the current font").
    ///
    /// Returns `(cell_width_px, cell_height_px)` derived from the terminal's
    /// reported window size, or `None` if the terminal does not report pixel
    /// dimensions. This is used to render screenshots at the same resolution
    /// the font is actually displayed at.
    pub fn query_font_pixel_size() -> Option<(u16, u16)> {
        use crossterm::terminal::window_size;

        if let Ok(ws) = window_size() {
            if ws.columns > 0 && ws.rows > 0 && ws.width > 0 && ws.height > 0 {
                return Some((ws.width / ws.columns, ws.height / ws.rows));
            }
        }
        None
    }

    /// Render the current screen buffer to a PNG file.
    ///
    /// The image is built from the embedded 8x16 bitmap font using an *integer*
    /// magnification, so glyphs stay crisp and keep the font's proportions. The
    /// scale is derived from the current font cell height reported by the
    /// terminal (so a larger on-screen font yields a larger image); it falls
    /// back to 1x when the terminal does not report pixel sizes. See
    /// [`crate::core::screenshot`] for rendering details.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written.
    pub fn save_screenshot_png(&self, path: &str) -> io::Result<()> {
        use crate::core::screenshot::{self, GLYPH_HEIGHT};
        // Pick the integer scale whose glyph height is closest to the real cell
        // height (rounding), clamped to a sane range.
        let scale = match Self::query_font_pixel_size() {
            Some((_, ch)) if ch > 0 => {
                ((ch as usize + GLYPH_HEIGHT / 2) / GLYPH_HEIGHT).clamp(1, 8)
            }
            _ => 1,
        };
        screenshot::render_to_png(
            &self.buffer,
            self.width as usize,
            self.height as usize,
            scale,
            std::path::Path::new(path),
        )
    }

    /// Dump the entire screen buffer to an ANSI text file for debugging.
    pub fn dump_screen(&self, path: &str) -> io::Result<()> {
        ansi_dump::dump_buffer_to_file(
            &self.buffer,
            self.width as usize,
            self.height as usize,
            path,
        )
    }

    /// Dump a rectangular region of the screen to an ANSI text file.
    pub fn dump_region(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        path: &str,
    ) -> io::Result<()> {
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

    /// Get a reference to the internal buffer for custom dumping.
    pub fn buffer(&self) -> &[Vec<Cell>] {
        &self.buffer
    }

    /// Flash the screen by inverting all colors briefly.
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

    /// Emit a terminal beep (bell) sound.
    pub fn beep(&mut self) -> io::Result<()> {
        self.backend.bell()
    }

    /// Get terminal capabilities.
    pub fn capabilities(&self) -> Capabilities {
        self.backend.capabilities()
    }

    /// Write raw Kitty graphics protocol data to the terminal.
    ///
    /// This method sends data directly to the terminal without buffering,
    /// as Kitty graphics protocol commands need to be sent immediately
    /// and bypass the normal double-buffered rendering.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes containing Kitty graphics protocol escape sequences.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the terminal fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Send a Kitty graphics command to display an image
    /// terminal.write_kitty_graphics(b"\x1b_Ga=T,f=100;...\x1b\\")?;
    /// ```
    pub fn write_kitty_graphics(&mut self, data: &[u8]) -> io::Result<()> {
        self.backend.write_raw(data)?;
        self.backend.flush()
    }

    /// Check if the terminal supports Kitty graphics protocol.
    ///
    /// This is a heuristic check based on the `TERM` environment variable
    /// and known terminal capabilities. Returns `true` for terminals known
    /// to support Kitty graphics (kitty, wezterm, ghostty, etc.).
    pub fn supports_kitty_graphics(&self) -> bool {
        // Check TERM environment variable for known Kitty-compatible terminals
        if let Ok(term) = std::env::var("TERM") {
            let term_lower = term.to_lowercase();
            if term_lower.contains("kitty")
                || term_lower.contains("wezterm")
                || term_lower.contains("ghostty")
            {
                return true;
            }
        }

        // Check TERM_PROGRAM for terminals that set it
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            let prog_lower = term_program.to_lowercase();
            if prog_lower.contains("kitty")
                || prog_lower.contains("wezterm")
                || prog_lower.contains("ghostty")
            {
                return true;
            }
        }

        // Check for KITTY_WINDOW_ID (set by Kitty terminal)
        if std::env::var("KITTY_WINDOW_ID").is_ok() {
            return true;
        }

        false
    }

    /// Delete a Kitty graphics image by ID.
    ///
    /// Sends a command to remove a previously transmitted image from the
    /// terminal's memory.
    ///
    /// # Arguments
    ///
    /// * `image_id` - The ID of the image to delete.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the terminal fails.
    pub fn delete_kitty_image(&mut self, image_id: u32) -> io::Result<()> {
        let cmd = format!("\x1b_Ga=d,d=I,i={},q=2;\x1b\\", image_id);
        self.write_kitty_graphics(cmd.as_bytes())
    }

    /// Clear all Kitty graphics images from the terminal.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the terminal fails.
    pub fn clear_kitty_images(&mut self) -> io::Result<()> {
        // Delete all images: a=d,d=A (delete all)
        self.write_kitty_graphics(b"\x1b_Ga=d,d=A,q=2;\x1b\\")
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attr_to_sgr_bold_and_reset() {
        use crate::core::palette::{Attr, TvColor};
        let bold = Attr::new(TvColor::White, TvColor::Black).bold();
        let plain = Attr::new(TvColor::White, TvColor::Black);

        let s = attr_to_sgr(bold);
        assert!(s.starts_with("\x1b[0;38;2;"), "leading reset + truecolor fg");
        assert!(s.contains(";48;2;"), "truecolor bg present");
        assert!(s.ends_with(";1m"), "bold code appended: {s:?}");

        // Plain attr emits no style codes, but still carries the leading reset so a
        // previous run's style does not bleed into it.
        let p = attr_to_sgr(plain);
        assert!(p.starts_with("\x1b[0;38;2;"));
        assert!(!p.contains(";1"), "no bold code for plain attr");
        assert!(p.ends_with("m"));
    }

    #[test]
    fn test_attr_to_sgr_multiple_styles_ordered() {
        use crate::core::palette::{Attr, TvColor, Style};
        let a = Attr::new(TvColor::White, TvColor::Blue)
            .with_style(Style::ITALIC | Style::UNDERLINE);
        let s = attr_to_sgr(a);
        // italic(3) before underline(4)
        let i3 = s.find(";3").unwrap();
        let i4 = s.find(";4").unwrap();
        assert!(i3 < i4, "style codes emitted in canonical order: {s:?}");
    }
}
