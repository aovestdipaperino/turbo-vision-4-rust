// (C) 2025 - Enzo Lombardi

//! ComboBox view - a text field showing one choice, with a drop-down list.
//!
//! Not part of the Borland Turbo Vision widget set. It follows the same
//! two-step pattern the history button already uses: the control itself cannot
//! reach the terminal from `handle_event`, so opening the list is a command
//! ([`CM_SHOW_DROPDOWN`]) that the modal `Dialog` loop, or `Application`,
//! turns into a [`DropdownWindow`] running modally.
//!
//! This is the read-only flavour: the user picks from the list and cannot type
//! a value that is not in it. An editable flavour, where the field is a real
//! `InputLine`, is still open on the roadmap.
//!
//! Every combo box registers its shared [`ComboState`] under a caller-chosen
//! id so the popup can find its items. Ids are per-thread and freed when the
//! control is dropped.
//!
//! # Keys
//!
//! | Key | Action |
//! |-----|--------|
//! | F4, Alt+Down | Open the drop-down list |
//! | Up, Down | Step to the previous or next item without opening |
//! | Home, End | Jump to the first or last item |
//!
//! Clicking anywhere on the field also opens the list.
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::views::combo_box::ComboBox;
//! use turbo_vision::core::geometry::Rect;
//!
//! let mut combo = ComboBox::new(Rect::new(10, 3, 30, 4), 1);
//! combo.set_items(vec!["Red".into(), "Green".into(), "Blue".into()]);
//! combo.set_selected(Some(2));
//! assert_eq!(combo.selected_text().as_deref(), Some("Blue"));
//! ```

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::{CM_SHOW_DROPDOWN, CommandId};
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_DOWN, KB_END, KB_ENTER, KB_ESC, KB_F4, KB_HOME, KB_UP,
};
use crate::core::geometry::{Point, Rect};
use crate::core::palette::{INPUT_ARROWS, INPUT_NORMAL, INPUT_SELECTED};
use crate::core::state::{State, StateFlags};
use crate::terminal::Terminal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Glyph drawn at the right edge of the field.
const DROP_ARROW: char = '\u{25BC}'; // ▼

/// Largest number of rows a drop-down list shows before it scrolls.
const MAX_DROPDOWN_ROWS: i16 = 8;

/// The items and current choice of one combo box.
///
/// Shared between the control and the popup that the dialog opens on its
/// behalf, so the popup can list the items and write the choice straight back.
#[derive(Debug, Default)]
pub struct ComboState {
    /// The choices, in display order.
    pub items: Vec<String>,
    /// Index into `items`, or `None` when nothing is chosen.
    pub selected: Option<usize>,
    /// Screen rect of the field, so the popup can be placed under it.
    pub field: Rect,
}

impl ComboState {
    /// Text of the current choice, if any.
    pub fn selected_text(&self) -> Option<&str> {
        self.selected.and_then(|i| self.items.get(i)).map(|s| &**s)
    }

    /// Clamp `selected` to the current item list, dropping it when the list is
    /// empty. Called after any change to `items`.
    fn clamp(&mut self) {
        if self.items.is_empty() {
            self.selected = None;
        } else if let Some(i) = self.selected {
            self.selected = Some(i.min(self.items.len() - 1));
        }
    }
}

thread_local! {
    /// Live combo states by id. Thread-local rather than a global mutex because
    /// the shared state is `Rc`, and the UI runs on one thread.
    static REGISTRY: RefCell<HashMap<u16, Rc<RefCell<ComboState>>>> =
        RefCell::new(HashMap::new());
}

/// Look up a registered combo state by id.
///
/// Used by `Dialog` and `Application` when they see [`CM_SHOW_DROPDOWN`].
/// Returns `None` when the id was never registered or its control was dropped.
pub fn lookup(id: u16) -> Option<Rc<RefCell<ComboState>>> {
    REGISTRY.with(|r| r.borrow().get(&id).cloned())
}

/// A field showing one choice, with a drop-down list of the alternatives.
pub struct ComboBox {
    core: ViewCore,
    id: u16,
    state: Rc<RefCell<ComboState>>,
    /// Command emitted when the choice changes. Zero means none.
    on_change: CommandId,
    view_state: StateFlags,
}

impl ComboBox {
    /// Create an empty combo box registered under `id`.
    ///
    /// The id identifies this control to the drop-down popup and must be unique
    /// among the combo boxes alive at the same time. Reusing a live id replaces
    /// the earlier registration, which leaves the earlier control unable to open
    /// its list.
    pub fn new(bounds: Rect, id: u16) -> Self {
        let state = Rc::new(RefCell::new(ComboState {
            items: Vec::new(),
            selected: None,
            field: bounds,
        }));
        REGISTRY.with(|r| r.borrow_mut().insert(id, Rc::clone(&state)));
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            id,
            state,
            on_change: 0,
            view_state: State::empty(),
        }
    }

    /// Create a combo box already holding `items`, with the first selected.
    pub fn with_items(bounds: Rect, id: u16, items: Vec<String>) -> Self {
        let mut combo = Self::new(bounds, id);
        combo.set_items(items);
        combo
    }

    /// Registration id, as passed to [`ComboBox::new`].
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Shared state, for callers that want to read the choice later without
    /// holding on to the control.
    pub fn state(&self) -> Rc<RefCell<ComboState>> {
        Rc::clone(&self.state)
    }

    /// Replace the item list. Selects the first item when there was no previous
    /// choice, and clamps an existing choice to the new list.
    pub fn set_items(&mut self, items: Vec<String>) {
        let mut state = self.state.borrow_mut();
        let had_selection = state.selected.is_some();
        state.items = items;
        if !had_selection && !state.items.is_empty() {
            state.selected = Some(0);
        }
        state.clamp();
    }

    /// Append one item.
    pub fn add_item(&mut self, item: impl Into<String>) {
        let mut state = self.state.borrow_mut();
        state.items.push(item.into());
        if state.selected.is_none() {
            state.selected = Some(0);
        }
    }

    /// Number of items in the list.
    pub fn item_count(&self) -> usize {
        self.state.borrow().items.len()
    }

    /// Index of the current choice.
    pub fn selected(&self) -> Option<usize> {
        self.state.borrow().selected
    }

    /// Set the current choice. Out-of-range indices are ignored, so a stale
    /// index never silently selects the wrong item.
    pub fn set_selected(&mut self, index: Option<usize>) {
        let mut state = self.state.borrow_mut();
        match index {
            None => state.selected = None,
            Some(i) if i < state.items.len() => state.selected = Some(i),
            Some(_) => {}
        }
    }

    /// Text of the current choice.
    pub fn selected_text(&self) -> Option<String> {
        self.state.borrow().selected_text().map(str::to_string)
    }

    /// Command broadcast when the choice changes. Zero, the default, sends none.
    pub fn set_on_change(&mut self, command: CommandId) {
        self.on_change = command;
    }

    /// Move the choice by `delta` items, clamping at both ends.
    ///
    /// Returns true when the choice actually changed.
    fn step(&mut self, delta: i32) -> bool {
        let mut state = self.state.borrow_mut();
        if state.items.is_empty() {
            return false;
        }
        let last = state.items.len() as i32 - 1;
        let current = state.selected.map_or(0, |i| i as i32);
        let next = (current + delta).clamp(0, last);
        if Some(next as usize) == state.selected {
            return false;
        }
        state.selected = Some(next as usize);
        true
    }

    /// Build the command event that asks the dialog to open the list.
    ///
    /// The field rect travels in the shared state, so the event only needs to
    /// name the control.
    fn open_request(&self) -> Event {
        let mut event = Event::command(CM_SHOW_DROPDOWN);
        event.info = self.id;
        event
    }
}

impl Drop for ComboBox {
    fn drop(&mut self) {
        // Only clear the slot if it still points at this control's state; a
        // later combo box may have taken the id over.
        REGISTRY.with(|r| {
            let mut map = r.borrow_mut();
            if let Some(existing) = map.get(&self.id) {
                if Rc::ptr_eq(existing, &self.state) {
                    map.remove(&self.id);
                }
            }
        });
    }
}

impl View for ComboBox {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.core.bounds = bounds;
        self.state.borrow_mut().field = bounds;
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.view_state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.view_state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped().max(0) as usize;
        if width == 0 {
            return;
        }
        // Borland's input palette gives "normal" and "focused" the same colour,
        // because an InputLine shows focus with its cursor. These controls draw
        // no cursor, so they borrow the selected-text colour to make focus
        // visible.
        let text_attr = if self.is_focused() {
            self.map_color(INPUT_SELECTED)
        } else {
            self.map_color(INPUT_NORMAL)
        };
        let arrow_attr = self.map_color(INPUT_ARROWS);

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', text_attr, width);

        // The arrow owns the last cell; the caption gets what is left.
        let caption_width = width.saturating_sub(1);
        if caption_width > 0 {
            if let Some(text) = self.state.borrow().selected_text() {
                let shown: String = text.chars().take(caption_width).collect();
                buf.move_str(1.min(caption_width), &shown, text_attr);
            }
        }
        buf.put_char(width - 1, DROP_ARROW, arrow_attr);

        write_line_to_terminal(terminal, 0, 0, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // A click anywhere on the field opens the list, focused or not, so the
        // control behaves the way a mouse user expects on first click.
        if event.what == EventType::MouseDown && self.extent().contains(event.mouse.pos) {
            *event = self.open_request();
            return;
        }

        if !self.is_focused() || event.what != EventType::Keyboard {
            return;
        }

        let alt = event
            .key_modifiers
            .contains(crossterm::event::KeyModifiers::ALT);

        match event.key_code {
            KB_F4 => {
                *event = self.open_request();
            }
            KB_DOWN if alt => {
                *event = self.open_request();
            }
            KB_DOWN => {
                if self.step(1) {
                    let cmd = self.on_change;
                    event.clear();
                    if cmd != 0 {
                        *event = Event::broadcast_with_info(cmd, self.id);
                    }
                } else {
                    event.clear();
                }
            }
            KB_UP => {
                if self.step(-1) {
                    let cmd = self.on_change;
                    event.clear();
                    if cmd != 0 {
                        *event = Event::broadcast_with_info(cmd, self.id);
                    }
                } else {
                    event.clear();
                }
            }
            KB_HOME => {
                self.step(i32::MIN / 2);
                event.clear();
            }
            KB_END => {
                self.step(i32::MAX / 2);
                event.clear();
            }
            _ => {}
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        // The field is an input line in everything but editability.
        Some(Palette::from_slice(palettes::CP_INPUT_LINE))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Box-drawing characters for the popup frame, clockwise from the top left.
const FRAME_CHARS: [char; 6] = [
    '\u{250C}', '\u{2500}', '\u{2510}', '\u{2502}', '\u{2514}', '\u{2518}',
];

/// The modal list a combo box drops down.
///
/// Mirrors `HistoryWindow`: created and run by whoever owns the terminal, and
/// it writes the choice straight into the shared [`ComboState`]. It draws its
/// own thin frame rather than using `Window`, because a drop-down has no title,
/// no close box and nothing to resize.
pub struct DropdownWindow {
    core: ViewCore,
    state: Rc<RefCell<ComboState>>,
    /// Index highlighted in the list.
    cursor: usize,
    /// First visible item, for lists taller than the popup.
    top: usize,
    /// Rect of the item area, inside the frame, in screen coordinates.
    list_rect: Rect,
}

impl DropdownWindow {
    /// Build the popup for `state`, placed just under its field.
    ///
    /// The popup is pushed back on screen if the field sits near an edge, using
    /// `screen` as the available area.
    pub fn new(state: Rc<RefCell<ComboState>>, screen: Rect) -> Self {
        let (field, count, selected) = {
            let s = state.borrow();
            (s.field, s.items.len(), s.selected)
        };

        let rows = (count as i16).clamp(1, MAX_DROPDOWN_ROWS);
        let height = rows + 2; // frame top and bottom
        let width = field.width().max(8);

        let mut x = field.a.x;
        let mut y = field.b.y; // the row under the field
        // Flip above the field when there is no room below.
        if y + height > screen.b.y {
            y = (field.a.y - height).max(screen.a.y);
        }
        if x + width > screen.b.x {
            x = (screen.b.x - width).max(screen.a.x);
        }

        let window_bounds = Rect::new(x, y, x + width, y + height);
        // The list sits inside the frame, in the popup's own space
        let list_rect = Rect::new(1, 1, width - 1, 1 + rows);

        Self {
            core: ViewCore {
                bounds: window_bounds,
                ..ViewCore::default()
            },
            state,
            cursor: selected.unwrap_or(0),
            top: 0,
            list_rect,
        }
    }

    fn visible_rows(&self) -> usize {
        self.list_rect.height_clamped().max(0) as usize
    }

    /// Scroll so the cursor is on screen.
    fn scroll_into_view(&mut self) {
        let rows = self.visible_rows();
        if rows == 0 {
            return;
        }
        if self.cursor < self.top {
            self.top = self.cursor;
        } else if self.cursor >= self.top + rows {
            self.top = self.cursor + 1 - rows;
        }
    }

    fn move_cursor(&mut self, delta: i32) {
        let count = self.state.borrow().items.len();
        if count == 0 {
            return;
        }
        let last = count as i32 - 1;
        self.cursor = (self.cursor as i32 + delta).clamp(0, last) as usize;
        self.scroll_into_view();
    }

    /// Item index under a screen point, if the point is on the list.
    fn item_at(&self, pos: Point) -> Option<usize> {
        if !self.list_rect.contains(pos) {
            return None;
        }
        let row = (pos.y - self.list_rect.a.y) as usize;
        let idx = self.top + row;
        (idx < self.state.borrow().items.len()).then_some(idx)
    }

    /// Draw the frame and the visible items.
    fn draw_popup(&mut self, terminal: &mut Terminal) {
        let width = self.list_rect.width_clamped().max(0) as usize;
        let outer = self.core.bounds.width_clamped().max(0) as usize;
        if width == 0 || outer == 0 {
            return;
        }
        let normal = self.map_color(crate::core::palette::LISTBOX_NORMAL);
        let selected = self.map_color(crate::core::palette::LISTBOX_SELECTED);

        let [tl, horiz, tr, vert, bl, br] = FRAME_CHARS;

        // Top and bottom frame rows.
        let mut top = DrawBuffer::new(outer);
        top.move_char(0, horiz, normal, outer);
        top.put_char(0, tl, normal);
        top.put_char(outer - 1, tr, normal);
        write_line_to_terminal(terminal, 0, 0, &top);

        let mut bottom = DrawBuffer::new(outer);
        bottom.move_char(0, horiz, normal, outer);
        bottom.put_char(0, bl, normal);
        bottom.put_char(outer - 1, br, normal);
        write_line_to_terminal(
            terminal,
            0,
            self.extent().b.y - 1,
            &bottom,
        );

        let state = self.state.borrow();
        for row in 0..self.visible_rows() {
            let mut buf = DrawBuffer::new(width);
            let idx = self.top + row;
            let attr = if idx == self.cursor { selected } else { normal };
            buf.move_char(0, ' ', attr, width);
            if let Some(text) = state.items.get(idx) {
                let shown: String = text.chars().take(width).collect();
                buf.move_str(0, &shown, attr);
            }
            let y = self.list_rect.a.y + row as i16;
            // Side frame, then the item text between the edges.
            let mut edge = DrawBuffer::new(1);
            edge.put_char(0, vert, normal);
            write_line_to_terminal(terminal, 0, y, &edge);
            write_line_to_terminal(terminal, self.extent().b.x - 1, y, &edge);
            write_line_to_terminal(terminal, self.list_rect.a.x, y, &buf);
        }
    }

    /// Run the popup modally.
    ///
    /// Returns the chosen index and writes it into the shared state, or `None`
    /// when the user cancelled, leaving the state untouched.
    pub fn execute(&mut self, terminal: &mut Terminal) -> Option<usize> {
        self.scroll_into_view();
        loop {
            // Nothing owns this popup, so it pushes its own origin and
            // translates the raw screen events itself.
            terminal.push_origin(self.core.bounds.a);
            self.draw_popup(terminal);
            terminal.pop_origin();
            let _ = terminal.flush();

            let Ok(Some(mut event)) = terminal.poll_event(std::time::Duration::from_millis(50))
            else {
                continue;
            };
            let origin = self.core.bounds.a;
            event.mouse.pos.x -= origin.x;
            event.mouse.pos.y -= origin.y;

            match event.what {
                EventType::Keyboard => match event.key_code {
                    KB_ENTER => return self.commit(),
                    KB_ESC | KB_F4 => return None,
                    KB_UP => self.move_cursor(-1),
                    KB_DOWN => self.move_cursor(1),
                    KB_HOME => self.move_cursor(i32::MIN / 2),
                    KB_END => self.move_cursor(i32::MAX / 2),
                    crate::core::event::KB_PGUP => {
                        let rows = self.visible_rows() as i32;
                        self.move_cursor(-rows);
                    }
                    crate::core::event::KB_PGDN => {
                        let rows = self.visible_rows() as i32;
                        self.move_cursor(rows);
                    }
                    _ => {}
                },
                EventType::MouseDown => {
                    match self.item_at(event.mouse.pos) {
                        Some(idx) => {
                            self.cursor = idx;
                            return self.commit();
                        }
                        // A click outside the popup dismisses it, the way every
                        // other drop-down behaves.
                        None if !self.extent().contains(event.mouse.pos) => return None,
                        None => {}
                    }
                    event.clear();
                }
                EventType::MouseWheelUp => self.move_cursor(-1),
                EventType::MouseWheelDown => self.move_cursor(1),
                _ => {}
            }
        }
    }

    /// Write the cursor position into the shared state and return it.
    fn commit(&mut self) -> Option<usize> {
        let mut state = self.state.borrow_mut();
        if state.items.is_empty() {
            return None;
        }
        let idx = self.cursor.min(state.items.len() - 1);
        state.selected = Some(idx);
        Some(idx)
    }
}

impl View for DropdownWindow {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    /// The popup drives its own loop through [`DropdownWindow::execute`]; this
    /// exists so it can use `map_color`, not so it can join a view hierarchy.
    fn draw(&mut self, terminal: &mut Terminal) {
        self.draw_popup(terminal);
    }

    fn handle_event(&mut self, _event: &mut Event) {}

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_LISTBOX))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating combo boxes with a fluent API.
pub struct ComboBoxBuilder {
    bounds: Option<Rect>,
    id: u16,
    items: Vec<String>,
    selected: Option<usize>,
    on_change: CommandId,
}

impl ComboBoxBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            id: 0,
            items: Vec::new(),
            selected: None,
            on_change: 0,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn id(mut self, id: u16) -> Self {
        self.id = id;
        self
    }

    #[must_use]
    pub fn items<I: Into<String>>(mut self, items: impl IntoIterator<Item = I>) -> Self {
        self.items = items.into_iter().map(Into::into).collect();
        self
    }

    #[must_use]
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    #[must_use]
    pub fn on_change(mut self, command: CommandId) -> Self {
        self.on_change = command;
        self
    }

    pub fn build(self) -> ComboBox {
        let bounds = self.bounds.expect("ComboBox bounds must be set");
        let mut combo = ComboBox::with_items(bounds, self.id, self.items);
        if let Some(i) = self.selected {
            combo.set_selected(Some(i));
        }
        combo.set_on_change(self.on_change);
        combo
    }

    pub fn build_boxed(self) -> Box<ComboBox> {
        Box::new(self.build())
    }
}

impl Default for ComboBoxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn combo(id: u16) -> ComboBox {
        ComboBox::with_items(
            Rect::new(0, 0, 20, 1),
            id,
            vec!["one".into(), "two".into(), "three".into()],
        )
    }

    fn key(code: u16) -> Event {
        Event::keyboard(code)
    }

    #[test]
    fn first_item_is_selected_by_default() {
        let c = combo(900);
        assert_eq!(c.selected(), Some(0));
        assert_eq!(c.selected_text().as_deref(), Some("one"));
    }

    #[test]
    fn empty_combo_has_no_selection() {
        let c = ComboBox::new(Rect::new(0, 0, 10, 1), 901);
        assert_eq!(c.selected(), None);
        assert_eq!(c.selected_text(), None);
    }

    #[test]
    fn out_of_range_selection_is_ignored() {
        let mut c = combo(902);
        c.set_selected(Some(99));
        assert_eq!(
            c.selected(),
            Some(0),
            "stale index must not move the choice"
        );
    }

    #[test]
    fn shrinking_the_item_list_clamps_the_choice() {
        let mut c = combo(903);
        c.set_selected(Some(2));
        c.set_items(vec!["only".into()]);
        assert_eq!(c.selected(), Some(0));
    }

    #[test]
    fn arrows_step_the_choice_and_clamp() {
        let mut c = combo(904);
        c.set_state(State::FOCUSED);
        let mut e = key(KB_DOWN);
        c.handle_event(&mut e);
        assert_eq!(c.selected(), Some(1));
        for _ in 0..5 {
            let mut e = key(KB_DOWN);
            c.handle_event(&mut e);
        }
        assert_eq!(c.selected(), Some(2), "clamped at the last item");
        let mut e = key(KB_UP);
        c.handle_event(&mut e);
        assert_eq!(c.selected(), Some(1));
    }

    #[test]
    fn home_and_end_jump_to_the_ends() {
        let mut c = combo(905);
        c.set_state(State::FOCUSED);
        let mut e = key(KB_END);
        c.handle_event(&mut e);
        assert_eq!(c.selected(), Some(2));
        let mut e = key(KB_HOME);
        c.handle_event(&mut e);
        assert_eq!(c.selected(), Some(0));
    }

    #[test]
    fn f4_asks_the_dialog_to_open_the_list() {
        let mut c = combo(906);
        c.set_state(State::FOCUSED);
        let mut e = key(KB_F4);
        c.handle_event(&mut e);
        assert_eq!(e.what, EventType::Command);
        assert_eq!(e.command, CM_SHOW_DROPDOWN);
        assert_eq!(e.info, 906, "the popup is told which combo asked");
    }

    #[test]
    fn a_click_opens_the_list_even_when_unfocused() {
        let mut c = combo(907);
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.pos = Point::new(3, 0);
        c.handle_event(&mut e);
        assert_eq!(e.command, CM_SHOW_DROPDOWN);
    }

    #[test]
    fn keys_are_ignored_when_not_focused() {
        let mut c = combo(908);
        let mut e = key(KB_DOWN);
        c.handle_event(&mut e);
        assert_eq!(c.selected(), Some(0));
        assert_eq!(e.what, EventType::Keyboard, "event left for other views");
    }

    #[test]
    fn on_change_command_is_broadcast_when_the_choice_moves() {
        let mut c = combo(909);
        c.set_state(State::FOCUSED);
        c.set_on_change(777);
        let mut e = key(KB_DOWN);
        c.handle_event(&mut e);
        assert_eq!(e.what, EventType::Broadcast);
        assert_eq!(e.command, 777);

        // Already at the last item: no change, so no broadcast.
        c.set_selected(Some(2));
        let mut e = key(KB_DOWN);
        c.handle_event(&mut e);
        assert_eq!(e.what, EventType::Nothing);
    }

    #[test]
    fn state_is_registered_for_the_popup_and_freed_on_drop() {
        {
            let c = combo(910);
            let found = lookup(910).expect("registered while alive");
            assert_eq!(found.borrow().items.len(), 3);
            drop(c);
        }
        assert!(lookup(910).is_none(), "registration freed on drop");
    }

    #[test]
    fn moving_the_control_moves_the_popup_anchor() {
        let mut c = combo(911);
        c.set_bounds(Rect::new(5, 9, 25, 10));
        assert_eq!(lookup(911).unwrap().borrow().field, Rect::new(5, 9, 25, 10));
    }

    #[test]
    fn popup_sits_under_the_field() {
        let c = combo(912);
        let screen = Rect::new(0, 0, 80, 25);
        let popup = DropdownWindow::new(c.state(), screen);
        assert_eq!(popup.bounds().a, Point::new(0, 1), "just below");
        assert_eq!(popup.visible_rows(), 3, "one row per item");
    }

    #[test]
    fn popup_flips_above_a_field_near_the_bottom() {
        let mut c = combo(913);
        c.set_bounds(Rect::new(0, 23, 20, 24));
        let screen = Rect::new(0, 0, 80, 25);
        let popup = DropdownWindow::new(c.state(), screen);
        assert!(
            popup.bounds().b.y <= 23,
            "popup must not run off the bottom: {:?}",
            popup.bounds()
        );
    }

    #[test]
    fn popup_height_is_capped_for_long_lists() {
        let mut c = combo(914);
        c.set_items((0..50).map(|i| format!("item {i}")).collect());
        let popup = DropdownWindow::new(c.state(), Rect::new(0, 0, 80, 25));
        assert_eq!(popup.visible_rows(), MAX_DROPDOWN_ROWS as usize);
    }

    #[test]
    fn popup_scrolls_to_keep_the_cursor_visible() {
        let mut c = combo(915);
        c.set_items((0..20).map(|i| format!("item {i}")).collect());
        let mut popup = DropdownWindow::new(c.state(), Rect::new(0, 0, 80, 25));
        popup.move_cursor(15);
        assert!(popup.top > 0, "list scrolled");
        assert!(popup.cursor >= popup.top && popup.cursor < popup.top + popup.visible_rows());
    }

    #[test]
    fn popup_commit_writes_the_choice_back() {
        let c = combo(916);
        let mut popup = DropdownWindow::new(c.state(), Rect::new(0, 0, 80, 25));
        popup.move_cursor(2);
        assert_eq!(popup.commit(), Some(2));
        assert_eq!(c.selected(), Some(2));
    }

    #[test]
    fn popup_click_maps_to_the_right_item() {
        let c = combo(917);
        let popup = DropdownWindow::new(c.state(), Rect::new(0, 0, 80, 25));
        let inside = Point::new(popup.list_rect.a.x, popup.list_rect.a.y + 1);
        assert_eq!(popup.item_at(inside), Some(1));
        assert_eq!(popup.item_at(Point::new(0, 0)), None, "outside the list");
    }
}
