// (C) 2025 - Enzo Lombardi

//! Background view - solid color background fill for containers.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::Attr;
use crate::core::state::{GF_GROW_HI_X, GF_GROW_HI_Y};
use crate::terminal::Terminal;

/// Background view - fills its bounds with a pattern character
/// Matches Borland's TBackground (tbackgro.cc)
pub struct Background {
    core: ViewCore,
    pattern: char,
    attr: Attr,
}

impl Background {
    pub fn new(bounds: Rect, pattern: char, attr: Attr) -> Self {
        Self {
            core: ViewCore {
                bounds,
                grow_mode: GF_GROW_HI_X | GF_GROW_HI_Y,
                palette_chain: None,
                ..ViewCore::default()
            },
            pattern,
            attr,
            // Matches Borland: TBackground's growMode is
            // gfGrowHiX | gfGrowHiY, so the background follows its owner's
            // bottom-right corner on a resize. Without it `Group::set_bounds`
            // only *translates* the background, and every column or row the
            // owner gains after construction is left unpainted - the classic
            // black band down the right edge of the desktop after the
            // terminal is widened.
        }
    }
}

impl View for Background {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, self.pattern, self.attr, width);

        // Draw every row
        for y in self.core.bounds.a.y..self.core.bounds.b.y {
            write_line_to_terminal(terminal, self.core.bounds.a.x, y, &buf);
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Background doesn't handle events
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
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
        Self {
            bounds: None,
            pattern: '░',
            attr: None,
        }
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
