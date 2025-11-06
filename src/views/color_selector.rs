// (C) 2025 - Enzo Lombardi

//! Color Selector - interactive color picker control
//!
//! Matches Borland: TColorSelector
//!
//! Provides an interactive grid of colors for selection.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_UP, KB_DOWN, KB_LEFT, KB_RIGHT, KB_ENTER, MB_LEFT_BUTTON};
use crate::core::state::StateFlags;
use crate::core::palette::{TvColor, Attr};
use crate::core::draw::DrawBuffer;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

const COLORS_PER_ROW: usize = 8;

/// Color Selector - interactive color picker
/// Matches Borland: TColorSelector
pub struct ColorSelector {
    bounds: Rect,
    state: StateFlags,
    /// Currently selected color (0-15)
    selected_color: u8,
    /// Whether selecting foreground (true) or background (false)
    selecting_foreground: bool,
}

impl ColorSelector {
    /// Create a new color selector
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            state: 0,
            selected_color: 7, // White
            selecting_foreground: true,
        }
    }

    /// Get the selected color
    pub fn get_selected_color(&self) -> u8 {
        self.selected_color
    }

    /// Set the selected color
    pub fn set_selected_color(&mut self, color: u8) {
        self.selected_color = color.min(15);
    }

    /// Get position of color in grid
    fn color_to_pos(&self, color: u8) -> (i16, i16) {
        let row = (color / COLORS_PER_ROW as u8) as i16;
        let col = (color % COLORS_PER_ROW as u8) as i16;
        (col * 3, row)  // 3 chars per color cell
    }

    /// Get color from position in grid
    fn pos_to_color(&self, x: i16, y: i16) -> Option<u8> {
        if x < 0 || y < 0 {
            return None;
        }
        let col = x / 3;
        let row = y;
        if row < 2 && col < COLORS_PER_ROW as i16 {
            Some((row * COLORS_PER_ROW as i16 + col) as u8)
        } else {
            None
        }
    }
}

impl View for ColorSelector {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;

        // Draw color grid (16 colors in 2 rows of 8)
        for row in 0..2 {
            let mut buf = DrawBuffer::new(width);

            for col in 0..COLORS_PER_ROW {
                let color_idx = (row * COLORS_PER_ROW + col) as u8;
                let is_selected = color_idx == self.selected_color;

                // Create color attribute for display
                let attr = Attr::from_u8((color_idx << 4) | color_idx);

                // Draw color cell
                let x = col * 3;
                if is_selected {
                    // Show selection with brackets
                    buf.move_char(x, '[', attr, 1);
                    buf.move_char(x + 1, ' ', attr, 1);
                    buf.move_char(x + 2, ']', attr, 1);
                } else {
                    buf.move_char(x, ' ', attr, 3);
                }
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + row as i16, &buf);
        }

        // Draw color labels row
        if self.bounds.height() > 2 {
            let mut label_buf = DrawBuffer::new(width);
            let label_attr = Attr::from_u8(0x07); // Normal text
            let text = format!("Selected: {} (0x{:02X})",
                self.selected_color, self.selected_color);
            label_buf.move_str(0, &text, label_attr);
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + 2, &label_buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => {
                let (col, row) = self.color_to_pos(self.selected_color);
                let mut new_col = col;
                let mut new_row = row;

                match event.key_code {
                    KB_LEFT => new_col = (col - 3).max(0),
                    KB_RIGHT => new_col = (col + 3).min((COLORS_PER_ROW - 1) as i16 * 3),
                    KB_UP => new_row = (row - 1).max(0),
                    KB_DOWN => new_row = (row + 1).min(1),
                    KB_ENTER => {
                        // Enter confirms selection (emit command)
                        *event = Event::command(100); // Custom command for color selected
                        return;
                    }
                    _ => return,
                }

                if let Some(new_color) = self.pos_to_color(new_col, new_row) {
                    self.selected_color = new_color;
                    event.clear();
                }
            }
            EventType::MouseDown => {
                if event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    let mouse_pos = event.mouse.pos;
                    if self.bounds.contains(mouse_pos) {
                        let rel_x = mouse_pos.x - self.bounds.a.x;
                        let rel_y = mouse_pos.y - self.bounds.a.y;

                        if let Some(color) = self.pos_to_color(rel_x, rel_y) {
                            self.selected_color = color;
                            event.clear();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }
}
