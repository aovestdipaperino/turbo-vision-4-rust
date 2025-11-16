// (C) 2025 - Enzo Lombardi

//! Background view - solid color background fill for containers.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::draw::DrawBuffer;
use crate::core::palette::Attr;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

/// Background view - fills its bounds with a pattern character
/// Matches Borland's TBackground (tbackgro.cc)
pub struct Background {
    bounds: Rect,
    pattern: char,
    attr: Attr,
    owner: Option<*const dyn View>,
}

impl Background {
    pub fn new(bounds: Rect, pattern: char, attr: Attr) -> Self {
        Self {
            bounds,
            pattern,
            attr,
            owner: None,
        }
    }
}

impl View for Background {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width_clamped() as usize;
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, self.pattern, self.attr, width);

        // Draw every row
        for y in self.bounds.a.y..self.bounds.b.y {
            write_line_to_terminal(terminal, self.bounds.a.x, y, &buf);
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

/// Builder for creating backgrounds with a fluent API.
pub struct BackgroundBuilder {
    bounds: Option<Rect>,
    pattern: char,
    attr: Option<Attr>,
}

impl BackgroundBuilder {
    pub fn new() -> Self {
        Self { bounds: None, pattern: '░', attr: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn pattern(mut self, pattern: char) -> Self {
        self.pattern = pattern;
        self
    }

    #[must_use]
    pub fn attr(mut self, attr: Attr) -> Self {
        self.attr = Some(attr);
        self
    }

    pub fn build(self) -> Background {
        let bounds = self.bounds.expect("Background bounds must be set");
        let attr = self.attr.expect("Background attr must be set");
        Background::new(bounds, self.pattern, attr)
    }

    pub fn build_boxed(self) -> Box<Background> {
        Box::new(self.build())
    }
}

impl Default for BackgroundBuilder {
    fn default() -> Self {
        Self::new()
    }
}
