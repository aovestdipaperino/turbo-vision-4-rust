// (C) 2025 - Enzo Lombardi

//! StaticText view - multi-line static text display with word wrapping.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::draw::DrawBuffer;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

pub struct StaticText {
    bounds: Rect,
    text: String,
    centered: bool,
    owner: Option<*const dyn View>,
}

impl StaticText {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            centered: false,
            owner: None,
        }
    }

    pub fn new_centered(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            centered: true,
            owner: None,
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
        let width = self.bounds.width() as usize;
        let lines: Vec<&str> = self.text.split('\n').collect();

        // StaticText palette color index 1 = normal text
        let text_attr = self.map_color(1);

        for (i, line) in lines.iter().enumerate() {
            if i >= self.bounds.height() as usize {
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

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_STATIC_TEXT))
    }
}
