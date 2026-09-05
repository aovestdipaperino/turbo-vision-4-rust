// (C) 2025 - Enzo Lombardi

//! Indicator view - visual indicator for displaying scroll position or progress.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::{Point, Rect};
use crate::terminal::Terminal;

/// Indicator displays window size or cursor position,
/// typically shown in the bottom-left of an editor window.
pub struct Indicator {
    core: ViewCore,
    location: Point, // Width x Height for window size display
    modified: bool,  // Has the document been modified?
}

impl Indicator {
    pub fn new(bounds: Rect) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            location: Point::new(1, 1),
            modified: false,
        }
    }

    /// Cursor position text: " line:col " (row first, 1-based).
    ///
    /// Matches Borland TIndicator::draw ("%d:%d", location.y+1, location.x+1).
    fn format_text(&self) -> String {
        format!(" {}:{} ", self.location.y + 1, self.location.x + 1)
    }

    pub fn set_value(&mut self, location: Point, modified: bool) {
        self.location = location;
        self.modified = modified;
    }
}

impl View for Indicator {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        let mut buf = DrawBuffer::new(width);

        // Use palette indices from CP_INDICATOR
        // 1 = Normal indicator, 2 = Modified indicator
        let color = if self.modified {
            self.map_color(2)
        } else {
            self.map_color(1)
        };

        // Fill with spaces (background)
        buf.move_char(0, ' ', color, width);

        // Show modified star at the left if modified (matching Borland)
        if self.modified {
            buf.move_char(0, '*', color, 1);
        }

        let text = self.format_text();

        // Center the text around the ':' separator
        if let Some(x_pos) = text.find(':') {
            let start_pos = (8_i32 - x_pos as i32).max(1) as usize;
            let start_pos = start_pos.min(width.saturating_sub(text.len()));
            buf.move_str(start_pos, &text, color);
        } else {
            // Fallback: center normally if no ':' found
            let start_pos = (width / 2).saturating_sub(text.len() / 2);
            buf.move_str(start_pos, &text, color);
        }

        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Indicator doesn't handle events
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_INDICATOR))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_shows_borland_line_colon_col() {
        // Regression: used to display " col x row " ("13x4")
        let mut ind = Indicator::new(Rect::new(0, 0, 12, 1));
        ind.set_value(Point::new(12, 3), false); // cursor col 12, row 3 (0-based)
        assert_eq!(ind.format_text(), " 4:13 ");
    }
}
