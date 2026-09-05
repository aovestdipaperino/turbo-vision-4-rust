// (C) 2026 - Enzo Lombardi

//! ComboBox - input field with a drop-down list.

use std::cell::RefCell;
use std::rc::Rc;

use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{
    Event, EventType, KB_DOWN, KB_ENTER, KB_ESC, KB_ESC_ESC, KB_UP, MB_LEFT_BUTTON,
};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{View, ViewCore};

/// Drop-down selection field: `[Choice        ▼]`.
///
/// The field shows the current selection (shared through an
/// `Rc<RefCell<String>>`, like `InputLine`). Down or a click opens the
/// drop-down under the field; Up/Down navigate, Enter or a click picks an
/// item, Esc closes without changing the value.
///
/// The drop-down is drawn below the field's bounds during this view's draw
/// pass, so place combo boxes after (on top of) the controls the list may
/// overlap, or leave room below.
///
/// # Examples
///
/// ```
/// use std::{cell::RefCell, rc::Rc};
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision_extras::ComboBox;
///
/// let choice = Rc::new(RefCell::new(String::new()));
/// let combo = ComboBox::new(
///     Rect::new(2, 2, 22, 3),
///     vec!["Red".into(), "Green".into(), "Blue".into()],
///     choice.clone(),
/// );
/// assert!(!combo.is_open());
/// ```
#[derive(Debug)]
pub struct ComboBox {
    core: ViewCore,
    items: Vec<String>,
    data: Rc<RefCell<String>>,
    open: bool,
    highlighted: usize,
    max_drop_rows: usize,
    palette_chain: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
}

impl ComboBox {
    /// Create a combo box over `items`, sharing the selection in `data`.
    pub fn new(bounds: Rect, items: Vec<String>, data: Rc<RefCell<String>>) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: 0,
                ..ViewCore::default()
            },
            items,
            data,
            open: false,
            highlighted: 0,
            max_drop_rows: 8,
            palette_chain: None,
        }
    }

    /// Limit the drop-down height (default 8 rows).
    pub fn set_max_drop_rows(&mut self, rows: usize) {
        self.max_drop_rows = rows.max(1);
    }

    /// Replace the item list.
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.highlighted = 0;
    }

    /// Current selection text.
    pub fn selection(&self) -> String {
        self.data.borrow().clone()
    }

    /// True while the drop-down list is open.
    pub fn is_open(&self) -> bool {
        self.open
    }

    fn drop_rows(&self) -> usize {
        self.items.len().min(self.max_drop_rows)
    }

    /// Screen rectangle of the open drop-down.
    fn drop_bounds(&self) -> Rect {
        Rect::new(
            self.core.bounds.a.x,
            self.core.bounds.a.y + 1,
            self.core.bounds.b.x,
            self.core.bounds.a.y + 1 + self.drop_rows() as i16,
        )
    }

    fn open_list(&mut self) {
        if self.items.is_empty() {
            return;
        }
        // Start highlighting the current selection when it is in the list
        let current = self.data.borrow();
        self.highlighted = self
            .items
            .iter()
            .position(|item| *item == *current)
            .unwrap_or(0);
        drop(current);
        self.open = true;
    }

    fn commit(&mut self) {
        if let Some(item) = self.items.get(self.highlighted) {
            *self.data.borrow_mut() = item.clone();
        }
        self.open = false;
    }
}

impl View for ComboBox {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        if width < 3 {
            return;
        }

        let field_attr = if self.is_focused() {
            Attr::new(TvColor::Yellow, TvColor::Blue)
        } else {
            Attr::new(TvColor::White, TvColor::Blue)
        };
        let arrow_attr = Attr::new(TvColor::Green, TvColor::Blue);

        // Field row
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', field_attr, width);
        let text: String = self.data.borrow().chars().take(width - 2).collect();
        buf.move_str(0, &text, field_attr);
        buf.put_char(width - 1, if self.open { '▲' } else { '▼' }, arrow_attr);
        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);

        // Drop-down
        if self.open {
            let list_attr = Attr::new(TvColor::Black, TvColor::Cyan);
            let hl_attr = Attr::new(TvColor::White, TvColor::Green);
            let rows = self.drop_rows();
            let top = if self.highlighted >= rows {
                self.highlighted + 1 - rows
            } else {
                0
            };
            for row in 0..rows {
                let idx = top + row;
                let attr = if idx == self.highlighted {
                    hl_attr
                } else {
                    list_attr
                };
                let mut buf = DrawBuffer::new(width);
                buf.move_char(0, ' ', attr, width);
                if let Some(item) = self.items.get(idx) {
                    let text: String = item.chars().take(width).collect();
                    buf.move_str(0, &text, attr);
                }
                write_line_to_terminal(
                    terminal,
                    self.core.bounds.a.x,
                    self.core.bounds.a.y + 1 + row as i16,
                    &buf,
                );
            }
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard if self.is_focused() => {
                if self.open {
                    match event.key_code {
                        KB_UP => self.highlighted = self.highlighted.saturating_sub(1),
                        KB_DOWN => {
                            self.highlighted =
                                (self.highlighted + 1).min(self.items.len().saturating_sub(1));
                        }
                        KB_ENTER => self.commit(),
                        KB_ESC | KB_ESC_ESC => self.open = false,
                        _ => return, // swallow nothing else
                    }
                    event.clear();
                } else if event.key_code == KB_DOWN {
                    self.open_list();
                    event.clear();
                }
            }
            EventType::MouseDown => {
                let pos = event.mouse.pos;
                if event.mouse.buttons & MB_LEFT_BUTTON == 0 {
                    return;
                }
                if self.core.bounds.contains(pos) {
                    if self.open {
                        self.open = false;
                    } else {
                        self.open_list();
                    }
                    event.clear();
                } else if self.open {
                    let drop = self.drop_bounds();
                    if drop.contains(pos) {
                        let rows = self.drop_rows();
                        let top = if self.highlighted >= rows {
                            self.highlighted + 1 - rows
                        } else {
                            0
                        };
                        let idx = top + (pos.y - drop.a.y) as usize;
                        if idx < self.items.len() {
                            self.highlighted = idx;
                            self.commit();
                        }
                    } else {
                        // Click elsewhere closes the list without selecting
                        self.open = false;
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

    fn set_focus(&mut self, focused: bool) {
        self.set_state_flag(turbo_vision::core::state::SF_FOCUSED, focused);
        if !focused {
            self.open = false;
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::geometry::Point;

    fn make() -> (ComboBox, Rc<RefCell<String>>) {
        let data = Rc::new(RefCell::new("Green".to_string()));
        let mut combo = ComboBox::new(
            Rect::new(0, 0, 20, 1),
            vec!["Red".into(), "Green".into(), "Blue".into()],
            data.clone(),
        );
        combo.set_focus(true);
        (combo, data)
    }

    #[test]
    fn keyboard_open_navigate_select() {
        let (mut combo, data) = make();

        // Down opens, highlighting the current selection
        let mut ev = Event::keyboard(KB_DOWN);
        combo.handle_event(&mut ev);
        assert!(combo.is_open());

        // Down again moves to "Blue"; Enter commits
        let mut ev = Event::keyboard(KB_DOWN);
        combo.handle_event(&mut ev);
        let mut ev = Event::keyboard(KB_ENTER);
        combo.handle_event(&mut ev);
        assert!(!combo.is_open());
        assert_eq!(*data.borrow(), "Blue");
    }

    #[test]
    fn escape_closes_without_changing() {
        let (mut combo, data) = make();
        let mut ev = Event::keyboard(KB_DOWN);
        combo.handle_event(&mut ev);
        let mut ev = Event::keyboard(KB_UP);
        combo.handle_event(&mut ev);
        let mut ev = Event::keyboard(KB_ESC);
        combo.handle_event(&mut ev);
        assert!(!combo.is_open());
        assert_eq!(*data.borrow(), "Green");
    }

    #[test]
    fn mouse_click_selects_item() {
        let (mut combo, data) = make();
        // Click the field opens
        let mut ev = Event::mouse(
            EventType::MouseDown,
            Point::new(3, 0),
            MB_LEFT_BUTTON,
            false,
        );
        combo.handle_event(&mut ev);
        assert!(combo.is_open());
        // Click the first drop-down row ("Red")
        let mut ev = Event::mouse(
            EventType::MouseDown,
            Point::new(3, 1),
            MB_LEFT_BUTTON,
            false,
        );
        combo.handle_event(&mut ev);
        assert!(!combo.is_open());
        assert_eq!(*data.borrow(), "Red");
    }

    #[test]
    fn losing_focus_closes_the_list() {
        let (mut combo, _) = make();
        let mut ev = Event::keyboard(KB_DOWN);
        combo.handle_event(&mut ev);
        combo.set_focus(false);
        assert!(!combo.is_open());
    }
}
