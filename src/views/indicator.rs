// (C) 2025 - Enzo Lombardi

//! Indicator view - visual indicator for displaying scroll position or progress.

use crate::core::geometry::{Point, Rect};
use crate::core::event::Event;
use crate::core::draw::DrawBuffer;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

/// Indicator displays the current line and column position,
/// typically shown in the top-right corner of an editor window.
pub struct Indicator {
    bounds: Rect,
    location: Point,  // Line and column (1-based)
    modified: bool,   // Has the document been modified?
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl Indicator {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            location: Point::new(1, 1),
            modified: false,
            owner: None,
            owner_type: super::view::OwnerType::None,
        }
    }

    pub fn set_value(&mut self, location: Point, modified: bool) {
        self.location = location;
        self.modified = modified;
    }
}

impl View for Indicator {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);

        // Format: " 12:34 " or " 12:34 * " (with modified star)
        let text = if self.modified {
            format!(" {}:{} * ", self.location.y, self.location.x)
        } else {
            format!(" {}:{} ", self.location.y, self.location.x)
        };

        // Use palette indices from CP_INDICATOR
        // 1 = Normal indicator, 2 = Modified indicator
        let color = if self.modified {
            self.map_color(2)
        } else {
            self.map_color(1)
        };

        // Right-align the indicator
        let text_len = text.len().min(width);
        let start_pos = width.saturating_sub(text_len);

        buf.move_char(0, ' ', color, width);
        buf.move_str(start_pos, &text, color);

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Indicator doesn't handle events
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_INDICATOR))
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

/// Builder for creating indicators with a fluent API.
pub struct IndicatorBuilder {
    bounds: Option<Rect>,
}

impl IndicatorBuilder {
    pub fn new() -> Self {
        Self { bounds: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    pub fn build(self) -> Indicator {
        let bounds = self.bounds.expect("Indicator bounds must be set");
        Indicator::new(bounds)
    }

    pub fn build_boxed(self) -> Box<Indicator> {
        Box::new(self.build())
    }
}

impl Default for IndicatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
