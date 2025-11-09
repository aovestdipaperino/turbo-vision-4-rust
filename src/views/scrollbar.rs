// (C) 2025 - Enzo Lombardi

//! ScrollBar view - vertical or horizontal scrollbar with draggable indicator.

use crate::core::geometry::{Point, Rect};
use crate::core::event::{Event, EventType, KB_UP, KB_DOWN, KB_LEFT, KB_RIGHT, KB_PGUP, KB_PGDN, KB_HOME, KB_END};
use crate::core::draw::DrawBuffer;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

/// Scroll bar part codes (used by getPartCode() method)
const SB_INDICATOR: i16 = 0;
const SB_UP_ARROW: i16 = 1;
const SB_DOWN_ARROW: i16 = 2;
const SB_PAGE_UP: i16 = 3;
const SB_PAGE_DOWN: i16 = 4;

/// Scroll bar characters for vertical scrollbar
pub const VSCROLL_CHARS: [char; 5] = [
    '█',  // Indicator
    '▲',  // Up arrow
    '▼',  // Down arrow
    '░',  // Page up area
    '░',  // Page down area
];

/// Scroll bar characters for horizontal scrollbar
pub const HSCROLL_CHARS: [char; 5] = [
    '█',  // Indicator
    '◄',  // Left arrow
    '►',  // Right arrow
    '░',  // Page left area
    '░',  // Page right area
];

pub struct ScrollBar {
    bounds: Rect,
    value: i32,
    min_val: i32,
    max_val: i32,
    pg_step: i32,    // Page step
    ar_step: i32,    // Arrow step
    chars: [char; 5],
    is_vertical: bool,
    owner: Option<*const dyn View>,
}

impl ScrollBar {
    pub fn new_vertical(bounds: Rect) -> Self {
        Self {
            bounds,
            value: 0,
            min_val: 0,
            max_val: 0,
            pg_step: 1,
            ar_step: 1,
            chars: VSCROLL_CHARS,
            is_vertical: true,
            owner: None,
        }
    }

    pub fn new_horizontal(bounds: Rect) -> Self {
        Self {
            bounds,
            value: 0,
            min_val: 0,
            max_val: 0,
            pg_step: 1,
            ar_step: 1,
            chars: HSCROLL_CHARS,
            is_vertical: false,
            owner: None,
        }
    }

    pub fn set_params(&mut self, value: i32, min_val: i32, max_val: i32, pg_step: i32, ar_step: i32) {
        // Ensure max_val >= min_val to prevent division by zero
        self.min_val = min_val;
        self.max_val = max_val.max(min_val);
        self.value = value.max(self.min_val).min(self.max_val);
        self.pg_step = pg_step;
        self.ar_step = ar_step;
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value.max(self.min_val).min(self.max_val);
    }

    pub fn set_range(&mut self, min_val: i32, max_val: i32) {
        self.min_val = min_val;
        self.max_val = max_val;
        self.value = self.value.max(min_val).min(max_val);
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// Get the size of the scrollbar track (not including arrows)
    fn get_size(&self) -> i32 {
        if self.is_vertical {
            (self.bounds.height() - 2).max(1) as i32
        } else {
            (self.bounds.width() - 2).max(1) as i32
        }
    }

    /// Get the position of the indicator
    fn get_pos(&self) -> i32 {
        let s = self.get_size();
        let range = self.max_val - self.min_val + 1;
        if range <= 0 || s <= 0 {
            // Safety check: invalid range or size
            0
        } else {
            ((self.value - self.min_val) * s / range).max(0).min(s - 1)
        }
    }

    /// Get the part of the scrollbar at a given position
    #[expect(dead_code, reason = "Borland TV API - reserved for advanced scrollbar interaction")]
    fn get_part_at(&self, p: Point) -> i16 {
        let rel_x = p.x - self.bounds.a.x;
        let rel_y = p.y - self.bounds.a.y;

        if self.is_vertical {
            if rel_y == 0 {
                SB_UP_ARROW
            } else if rel_y == self.bounds.height() - 1 {
                SB_DOWN_ARROW
            } else {
                let pos = self.get_pos();
                if rel_y - 1 == pos as i16 {
                    SB_INDICATOR
                } else if rel_y - 1 < pos as i16 {
                    SB_PAGE_UP
                } else {
                    SB_PAGE_DOWN
                }
            }
        } else if rel_x == 0 {
            SB_UP_ARROW  // Left arrow for horizontal
        } else if rel_x == self.bounds.width() - 1 {
            SB_DOWN_ARROW  // Right arrow for horizontal
        } else {
            let pos = self.get_pos();
            if rel_x - 1 == pos as i16 {
                SB_INDICATOR
            } else if rel_x - 1 < pos as i16 {
                SB_PAGE_UP  // Page left
            } else {
                SB_PAGE_DOWN  // Page right
            }
        }
    }

    /// Scroll by a given part
    #[expect(dead_code, reason = "Borland TV API - reserved for advanced scrollbar interaction")]
    fn scroll_step(&mut self, part: i16) -> i32 {
        match part {
            SB_UP_ARROW => -self.ar_step,
            SB_DOWN_ARROW => self.ar_step,
            SB_PAGE_UP => -self.pg_step,
            SB_PAGE_DOWN => self.pg_step,
            _ => 0,
        }
    }
}

impl View for ScrollBar {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // ScrollBar palette indices:
        // 1: Page, 2: Arrows, 3: Indicator
        let page_attr = self.map_color(1);
        let indicator_attr = self.map_color(3);

        if self.is_vertical {
            // Draw vertical scrollbar
            let height = self.bounds.height();
            let pos = self.get_pos();

            for y in 0..height {
                let mut buf = DrawBuffer::new(1);
                let ch = if y == 0 {
                    self.chars[1]  // Up arrow
                } else if y == height - 1 {
                    self.chars[2]  // Down arrow
                } else if y - 1 == pos as i16 {
                    self.chars[0]  // Indicator
                } else {
                    self.chars[3]  // Page area
                };

                let attr = if y - 1 == pos as i16 {
                    indicator_attr
                } else {
                    page_attr
                };

                buf.put_char(0, ch, attr);
                write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y, &buf);
            }
        } else {
            // Draw horizontal scrollbar
            let width = self.bounds.width();
            let pos = self.get_pos();
            let mut buf = DrawBuffer::new(width as usize);

            for x in 0..width {
                let ch = if x == 0 {
                    self.chars[1]  // Left arrow
                } else if x == width - 1 {
                    self.chars[2]  // Right arrow
                } else if x - 1 == pos as i16 {
                    self.chars[0]  // Indicator
                } else {
                    self.chars[3]  // Page area
                };

                let attr = if x - 1 == pos as i16 {
                    indicator_attr
                } else {
                    page_attr
                };

                buf.put_char(x as usize, ch, attr);
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::Keyboard {
            if self.is_vertical {
                match event.key_code {
                    KB_UP => {
                        self.value = (self.value - self.ar_step).max(self.min_val);
                        event.clear();
                    }
                    KB_DOWN => {
                        self.value = (self.value + self.ar_step).min(self.max_val);
                        event.clear();
                    }
                    KB_PGUP => {
                        self.value = (self.value - self.pg_step).max(self.min_val);
                        event.clear();
                    }
                    KB_PGDN => {
                        self.value = (self.value + self.pg_step).min(self.max_val);
                        event.clear();
                    }
                    KB_HOME => {
                        self.value = self.min_val;
                        event.clear();
                    }
                    KB_END => {
                        self.value = self.max_val;
                        event.clear();
                    }
                    _ => {}
                }
            } else {
                match event.key_code {
                    KB_LEFT => {
                        self.value = (self.value - self.ar_step).max(self.min_val);
                        event.clear();
                    }
                    KB_RIGHT => {
                        self.value = (self.value + self.ar_step).min(self.max_val);
                        event.clear();
                    }
                    KB_HOME => {
                        self.value = self.min_val;
                        event.clear();
                    }
                    KB_END => {
                        self.value = self.max_val;
                        event.clear();
                    }
                    _ => {}
                }
            }
        }
        // TODO: Add mouse support when mouse events are implemented
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_SCROLLBAR))
    }
}
