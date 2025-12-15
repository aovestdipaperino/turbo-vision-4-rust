// (C) 2025 - Enzo Lombardi

//! ANSI art background view - displays colored ASCII art from ANSI escape sequences.
//!
//! This view renders ANSI art centered on the desktop background, supporting
//! 16-color, 256-color, and true color ANSI escape sequences.
//!
//! # Example
//!
//! ```rust,no_run
//! use turbo_vision::views::ansi_background::AnsiBackground;
//! use turbo_vision::core::geometry::Rect;
//! use turbo_vision::core::palette::{Attr, TvColor};
//!
//! // Load from file
//! let bg = AnsiBackground::from_file(
//!     Rect::new(0, 0, 80, 24),
//!     "logo.ans",
//!     Attr::new(TvColor::LightGray, TvColor::DarkGray),
//! ).expect("Failed to load ANSI file");
//!
//! // Or from string
//! let ansi_art = "\x1b[31mRed\x1b[0m Logo";
//! let bg = AnsiBackground::from_string(
//!     Rect::new(0, 0, 80, 24),
//!     ansi_art,
//!     Attr::new(TvColor::LightGray, TvColor::DarkGray),
//! );
//! ```

use crate::core::ansi::AnsiImage;
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::Attr;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};
use std::io;
use std::path::Path;

/// A background view that displays ANSI art.
pub struct AnsiBackground {
    bounds: Rect,
    state: StateFlags,
    /// The parsed ANSI image.
    image: AnsiImage,
    /// Default attribute for areas outside the image.
    default_attr: Attr,
    /// Whether to center the image horizontally.
    center_x: bool,
    /// Whether to center the image vertically.
    center_y: bool,
    owner: Option<*const dyn View>,
}

impl AnsiBackground {
    /// Creates a new ANSI background from an `AnsiImage`.
    ///
    /// # Arguments
    /// * `bounds` - The bounding rectangle for the view
    /// * `image` - The parsed ANSI image to display
    /// * `default_attr` - Color attributes for areas outside the image
    pub fn new(bounds: Rect, image: AnsiImage, default_attr: Attr) -> Self {
        Self {
            bounds,
            state: 0,
            image,
            default_attr,
            center_x: true,
            center_y: true,
            owner: None,
        }
    }

    /// Creates a new ANSI background by loading from a file.
    ///
    /// # Arguments
    /// * `bounds` - The bounding rectangle for the view
    /// * `path` - Path to the ANSI art file
    /// * `default_attr` - Color attributes for areas outside the image
    ///
    /// # Errors
    /// Returns an IO error if the file cannot be read.
    pub fn from_file<P: AsRef<Path>>(
        bounds: Rect,
        path: P,
        default_attr: Attr,
    ) -> io::Result<Self> {
        let image = AnsiImage::load(path)?;
        Ok(Self::new(bounds, image, default_attr))
    }

    /// Creates a new ANSI background from a string.
    ///
    /// # Arguments
    /// * `bounds` - The bounding rectangle for the view
    /// * `content` - String containing ANSI escape sequences
    /// * `default_attr` - Color attributes for areas outside the image
    pub fn from_string(bounds: Rect, content: &str, default_attr: Attr) -> Self {
        let image = AnsiImage::parse(content);
        Self::new(bounds, image, default_attr)
    }

    /// Sets whether to center the image horizontally.
    #[must_use]
    pub fn center_x(mut self, center: bool) -> Self {
        self.center_x = center;
        self
    }

    /// Sets whether to center the image vertically.
    #[must_use]
    pub fn center_y(mut self, center: bool) -> Self {
        self.center_y = center;
        self
    }

    /// Sets whether to center the image both horizontally and vertically.
    #[must_use]
    pub fn centered(mut self, center: bool) -> Self {
        self.center_x = center;
        self.center_y = center;
        self
    }

    /// Gets the ANSI image.
    pub fn image(&self) -> &AnsiImage {
        &self.image
    }

    /// Sets a new ANSI image.
    pub fn set_image(&mut self, image: AnsiImage) {
        self.image = image;
    }

    /// Loads a new image from a file.
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        self.image = AnsiImage::load(path)?;
        Ok(())
    }

    /// Parses and sets content from a string.
    pub fn set_content(&mut self, content: &str) {
        self.image = AnsiImage::parse(content);
    }
}

impl View for AnsiBackground {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
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

        // Calculate offsets for centering
        let x_offset = if self.center_x && self.image.width < width {
            (width - self.image.width) / 2
        } else {
            0
        };

        let y_offset = if self.center_y && self.image.height < height {
            (height - self.image.height) / 2
        } else {
            0
        };

        // Draw each row
        for row in 0..height {
            let mut buf = DrawBuffer::new(width);

            // Fill entire line with default attribute
            buf.move_char(0, ' ', self.default_attr, width);

            // Calculate which image row to draw (if any)
            if row >= y_offset && row < y_offset + self.image.height {
                let image_row = row - y_offset;

                if let Some(line) = self.image.lines.get(image_row) {
                    // Draw the image line at the offset position
                    for (col, cell) in line.iter().enumerate() {
                        let x = x_offset + col;
                        if x < width {
                            buf.put_char(x, cell.ch, cell.attr);
                        }
                    }
                }
            }

            write_line_to_terminal(
                terminal,
                self.bounds.a.x,
                self.bounds.a.y + row as i16,
                &buf,
            );
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Background doesn't handle events
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

/// Builder for creating ANSI backgrounds with a fluent API.
pub struct AnsiBackgroundBuilder {
    bounds: Option<Rect>,
    image: Option<AnsiImage>,
    default_attr: Option<Attr>,
    center_x: bool,
    center_y: bool,
}

impl AnsiBackgroundBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            bounds: None,
            image: None,
            default_attr: None,
            center_x: true,
            center_y: true,
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
        if let Ok(image) = AnsiImage::load(path) {
            self.image = Some(image);
        }
        self
    }

    /// Sets the image from a string.
    #[must_use]
    pub fn content(mut self, content: &str) -> Self {
        self.image = Some(AnsiImage::parse(content));
        self
    }

    /// Sets the image directly.
    #[must_use]
    pub fn image(mut self, image: AnsiImage) -> Self {
        self.image = Some(image);
        self
    }

    /// Sets the default attribute.
    #[must_use]
    pub fn default_attr(mut self, attr: Attr) -> Self {
        self.default_attr = Some(attr);
        self
    }

    /// Sets horizontal centering.
    #[must_use]
    pub fn center_x(mut self, center: bool) -> Self {
        self.center_x = center;
        self
    }

    /// Sets vertical centering.
    #[must_use]
    pub fn center_y(mut self, center: bool) -> Self {
        self.center_y = center;
        self
    }

    /// Sets both centering options.
    #[must_use]
    pub fn centered(mut self, center: bool) -> Self {
        self.center_x = center;
        self.center_y = center;
        self
    }

    /// Builds the `AnsiBackground`.
    ///
    /// # Panics
    /// Panics if bounds, image, or `default_attr` are not set.
    pub fn build(self) -> AnsiBackground {
        let mut bg = AnsiBackground::new(
            self.bounds.expect("bounds must be set"),
            self.image.expect("image must be set"),
            self.default_attr.expect("default_attr must be set"),
        );
        bg.center_x = self.center_x;
        bg.center_y = self.center_y;
        bg
    }

    /// Builds and boxes the `AnsiBackground`.
    pub fn build_boxed(self) -> Box<AnsiBackground> {
        Box::new(self.build())
    }
}

impl Default for AnsiBackgroundBuilder {
    fn default() -> Self {
        Self::new()
    }
}
