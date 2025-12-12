// (C) 2025 - Enzo Lombardi

//! SSH-based backend implementation.
//!
//! This module provides the [`SshBackend`] which implements the [`Backend`]
//! trait for SSH channels. This allows turbo-vision applications to be
//! served over SSH connections.
//!
//! # Architecture
//!
//! The SSH backend uses channels to communicate between the async SSH handler
//! and the synchronous turbo-vision event loop:
//!
//! ```text
//! ┌──────────────────┐          ┌──────────────────┐
//! │   SSH Handler    │          │   SshBackend     │
//! │   (async)        │          │   (sync)         │
//! ├──────────────────┤          ├──────────────────┤
//! │                  │  events  │                  │
//! │  InputParser ────┼─────────▶│  event_rx        │
//! │                  │          │                  │
//! │                  │  output  │                  │
//! │  SSH channel ◀───┼──────────┤  output_tx       │
//! │                  │          │                  │
//! │  PTY size ───────┼─────────▶│  size (shared)   │
//! └──────────────────┘          └──────────────────┘
//! ```

use std::io;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use tokio::sync::mpsc;

use super::backend::{Backend, Capabilities};
use super::input_parser::InputParser;
use crate::core::event::Event;

/// SSH backend for turbo-vision applications.
///
/// This backend communicates with an SSH handler through channels,
/// allowing the TUI to run over an SSH connection.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use parking_lot::Mutex;
/// use tokio::sync::mpsc;
/// use turbo_vision::terminal::{SshBackend, Terminal};
///
/// // Create channels for communication
/// let (event_tx, event_rx) = mpsc::unbounded_channel();
/// let (output_tx, output_rx) = mpsc::unbounded_channel();
/// let size = Arc::new(Mutex::new((80u16, 24u16)));
///
/// // Create backend
/// let backend = SshBackend::new(output_tx, event_rx, Arc::clone(&size));
///
/// // Use with Terminal
/// let terminal = Terminal::with_backend(Box::new(backend)).unwrap();
/// ```
pub struct SshBackend {
    output_buffer: Vec<u8>,
    output_tx: mpsc::UnboundedSender<Vec<u8>>,
    event_rx: mpsc::UnboundedReceiver<Event>,
    event_queue: Vec<Event>,
    size: Arc<Mutex<(u16, u16)>>,
    capabilities: Capabilities,
    initialized: bool,
}

impl SshBackend {
    /// Create a new SSH backend.
    ///
    /// # Arguments
    ///
    /// * `output_tx` - Channel for sending output to the SSH client.
    /// * `event_rx` - Channel for receiving events from the SSH handler.
    /// * `size` - Shared terminal size, updated by the SSH handler on resize.
    pub fn new(
        output_tx: mpsc::UnboundedSender<Vec<u8>>,
        event_rx: mpsc::UnboundedReceiver<Event>,
        size: Arc<Mutex<(u16, u16)>>,
    ) -> Self {
        Self {
            output_buffer: Vec::with_capacity(8192),
            output_tx,
            event_rx,
            event_queue: Vec::new(),
            size,
            capabilities: Capabilities {
                mouse: true,
                colors_256: true,
                true_color: false, // Conservative default for SSH clients
                bracketed_paste: false,
                focus_events: false,
                kitty_keyboard: false,
            },
            initialized: false,
        }
    }

    /// Get a clone of the size handle for the SSH handler.
    pub fn size_handle(&self) -> Arc<Mutex<(u16, u16)>> {
        Arc::clone(&self.size)
    }

    /// Set terminal capabilities based on client negotiation.
    pub fn set_capabilities(&mut self, caps: Capabilities) {
        self.capabilities = caps;
    }

    /// Send buffered output to the SSH channel.
    fn send_output(&mut self) -> io::Result<()> {
        if !self.output_buffer.is_empty() {
            let data = std::mem::take(&mut self.output_buffer);
            self.output_tx
                .send(data)
                .map_err(|_| io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "SSH channel closed"
                ))?;
        }
        Ok(())
    }
}

impl Backend for SshBackend {
    fn init(&mut self) -> io::Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Enter alternate screen
        self.output_buffer.extend_from_slice(b"\x1b[?1049h");
        // Enable mouse tracking (X10 compatible)
        self.output_buffer.extend_from_slice(b"\x1b[?1000h");
        // Enable SGR mouse mode for better coordinate support
        self.output_buffer.extend_from_slice(b"\x1b[?1006h");
        // Enable mouse motion events while button pressed
        self.output_buffer.extend_from_slice(b"\x1b[?1002h");
        // Hide cursor
        self.output_buffer.extend_from_slice(b"\x1b[?25l");
        // Disable line wrapping
        self.output_buffer.extend_from_slice(b"\x1b[?7l");

        self.send_output()?;
        self.initialized = true;
        Ok(())
    }

    fn cleanup(&mut self) -> io::Result<()> {
        if !self.initialized {
            return Ok(());
        }

        // Show cursor
        self.output_buffer.extend_from_slice(b"\x1b[?25h");
        // Re-enable line wrapping
        self.output_buffer.extend_from_slice(b"\x1b[?7h");
        // Disable mouse motion events
        self.output_buffer.extend_from_slice(b"\x1b[?1002l");
        // Disable SGR mouse mode
        self.output_buffer.extend_from_slice(b"\x1b[?1006l");
        // Disable mouse tracking
        self.output_buffer.extend_from_slice(b"\x1b[?1000l");
        // Leave alternate screen
        self.output_buffer.extend_from_slice(b"\x1b[?1049l");
        // Reset attributes
        self.output_buffer.extend_from_slice(b"\x1b[0m");

        self.send_output()?;
        self.initialized = false;
        Ok(())
    }

    fn size(&self) -> io::Result<(u16, u16)> {
        Ok(*self.size.lock())
    }

    fn poll_event(&mut self, _timeout: Duration) -> io::Result<Option<Event>> {
        // Return queued events first
        if let Some(ev) = self.event_queue.pop() {
            return Ok(Some(ev));
        }

        // Try to receive from the channel (non-blocking)
        match self.event_rx.try_recv() {
            Ok(ev) => Ok(Some(ev)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "SSH channel disconnected"))
            }
        }
    }

    fn write_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.output_buffer.extend_from_slice(data);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.send_output()
    }

    fn show_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        use std::io::Write;
        write!(self.output_buffer, "\x1b[{};{}H\x1b[?25h", y + 1, x + 1)?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.output_buffer.extend_from_slice(b"\x1b[?25l");
        Ok(())
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn suspend(&mut self) -> io::Result<()> {
        // SSH sessions don't support traditional suspend
        // Just ignore the request
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        // SSH sessions don't support traditional suspend
        // Just ignore the request
        Ok(())
    }

    fn cell_aspect_ratio(&self) -> (i16, i16) {
        // SSH clients may have varying aspect ratios
        // Use conservative 2:1 default
        (2, 1)
    }

    fn bell(&mut self) -> io::Result<()> {
        self.output_buffer.push(0x07);
        self.send_output()
    }

    fn clear_screen(&mut self) -> io::Result<()> {
        self.output_buffer.extend_from_slice(b"\x1b[2J\x1b[H");
        self.send_output()
    }
}

/// Builder for SSH session components.
///
/// Creates the channels and shared state needed for an SSH TUI session.
pub struct SshSessionBuilder {
    width: u16,
    height: u16,
}

impl SshSessionBuilder {
    /// Create a new session builder with default terminal size.
    pub fn new() -> Self {
        Self {
            width: 80,
            height: 24,
        }
    }

    /// Set the initial terminal size.
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Build the session components.
    ///
    /// Returns a tuple of:
    /// - `SshBackend` - For use with `Terminal::with_backend()`
    /// - `SshSessionHandle` - For use by the SSH handler
    pub fn build(self) -> (SshBackend, SshSessionHandle) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        let size = Arc::new(Mutex::new((self.width, self.height)));

        let backend = SshBackend::new(output_tx, event_rx, Arc::clone(&size));
        let handle = SshSessionHandle {
            event_tx,
            output_rx,
            size,
            input_parser: InputParser::new(),
        };

        (backend, handle)
    }
}

impl Default for SshSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for the SSH handler to communicate with the TUI.
///
/// This is the counterpart to `SshBackend`, used by the SSH handler
/// to send events and receive output.
pub struct SshSessionHandle {
    /// Send events to the TUI.
    pub event_tx: mpsc::UnboundedSender<Event>,
    /// Receive output from the TUI.
    pub output_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    /// Shared terminal size.
    pub size: Arc<Mutex<(u16, u16)>>,
    /// Input parser for converting raw bytes to events.
    pub input_parser: InputParser,
}

impl SshSessionHandle {
    /// Update the terminal size.
    ///
    /// This should be called when the SSH client sends a window change request.
    pub fn resize(&mut self, width: u16, height: u16) {
        *self.size.lock() = (width, height);

        // Send resize event to the TUI
        let event = Event::mouse(
            crate::core::event::EventType::Nothing,
            crate::core::geometry::Point::zero(),
            0,
            false,
        );
        // Note: turbo-vision handles resize through polling size(), not events
        let _ = self.event_tx.send(event);
    }

    /// Process raw input bytes from the SSH client.
    ///
    /// Parses the bytes into events and sends them to the TUI.
    pub fn process_input(&mut self, data: &[u8]) {
        let events = self.input_parser.parse(data);
        for event in events {
            let _ = self.event_tx.send(event);
        }
    }

    /// Try to receive output for the SSH client.
    ///
    /// Returns `None` if no output is available.
    pub fn try_recv_output(&mut self) -> Option<Vec<u8>> {
        self.output_rx.try_recv().ok()
    }

    /// Check if the TUI has disconnected.
    pub fn is_disconnected(&self) -> bool {
        self.event_tx.is_closed()
    }
}
