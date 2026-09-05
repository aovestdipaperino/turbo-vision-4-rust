// (C) 2026 - Enzo Lombardi

//! SpinControl - numeric field with ▲/▼ steppers (TV Tool Box style).

use std::cell::RefCell;
use std::rc::Rc;

use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{
    Event, EventType, KB_DOWN, KB_PGDN, KB_PGUP, KB_UP, MB_LEFT_BUTTON,
};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{View, ViewCore};

/// Numeric spinner: `[  42 ]▲▼`.
///
/// Up/Down step by 1 (PgUp/PgDn by 10), and the ▲/▼ cells respond to
/// clicks. The value is shared through an `Rc<RefCell<i32>>` so the owner
/// reads it back after the dialog closes, mirroring how `InputLine` shares
/// its data.
///
/// # Examples
///
/// ```
/// use std::{cell::RefCell, rc::Rc};
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision_extras::SpinControl;
///
/// let value = Rc::new(RefCell::new(5));
/// let spin = SpinControl::new(Rect::new(2, 2, 12, 3), 0, 10, value.clone());
/// assert_eq!(*value.borrow(), 5);
/// ```
#[derive(Debug)]
pub struct SpinControl {
    core: ViewCore,
    min: i32,
    max: i32,
    value: Rc<RefCell<i32>>,
    palette_chain: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
}

impl SpinControl {
    /// Create a spinner over `min..=max` sharing `value` with the caller.
    ///
    /// The initial shared value is clamped into range.
    pub fn new(bounds: Rect, min: i32, max: i32, value: Rc<RefCell<i32>>) -> Self {
        let max = max.max(min);
        {
            let mut v = value.borrow_mut();
            *v = (*v).clamp(min, max);
        }
        Self {
            core: ViewCore {
                bounds,
                state: 0,
                ..ViewCore::default()
            },
            min,
            max,
            value,
            palette_chain: None,
        }
    }

    /// Current value.
    pub fn value(&self) -> i32 {
        *self.value.borrow()
    }

    /// Set the value, clamped to the range.
    pub fn set_value(&mut self, value: i32) {
        *self.value.borrow_mut() = value.clamp(self.min, self.max);
    }

    fn step(&mut self, delta: i32) {
        let v = self.value();
        self.set_value(v.saturating_add(delta));
    }

    /// Column of the ▲ cell (relative to bounds).
    fn up_col(&self) -> i16 {
        self.core.bounds.width() - 2
    }

    /// Column of the ▼ cell (relative to bounds).
    fn down_col(&self) -> i16 {
        self.core.bounds.width() - 1
    }
}

impl View for SpinControl {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        if width < 4 {
            return;
        }

        let field_attr = if self.is_focused() {
            Attr::new(TvColor::Yellow, TvColor::Blue)
        } else {
            Attr::new(TvColor::White, TvColor::Blue)
        };
        let arrow_attr = Attr::new(TvColor::Green, TvColor::Blue);

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', field_attr, width);
        let text = self.value().to_string();
        let field_width = width - 2;
        let start = field_width.saturating_sub(text.chars().count() + 1);
        buf.move_str(start, &text, field_attr);
        buf.put_char(self.up_col() as usize, '▲', arrow_attr);
        buf.put_char(self.down_col() as usize, '▼', arrow_attr);

        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard if self.is_focused() => {
                match event.key_code {
                    KB_UP => self.step(1),
                    KB_DOWN => self.step(-1),
                    KB_PGUP => self.step(10),
                    KB_PGDN => self.step(-10),
                    _ => return,
                }
                event.clear();
            }
            EventType::MouseDown => {
                let pos = event.mouse.pos;
                if event.mouse.buttons & MB_LEFT_BUTTON != 0 && self.core.bounds.contains(pos) {
                    let col = pos.x - self.core.bounds.a.x;
                    if col == self.up_col() {
                        self.step(1);
                    } else if col == self.down_col() {
                        self.step(-1);
                    }
                    event.clear();
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }

    fn set_palette_chain(
        &mut self,
        node: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
    ) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&turbo_vision::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::geometry::Point;

    fn make(min: i32, max: i32, initial: i32) -> (SpinControl, Rc<RefCell<i32>>) {
        let value = Rc::new(RefCell::new(initial));
        let mut spin = SpinControl::new(Rect::new(0, 0, 10, 1), min, max, value.clone());
        spin.set_focus(true);
        (spin, value)
    }

    #[test]
    fn keyboard_steps_and_clamps() {
        let (mut spin, value) = make(0, 5, 4);

        let mut ev = Event::keyboard(KB_UP);
        spin.handle_event(&mut ev);
        assert_eq!(*value.borrow(), 5);
        let mut ev = Event::keyboard(KB_UP);
        spin.handle_event(&mut ev);
        assert_eq!(*value.borrow(), 5); // clamped

        let mut ev = Event::keyboard(KB_PGDN);
        spin.handle_event(&mut ev);
        assert_eq!(*value.borrow(), 0); // clamped page step
    }

    #[test]
    fn arrow_cells_respond_to_clicks() {
        let (mut spin, value) = make(0, 10, 5);

        // ▲ is at column width-2 = 8
        let mut ev = Event::mouse(
            EventType::MouseDown,
            Point::new(8, 0),
            MB_LEFT_BUTTON,
            false,
        );
        spin.handle_event(&mut ev);
        assert_eq!(*value.borrow(), 6);

        // ▼ is at column width-1 = 9
        let mut ev = Event::mouse(
            EventType::MouseDown,
            Point::new(9, 0),
            MB_LEFT_BUTTON,
            false,
        );
        spin.handle_event(&mut ev);
        assert_eq!(*value.borrow(), 5);
    }

    #[test]
    fn initial_value_is_clamped() {
        let (_spin, value) = make(0, 5, 99);
        assert_eq!(*value.borrow(), 5);
    }
}
