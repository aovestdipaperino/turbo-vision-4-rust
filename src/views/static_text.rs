// (C) 2025 - Enzo Lombardi

//! StaticText view - multi-line static text display with word wrapping.

use super::view::{write_line_to_terminal, View};
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::STATIC_TEXT_NORMAL;
use crate::terminal::Terminal;

pub struct StaticText {
    bounds: Rect,
    text: String,
    centered: bool,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl StaticText {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            centered: false,
            owner: None,
            owner_type: super::view::OwnerType::Dialog, // StaticText defaults to Dialog context
        }
    }

    pub fn new_centered(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            centered: true,
            owner: None,
            owner_type: super::view::OwnerType::Dialog, // StaticText defaults to Dialog context
        }
    }
}

impl View for StaticText {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width_clamped() as usize;
        let lines: Vec<&str> = self.text.split('\n').collect();

        // StaticText palette color index 1 = normal text
        let text_attr = self.map_color(STATIC_TEXT_NORMAL);

        for (i, line) in lines.iter().enumerate() {
            if i >= self.bounds.height_clamped() as usize {
                break;
            }
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', text_attr, width);

            // Calculate starting position based on centering
            let start_pos = if self.centered {
                let line_len = line.len();
                if width > line_len {
                    (width - line_len) / 2
                } else {
                    0
                }
            } else {
                0
            };

            // For now, use same color for shortcuts (no separate shortcut color in StaticText palette)
            buf.move_str_with_shortcut(start_pos, line, text_attr, text_attr);
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Static text doesn't handle events
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_STATIC_TEXT))
    }
}

/// Builder for creating static text views with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::static_text::StaticTextBuilder;
/// use turbo_vision::core::geometry::Rect;
///
/// // Create left-aligned static text
/// let text = StaticTextBuilder::new()
///     .bounds(Rect::new(2, 2, 40, 4))
///     .text("Hello, World!")
///     .build();
///
/// // Create centered static text
/// let text = StaticTextBuilder::new()
///     .bounds(Rect::new(2, 6, 40, 8))
///     .text("Centered Text")
///     .centered(true)
///     .build();
/// ```
pub struct StaticTextBuilder {
    bounds: Option<Rect>,
    text: Option<String>,
    centered: bool,
}

impl StaticTextBuilder {
    /// Creates a new StaticTextBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            text: None,
            centered: false,
        }
    }

    /// Sets the static text bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the text to display (required).
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets whether the text should be centered (default: false).
    #[must_use]
    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    /// Builds the StaticText.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, text) are not set.
    pub fn build(self) -> StaticText {
        let bounds = self.bounds.expect("StaticText bounds must be set");
        let text = self.text.expect("StaticText text must be set");

        StaticText {
            bounds,
            text,
            centered: self.centered,
            owner: None,
            owner_type: super::view::OwnerType::Dialog,
        }
    }

    /// Builds the StaticText as a Box.
    pub fn build_boxed(self) -> Box<StaticText> {
        Box::new(self.build())
    }
}

impl Default for StaticTextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
