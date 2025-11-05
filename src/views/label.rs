// (C) 2025 - Enzo Lombardi
use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::draw::DrawBuffer;
use crate::core::palette::colors;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

pub struct Label {
    bounds: Rect,
    text: String,
}

impl Label {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
        }
    }
}

impl View for Label {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);

        buf.move_char(0, ' ', colors::DIALOG_NORMAL, width);
        buf.move_str_with_shortcut(0, &self.text, colors::DIALOG_NORMAL, colors::DIALOG_SHORTCUT);

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Labels don't handle events
    }
}
