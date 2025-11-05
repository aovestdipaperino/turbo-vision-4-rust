// (C) 2025 - Enzo Lombardi
use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::draw::DrawBuffer;
use crate::core::palette::colors;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

pub struct StaticText {
    bounds: Rect,
    text: String,
    centered: bool,
}

impl StaticText {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            centered: false,
        }
    }

    pub fn new_centered(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            centered: true,
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

        for (i, line) in lines.iter().enumerate() {
            if i >= self.bounds.height() as usize {
                break;
            }
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', colors::DIALOG_NORMAL, width);

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

            buf.move_str_with_shortcut(start_pos, line, colors::DIALOG_NORMAL, colors::DIALOG_SHORTCUT);
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Static text doesn't handle events
    }
}
