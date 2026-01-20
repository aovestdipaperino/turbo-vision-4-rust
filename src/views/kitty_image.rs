// (C) 2025 - Enzo Lombardi

//! Kitty image view - displays images using the Kitty graphics protocol.
//!
//! This view renders images in terminals that support the Kitty graphics protocol,
//! such as Kitty, WezTerm, and Ghostty. The image is displayed at the specified
//! position and can be scaled to fit the view bounds.
//!
//! # Kitty Graphics Protocol
//!
//! The Kitty graphics protocol transmits images as base64-encoded PNG data via
//! escape sequences. This allows terminals to display actual graphics inline
//! with text content.
//!
//! # Example
//!
//! ```rust,no_run
//! use turbo_vision::views::kitty_image::KittyImage;
//! use turbo_vision::core::geometry::Rect;
//!
//! // Load from file
//! let image = KittyImage::from_file(
//!     Rect::new(5, 2, 45, 22),
//!     "logo.png",
//! ).expect("Failed to load image");
//!
//! // Or from bytes
//! let png_data = std::fs::read("logo.png").unwrap();
//! let image = KittyImage::from_bytes(
//!     Rect::new(5, 2, 45, 22),
//!     png_data,
//! );
//! ```

use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::Attr;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

/// Global image ID counter for Kitty protocol
static IMAGE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

/// A view that displays an image using the Kitty graphics protocol.
///
/// This view is designed for terminals that support the Kitty graphics protocol.
/// The image is transmitted to the terminal and displayed at the view's position.
/// The view clears its area with a background color when the terminal doesn't
/// support Kitty graphics.
pub struct KittyImage {
    bounds: Rect,
    state: StateFlags,
    /// The PNG image data (raw bytes)
    png_data: Vec<u8>,
    /// Unique image ID for this image (used by Kitty protocol)
    image_id: u32,
    /// Whether the image has been transmitted to the terminal
    transmitted: bool,
    /// Background attribute for areas not covered by the image
    background_attr: Attr,
    /// Number of columns the image should span (0 = auto based on bounds)
    columns: u16,
    /// Number of rows the image should span (0 = auto based on bounds)
    rows: u16,
    /// Owner pointer for palette chain
    owner: Option<*const dyn View>,
    /// Z-index for layering (higher = on top)
    z_index: i32,
    /// Last drawn bounds (for clearing old placements when moved/resized)
    last_bounds: Option<Rect>,
}

impl KittyImage {
    /// Creates a new Kitty image view from PNG data bytes.
    ///
    /// # Arguments
    /// * `bounds` - The bounding rectangle for the view
    /// * `png_data` - Raw PNG image data
    pub fn from_bytes(bounds: Rect, png_data: Vec<u8>) -> Self {
        Self {
            bounds,
            state: 0,
            png_data,
            image_id: IMAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            transmitted: false,
            background_attr: Attr::new(
                crate::core::palette::TvColor::Black,
                crate::core::palette::TvColor::Black,
            ),
            columns: 0,
            rows: 0,
            owner: None,
            z_index: 0,
            last_bounds: None,
        }
    }

    /// Creates a new Kitty image view by loading from a file.
    ///
    /// # Arguments
    /// * `bounds` - The bounding rectangle for the view
    /// * `path` - Path to the PNG image file
    ///
    /// # Errors
    /// Returns an IO error if the file cannot be read.
    pub fn from_file<P: AsRef<Path>>(bounds: Rect, path: P) -> io::Result<Self> {
        let png_data = std::fs::read(path)?;
        Ok(Self::from_bytes(bounds, png_data))
    }

    /// Sets the background attribute for areas not covered by the image.
    #[must_use]
    pub fn background(mut self, attr: Attr) -> Self {
        self.background_attr = attr;
        self
    }

    /// Sets the number of columns the image should span.
    /// If 0, uses the view width.
    #[must_use]
    pub fn columns(mut self, cols: u16) -> Self {
        self.columns = cols;
        self
    }

    /// Sets the number of rows the image should span.
    /// If 0, uses the view height.
    #[must_use]
    pub fn rows(mut self, rows: u16) -> Self {
        self.rows = rows;
        self
    }

    /// Sets the Z-index for layering.
    #[must_use]
    pub fn z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }

    /// Gets the image ID used by the Kitty protocol.
    pub fn image_id(&self) -> u32 {
        self.image_id
    }

    /// Replaces the image data with new PNG data.
    pub fn set_image(&mut self, png_data: Vec<u8>) {
        self.png_data = png_data;
        self.transmitted = false;
        self.image_id = IMAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    }

    /// Loads a new image from a file.
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let png_data = std::fs::read(path)?;
        self.set_image(png_data);
        Ok(())
    }

    /// Marks the image as needing retransmission.
    pub fn invalidate(&mut self) {
        self.transmitted = false;
    }

    /// Builds the Kitty graphics protocol escape sequence for transmitting an image.
    ///
    /// The Kitty graphics protocol format:
    /// `ESC_G<control data>;<payload>ESC\`
    ///
    /// Control keys used:
    /// - `a=t` - action: transmit only (lowercase, doesn't display)
    /// - `f=100` - format: PNG
    /// - `t=d` - transmission: direct (data in payload)
    /// - `i=<id>` - image ID for later reference
    /// - `q=2` - quiet mode (suppress responses)
    /// - `m=0/1` - more data follows (for chunked transfer)
    fn build_transmit_sequence(&self) -> Vec<u8> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let encoded = STANDARD.encode(&self.png_data);
        let mut result = Vec::new();

        // For large images, we need to chunk the data
        // Kitty protocol recommends chunks of 4096 bytes
        const CHUNK_SIZE: usize = 4096;

        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect();

        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == chunks.len() - 1;
            let is_first = i == 0;

            // Start escape sequence
            result.extend_from_slice(b"\x1b_G");

            // m=0 for last chunk, m=1 for more data
            let more_flag = i32::from(!is_last);

            if is_first {
                // First chunk includes all the metadata
                // Use a=t (lowercase) for transmit-only, don't display yet
                result.extend_from_slice(
                    format!(
                        "a=t,f=100,t=d,i={},q=2,m={}",
                        self.image_id,
                        more_flag
                    ).as_bytes()
                );
            } else {
                // Continuation chunks only need m flag
                result.extend_from_slice(
                    format!("m={}", more_flag).as_bytes()
                );
            }

            // Add payload separator and data
            result.push(b';');
            result.extend_from_slice(chunk.as_bytes());

            // End escape sequence
            result.extend_from_slice(b"\x1b\\");
        }

        result
    }

    /// Builds the Kitty graphics protocol escape sequence for displaying a previously
    /// transmitted image at a specific position.
    fn build_display_sequence(&self, x: u16, y: u16, cols: u16, rows: u16) -> Vec<u8> {
        // First, move cursor to position
        let mut result = format!("\x1b[{};{}H", y + 1, x + 1).into_bytes();

        // Then display the image
        // ESC_Ga=p,i=<id>,c=<cols>,r=<rows>,z=<z_index>,q=2;ESC\
        // z parameter: negative values place image behind text, positive in front
        result.extend_from_slice(b"\x1b_G");
        result.extend_from_slice(
            format!("a=p,i={},c={},r={},z={},q=2", self.image_id, cols, rows, self.z_index).as_bytes()
        );
        result.extend_from_slice(b";\x1b\\");

        result
    }

    /// Builds the Kitty graphics protocol escape sequence for deleting an image
    /// and all its placements.
    #[allow(dead_code, reason = "Reserved for future use")]
    fn build_delete_sequence(&self) -> Vec<u8> {
        // ESC_Ga=d,d=I,i=<id>,q=2;ESC\
        // d=I deletes the image data and all placements
        format!("\x1b_Ga=d,d=I,i={},q=2;\x1b\\", self.image_id).into_bytes()
    }

    /// Builds the Kitty graphics protocol escape sequence for deleting all
    /// placements of this image (keeps image data for re-placement).
    fn build_delete_placements_sequence(&self) -> Vec<u8> {
        // ESC_Ga=d,d=a,i=<id>,q=2;ESC\
        // d=a deletes all placements of the specified image
        format!("\x1b_Ga=d,d=a,i={},q=2;\x1b\\", self.image_id).into_bytes()
    }
}

impl View for KittyImage {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
        // Image needs retransmission if bounds change
        self.transmitted = false;
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width_clamped() as usize;
        let height = self.bounds.height() as usize;

        // Calculate columns and rows for the image
        let cols = if self.columns > 0 {
            self.columns
        } else {
            width as u16
        };
        let rows = if self.rows > 0 {
            self.rows
        } else {
            height as u16
        };

        // Fill the view area with background color (for non-Kitty terminals
        // or as a fallback)
        for row in 0..height {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', self.background_attr, width);
            write_line_to_terminal(
                terminal,
                self.bounds.a.x,
                self.bounds.a.y + row as i16,
                &buf,
            );
        }

        // Check if we have image data
        if self.png_data.is_empty() {
            return;
        }

        let bounds_changed = self.last_bounds.is_none_or(|last| last != self.bounds);

        // Only update the image placement if something changed
        if bounds_changed || !self.transmitted {
            // Delete old placements if bounds changed (position or size)
            if self.last_bounds.is_some() && bounds_changed {
                let delete_seq = self.build_delete_placements_sequence();
                let _ = terminal.write_kitty_graphics(&delete_seq);
            }

            // Flush the terminal buffer first so the background is drawn
            // before we overlay the Kitty image
            let _ = terminal.flush();

            // Transmit the image if not already transmitted
            if !self.transmitted {
                let transmit_seq = self.build_transmit_sequence();
                let _ = terminal.write_kitty_graphics(&transmit_seq);
                self.transmitted = true;
            }

            // Display the image at the view position
            let display_seq = self.build_display_sequence(
                self.bounds.a.x as u16,
                self.bounds.a.y as u16,
                cols,
                rows,
            );
            let _ = terminal.write_kitty_graphics(&display_seq);

            // Remember bounds for next draw
            self.last_bounds = Some(self.bounds);
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Image view doesn't handle events
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_BACKGROUND))
    }
}

/// Builder for creating Kitty image views with a fluent API.
pub struct KittyImageBuilder {
    bounds: Option<Rect>,
    png_data: Option<Vec<u8>>,
    background_attr: Option<Attr>,
    columns: u16,
    rows: u16,
    z_index: i32,
}

impl KittyImageBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            bounds: None,
            png_data: None,
            background_attr: None,
            columns: 0,
            rows: 0,
            z_index: 0,
        }
    }

    /// Sets the bounds.
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the image from a file path.
    #[must_use]
    pub fn file<P: AsRef<Path>>(mut self, path: P) -> Self {
        if let Ok(data) = std::fs::read(path) {
            self.png_data = Some(data);
        }
        self
    }

    /// Sets the image from raw PNG bytes.
    #[must_use]
    pub fn bytes(mut self, data: Vec<u8>) -> Self {
        self.png_data = Some(data);
        self
    }

    /// Sets the background attribute.
    #[must_use]
    pub fn background(mut self, attr: Attr) -> Self {
        self.background_attr = Some(attr);
        self
    }

    /// Sets the number of columns.
    #[must_use]
    pub fn columns(mut self, cols: u16) -> Self {
        self.columns = cols;
        self
    }

    /// Sets the number of rows.
    #[must_use]
    pub fn rows(mut self, rows: u16) -> Self {
        self.rows = rows;
        self
    }

    /// Sets the Z-index.
    #[must_use]
    pub fn z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }

    /// Builds the `KittyImage`.
    ///
    /// # Panics
    /// Panics if bounds or png_data are not set.
    pub fn build(self) -> KittyImage {
        let mut image = KittyImage::from_bytes(
            self.bounds.expect("bounds must be set"),
            self.png_data.expect("png_data must be set"),
        );
        if let Some(attr) = self.background_attr {
            image.background_attr = attr;
        }
        image.columns = self.columns;
        image.rows = self.rows;
        image.z_index = self.z_index;
        image
    }

    /// Builds and boxes the `KittyImage`.
    pub fn build_boxed(self) -> Box<KittyImage> {
        Box::new(self.build())
    }
}

impl Default for KittyImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
