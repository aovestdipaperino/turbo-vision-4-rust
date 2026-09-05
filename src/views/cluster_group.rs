// (C) 2025 - Enzo Lombardi

//! CheckBoxes and RadioButtons - one focusable control holding several items.
//!
//! This is the Borland shape the port was missing. `TCheckBoxes` and
//! `TRadioButtons` each hold a list of items in a single view with one bitmask
//! value; the focus lands on the cluster, and the arrow keys move within it.
//!
//! The existing [`CheckBox`](super::checkbox::CheckBox) and
//! [`RadioButton`](super::radiobutton::RadioButton) hold one label each, and a
//! radio group is emulated by broadcasting on a group id. That works and stays
//! supported, but it costs one focus stop per item and one broadcast per
//! selection. Reach for these when you want a group to behave as a unit.
//!
//! # Keys
//!
//! | Key | Action |
//! |-----|--------|
//! | Up, Down | Move within the cluster |
//! | Home, End | First or last item |
//! | Space | Toggle the item, or select it in a radio group |
//! | Alt+letter | The item whose label marks that letter with tildes |
//!
//! Tab still leaves the cluster, because the whole cluster is one focus stop.
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::views::cluster_group::CheckBoxes;
//! use turbo_vision::core::geometry::Rect;
//!
//! let mut boxes = CheckBoxes::new(
//!     Rect::new(2, 2, 24, 5),
//!     vec!["~B~old".into(), "~I~talic".into(), "~U~nderline".into()],
//! );
//! boxes.set_checked(0, true);
//! boxes.set_checked(2, true);
//! assert_eq!(boxes.value(), 0b101);
//! ```

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_DOWN, KB_END, KB_HOME, KB_UP, MB_LEFT_BUTTON};
use crate::core::geometry::{Point, Rect};
use crate::core::palette::{
    Attr, CLUSTER_DISABLED, CLUSTER_FOCUSED, CLUSTER_NORMAL, CLUSTER_SHORTCUT,
};
use crate::core::state::{SF_FOCUSED, StateFlags};
use crate::terminal::Terminal;

/// Key code for the space bar, which toggles or selects an item.
const KB_SPACE: u16 = b' ' as u16;

/// Cells a marker such as `[X] ` or `( ) ` occupies before the label.
const MARKER_WIDTH: usize = 4;

/// The largest number of items a cluster can hold, one per bit of the value.
pub const MAX_CLUSTER_ITEMS: usize = 32;

/// One item: its drawn label plus the hotkey pulled out of the tildes.
#[derive(Debug, Clone)]
struct Item {
    /// Label with the tilde markers stripped.
    label: String,
    /// The letter between tildes, lowercased.
    hotkey: Option<char>,
    /// Where that letter sits within `label`.
    hotkey_pos: Option<usize>,
    /// Disabled items are drawn dimmed and cannot be chosen.
    enabled: bool,
}

/// Split `"~B~old"` into its drawn label, hotkey letter and the letter's index.
///
/// A label with no tildes, or an unterminated one, simply has no hotkey.
fn parse_label(text: &str) -> Item {
    let mut label = String::new();
    let mut hotkey = None;
    let mut hotkey_pos = None;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '~' {
            label.push(ch);
            continue;
        }
        if let Some(letter) = chars.next() {
            if hotkey.is_none() {
                hotkey = Some(letter.to_ascii_lowercase());
                hotkey_pos = Some(label.chars().count());
            }
            label.push(letter);
            if chars.peek() == Some(&'~') {
                chars.next();
            }
        }
    }

    Item {
        label,
        hotkey,
        hotkey_pos,
        enabled: true,
    }
}

/// What the marker before each label looks like, and how selection behaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Square brackets; any number of items can be on at once.
    Check,
    /// Round brackets; exactly one item is on.
    Radio,
}

/// The machinery shared by [`CheckBoxes`] and [`RadioButtons`].
struct ClusterGroup {
    core: ViewCore,
    kind: Kind,
    items: Vec<Item>,
    /// One bit per item. A radio cluster keeps exactly one bit set.
    value: u32,
    /// Item the arrow keys are on.
    focused_item: usize,
    /// Command broadcast when the value changes. Zero means none.
    on_change: CommandId,
    view_state: StateFlags,
}

impl ClusterGroup {
    fn new(bounds: Rect, kind: Kind, labels: Vec<String>) -> Self {
        let mut group = Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            kind,
            items: Vec::new(),
            value: 0,
            focused_item: 0,
            on_change: 0,
            view_state: 0,
        };
        group.set_labels(labels);
        if kind == Kind::Radio && !group.items.is_empty() {
            group.value = 1;
        }
        group
    }

    /// Replace the labels. Items past [`MAX_CLUSTER_ITEMS`] are dropped, since
    /// the value has one bit each and a silently half-stored item would be
    /// worse than a missing one.
    fn set_labels(&mut self, labels: Vec<String>) {
        self.items = labels
            .iter()
            .take(MAX_CLUSTER_ITEMS)
            .map(|l| parse_label(l))
            .collect();
        self.focused_item = self.focused_item.min(self.items.len().saturating_sub(1));
        self.value &= self.item_mask();
        if self.kind == Kind::Radio && self.value == 0 && !self.items.is_empty() {
            self.value = 1;
        }
    }

    /// Bits that correspond to real items.
    fn item_mask(&self) -> u32 {
        if self.items.len() >= MAX_CLUSTER_ITEMS {
            u32::MAX
        } else {
            (1u32 << self.items.len()) - 1
        }
    }

    fn is_set(&self, index: usize) -> bool {
        index < self.items.len() && self.value & (1 << index) != 0
    }

    /// Turn one item on or off. Out-of-range indices are ignored.
    ///
    /// Returns true when the value changed.
    fn set_bit(&mut self, index: usize, on: bool) -> bool {
        if index >= self.items.len() {
            return false;
        }
        let before = self.value;
        match (self.kind, on) {
            // A radio cluster holds exactly one bit, so selecting clears the rest.
            (Kind::Radio, true) => self.value = 1 << index,
            // Turning the only radio item off would leave nothing selected.
            (Kind::Radio, false) => {}
            (Kind::Check, true) => self.value |= 1 << index,
            (Kind::Check, false) => self.value &= !(1 << index),
        }
        self.value != before
    }

    /// Act on one item the way Space would: toggle a check, select a radio.
    fn activate(&mut self, index: usize) -> bool {
        if !self.items.get(index).is_some_and(|i| i.enabled) {
            return false;
        }
        match self.kind {
            Kind::Check => {
                let on = self.is_set(index);
                self.set_bit(index, !on)
            }
            Kind::Radio => self.set_bit(index, true),
        }
    }

    /// Move the focused item by `delta`, clamping at both ends.
    fn move_focus(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let last = self.items.len() as i32 - 1;
        self.focused_item = (self.focused_item as i32 + delta).clamp(0, last) as usize;
    }

    /// Index of the item whose hotkey is `letter`.
    fn item_for_hotkey(&self, letter: char) -> Option<usize> {
        let letter = letter.to_ascii_lowercase();
        self.items
            .iter()
            .position(|i| i.enabled && i.hotkey == Some(letter))
    }

    /// Item under a screen point, if the point is on one.
    fn item_at(&self, pos: Point) -> Option<usize> {
        if !self.core.bounds.contains(pos) {
            return None;
        }
        let row = (pos.y - self.core.bounds.a.y) as usize;
        (row < self.items.len()).then_some(row)
    }

    /// The marker drawn before an item's label.
    fn marker(&self, index: usize) -> &'static str {
        match (self.kind, self.is_set(index)) {
            (Kind::Check, true) => "[X] ",
            (Kind::Check, false) => "[ ] ",
            (Kind::Radio, true) => "(\u{2022}) ",
            (Kind::Radio, false) => "( ) ",
        }
    }

    fn is_focused_view(&self) -> bool {
        self.view_state & SF_FOCUSED != 0
    }

    fn draw_group(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped().max(0) as usize;
        let height = self.core.bounds.height_clamped().max(0) as usize;
        if width == 0 || height == 0 {
            return;
        }
        let normal = self.map_color(CLUSTER_NORMAL);
        let focused = self.map_color(CLUSTER_FOCUSED);
        let shortcut = self.map_color(CLUSTER_SHORTCUT);
        let disabled = self.map_color(CLUSTER_DISABLED);

        for row in 0..height {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', normal, width);

            if let Some(item) = self.items.get(row) {
                // Only the focused item of a focused cluster is highlighted;
                // the cluster is one focus stop, so the rest stay plain.
                let attr: Attr = if !item.enabled {
                    disabled
                } else if self.is_focused_view() && row == self.focused_item {
                    focused
                } else {
                    normal
                };
                buf.move_str(0, self.marker(row), attr);

                if MARKER_WIDTH < width {
                    let room = width - MARKER_WIDTH;
                    let shown: String = item.label.chars().take(room).collect();
                    buf.move_str(MARKER_WIDTH, &shown, attr);
                    // Repaint just the hotkey letter, unless the item is
                    // disabled, where a highlight would invite a click.
                    if item.enabled {
                        if let Some(pos) = item.hotkey_pos {
                            if let Some(letter) = item.label.chars().nth(pos) {
                                if MARKER_WIDTH + pos < width {
                                    buf.put_char(MARKER_WIDTH + pos, letter, shortcut);
                                }
                            }
                        }
                    }
                }
            }

            write_line_to_terminal(
                terminal,
                self.core.bounds.a.x,
                self.core.bounds.a.y + row as i16,
                &buf,
            );
        }
    }

    /// Turn a value change into the outgoing event.
    fn report(&self, event: &mut Event, changed: bool) {
        if changed && self.on_change != 0 {
            *event = Event::broadcast(self.on_change);
        } else {
            event.clear();
        }
    }

    fn handle(&mut self, event: &mut Event) {
        if event.what == EventType::MouseDown && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
            if let Some(index) = self.item_at(event.mouse.pos) {
                self.focused_item = index;
                let changed = self.activate(index);
                self.report(event, changed);
            }
            return;
        }

        if event.what != EventType::Keyboard {
            return;
        }

        // Hotkeys work whether or not the cluster holds the focus, which is how
        // a dialog's Alt shortcuts are expected to behave.
        if event
            .key_modifiers
            .contains(crossterm::event::KeyModifiers::ALT)
        {
            let letter = (event.key_code & 0xFF) as u8 as char;
            if let Some(index) = self.item_for_hotkey(letter) {
                self.focused_item = index;
                let changed = self.activate(index);
                self.report(event, changed);
                return;
            }
        }

        if !self.is_focused_view() {
            return;
        }

        match event.key_code {
            KB_UP => self.move_focus(-1),
            KB_DOWN => self.move_focus(1),
            KB_HOME => self.move_focus(i32::MIN / 2),
            KB_END => self.move_focus(i32::MAX / 2),
            KB_SPACE => {
                let index = self.focused_item;
                let changed = self.activate(index);
                self.report(event, changed);
                return;
            }
            // Not ours: Tab, Enter and the dialog's own keys must get through.
            _ => return,
        }
        event.clear();
    }
}

impl View for ClusterGroup {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
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
        self.draw_group(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.handle(event);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_CLUSTER))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Generates the shared surface of [`CheckBoxes`] and [`RadioButtons`].
///
/// Both wrap the same machinery and differ only in their marker and in whether
/// more than one item can be on, so the API is written once here rather than
/// copied twice with one word changed.
macro_rules! cluster_control {
    ($name:ident, $kind:expr, $doc:literal) => {
        #[doc = $doc]
        pub struct $name {
            inner: ClusterGroup,
        }

        impl $name {
            /// Create the cluster from labels, one per row.
            ///
            /// A tilde-wrapped letter, as in `"~B~old"`, becomes that item's Alt
            /// hotkey. At most [`MAX_CLUSTER_ITEMS`] labels are kept.
            pub fn new(bounds: Rect, labels: Vec<String>) -> Self {
                Self {
                    inner: ClusterGroup::new(bounds, $kind, labels),
                }
            }

            /// Replace the labels, keeping the value bits that still apply.
            pub fn set_labels(&mut self, labels: Vec<String>) {
                self.inner.set_labels(labels);
            }

            /// Number of items.
            pub fn item_count(&self) -> usize {
                self.inner.items.len()
            }

            /// The raw bitmask: bit *n* is item *n*.
            pub fn value(&self) -> u32 {
                self.inner.value
            }

            /// Set the raw bitmask. Bits past the last item are dropped.
            pub fn set_value(&mut self, value: u32) {
                self.inner.value = value & self.inner.item_mask();
            }

            /// Index of the item the arrow keys are on.
            pub fn focused_item(&self) -> usize {
                self.inner.focused_item
            }

            /// Move the arrow-key focus within the cluster.
            pub fn set_focused_item(&mut self, index: usize) {
                if index < self.inner.items.len() {
                    self.inner.focused_item = index;
                }
            }

            /// Whether one item can be chosen. Disabled items draw dimmed.
            pub fn set_enabled(&mut self, index: usize, enabled: bool) {
                if let Some(item) = self.inner.items.get_mut(index) {
                    item.enabled = enabled;
                }
            }

            /// Whether one item can be chosen.
            pub fn is_enabled(&self, index: usize) -> bool {
                self.inner.items.get(index).is_some_and(|i| i.enabled)
            }

            /// Command broadcast when the value changes. Zero, the default,
            /// sends none.
            pub fn set_on_change(&mut self, command: CommandId) {
                self.inner.on_change = command;
            }
        }

        impl View for $name {
            fn core(&self) -> &ViewCore {
                self.inner.core()
            }

            fn core_mut(&mut self) -> &mut ViewCore {
                self.inner.core_mut()
            }

            fn state(&self) -> StateFlags {
                self.inner.state()
            }

            fn set_state(&mut self, state: StateFlags) {
                // `ClusterGroup::set_state` has focus bookkeeping the default skips.
                self.inner.set_state(state);
            }

            fn can_focus(&self) -> bool {
                true
            }

            fn draw(&mut self, terminal: &mut Terminal) {
                self.inner.draw(terminal);
            }

            fn handle_event(&mut self, event: &mut Event) {
                self.inner.handle_event(event);
            }

            fn set_palette_chain(
                &mut self,
                node: Option<crate::core::palette_chain::PaletteChainNode>,
            ) {
                self.inner.set_palette_chain(node);
            }

            fn get_palette(&self) -> Option<crate::core::palette::Palette> {
                self.inner.get_palette()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}

cluster_control!(
    CheckBoxes,
    Kind::Check,
    "Several check boxes in one focusable control, with one bit of `value` each.\n\nMatches Borland: `TCheckBoxes`."
);

cluster_control!(
    RadioButtons,
    Kind::Radio,
    "Several radio buttons in one focusable control, exactly one of them on.\n\nMatches Borland: `TRadioButtons`."
);

impl CheckBoxes {
    /// Whether one box is ticked.
    pub fn is_checked(&self, index: usize) -> bool {
        self.inner.is_set(index)
    }

    /// Tick or untick one box.
    pub fn set_checked(&mut self, index: usize, checked: bool) {
        self.inner.set_bit(index, checked);
    }

    /// The ticked boxes, in order.
    pub fn checked_items(&self) -> Vec<usize> {
        (0..self.inner.items.len())
            .filter(|&i| self.inner.is_set(i))
            .collect()
    }
}

impl RadioButtons {
    /// Index of the selected button, or `None` when the cluster is empty.
    pub fn selected(&self) -> Option<usize> {
        if self.inner.items.is_empty() {
            return None;
        }
        Some(self.inner.value.trailing_zeros() as usize)
    }

    /// Select one button. Out-of-range indices are ignored.
    pub fn set_selected(&mut self, index: usize) {
        self.inner.set_bit(index, true);
    }

    /// Text of the selected button.
    pub fn selected_label(&self) -> Option<&str> {
        self.inner.items.get(self.selected()?).map(|i| &*i.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn labels() -> Vec<String> {
        vec!["~B~old".into(), "~I~talic".into(), "~U~nderline".into()]
    }

    fn boxes() -> CheckBoxes {
        let mut c = CheckBoxes::new(Rect::new(0, 0, 20, 3), labels());
        c.set_state(SF_FOCUSED);
        c
    }

    fn radios() -> RadioButtons {
        let mut r = RadioButtons::new(Rect::new(0, 0, 20, 3), labels());
        r.set_state(SF_FOCUSED);
        r
    }

    fn key(code: u16) -> Event {
        Event::keyboard(code)
    }

    fn alt(letter: char) -> Event {
        let mut e = Event::keyboard(letter as u16);
        e.key_modifiers = KeyModifiers::ALT;
        e
    }

    fn click(row: i16) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.pos = Point::new(1, row);
        e
    }

    #[test]
    fn labels_lose_their_tilde_markers() {
        let item = parse_label("~B~old");
        assert_eq!(item.label, "Bold");
        assert_eq!(item.hotkey, Some('b'));
        assert_eq!(item.hotkey_pos, Some(0));
    }

    #[test]
    fn a_label_without_tildes_has_no_hotkey() {
        let item = parse_label("Plain");
        assert_eq!(item.label, "Plain");
        assert_eq!(item.hotkey, None);
    }

    #[test]
    fn check_boxes_start_empty() {
        let c = boxes();
        assert_eq!(c.value(), 0);
        assert_eq!(c.checked_items(), Vec::<usize>::new());
    }

    #[test]
    fn each_box_owns_one_bit() {
        let mut c = boxes();
        c.set_checked(0, true);
        c.set_checked(2, true);
        assert_eq!(c.value(), 0b101);
        assert_eq!(c.checked_items(), vec![0, 2]);
        assert!(c.is_checked(0));
        assert!(!c.is_checked(1));
    }

    #[test]
    fn space_toggles_the_focused_box() {
        let mut c = boxes();
        let mut e = key(KB_SPACE);
        c.handle_event(&mut e);
        assert_eq!(c.value(), 0b001);
        let mut e = key(KB_SPACE);
        c.handle_event(&mut e);
        assert_eq!(c.value(), 0, "toggled back off");
    }

    #[test]
    fn arrows_move_within_the_cluster() {
        let mut c = boxes();
        c.handle_event(&mut key(KB_DOWN));
        assert_eq!(c.focused_item(), 1);
        c.handle_event(&mut key(KB_SPACE));
        assert_eq!(c.value(), 0b010, "the second box, not the first");

        for _ in 0..5 {
            c.handle_event(&mut key(KB_DOWN));
        }
        assert_eq!(c.focused_item(), 2, "clamped at the last item");
    }

    #[test]
    fn home_and_end_jump_within_the_cluster() {
        let mut c = boxes();
        c.handle_event(&mut key(KB_END));
        assert_eq!(c.focused_item(), 2);
        c.handle_event(&mut key(KB_HOME));
        assert_eq!(c.focused_item(), 0);
    }

    #[test]
    fn tab_and_enter_are_left_for_the_dialog() {
        let mut c = boxes();
        for code in [crate::core::event::KB_TAB, crate::core::event::KB_ENTER] {
            let mut e = key(code);
            c.handle_event(&mut e);
            assert_eq!(
                e.what,
                EventType::Keyboard,
                "key {code:#x} must pass through"
            );
        }
    }

    #[test]
    fn hotkeys_work_without_the_focus() {
        let mut c = CheckBoxes::new(Rect::new(0, 0, 20, 3), labels());
        let mut e = alt('u');
        c.handle_event(&mut e);
        assert_eq!(c.value(), 0b100, "Alt+U ticked Underline");
        assert_eq!(e.what, EventType::Nothing);
    }

    #[test]
    fn an_unknown_hotkey_passes_through() {
        let mut c = boxes();
        let mut e = alt('z');
        c.handle_event(&mut e);
        assert_eq!(c.value(), 0);
        assert_eq!(e.what, EventType::Keyboard);
    }

    #[test]
    fn clicking_a_row_toggles_that_item() {
        let mut c = boxes();
        c.handle_event(&mut click(1));
        assert_eq!(c.value(), 0b010);
        assert_eq!(c.focused_item(), 1, "the click also moved the focus");
    }

    #[test]
    fn clicking_past_the_last_item_does_nothing() {
        let mut c = boxes();
        let mut e = click(2);
        e.mouse.pos = Point::new(1, 9);
        c.handle_event(&mut e);
        assert_eq!(c.value(), 0);
    }

    #[test]
    fn disabled_items_cannot_be_chosen() {
        let mut c = boxes();
        c.set_enabled(1, false);
        assert!(!c.is_enabled(1));
        c.handle_event(&mut click(1));
        assert_eq!(c.value(), 0);
        // Its hotkey is inert too.
        c.handle_event(&mut alt('i'));
        assert_eq!(c.value(), 0);
    }

    #[test]
    fn on_change_broadcasts_only_on_a_real_change() {
        let mut c = boxes();
        c.set_on_change(321);
        let mut e = key(KB_SPACE);
        c.handle_event(&mut e);
        assert_eq!(e.what, EventType::Broadcast);
        assert_eq!(e.command, 321);

        c.set_enabled(0, false);
        let mut e = key(KB_SPACE);
        c.handle_event(&mut e);
        assert_eq!(e.what, EventType::Nothing, "disabled, so nothing changed");
    }

    #[test]
    fn setting_the_value_drops_bits_past_the_last_item() {
        let mut c = boxes();
        c.set_value(0xFFFF);
        assert_eq!(c.value(), 0b111, "three items, three bits");
    }

    #[test]
    fn shrinking_the_label_list_drops_stale_bits() {
        let mut c = boxes();
        c.set_value(0b111);
        c.set_labels(vec!["only".into()]);
        assert_eq!(c.value(), 0b1);
        assert_eq!(c.focused_item(), 0);
    }

    #[test]
    fn a_cluster_is_capped_at_the_value_width() {
        let many: Vec<String> = (0..40).map(|i| format!("item{i}")).collect();
        let c = CheckBoxes::new(Rect::new(0, 0, 20, 40), many);
        assert_eq!(c.item_count(), MAX_CLUSTER_ITEMS);
    }

    // --- Radio buttons ---------------------------------------------------

    #[test]
    fn a_radio_cluster_starts_on_its_first_button() {
        let r = radios();
        assert_eq!(r.selected(), Some(0));
        assert_eq!(r.selected_label(), Some("Bold"));
        assert_eq!(r.value(), 0b001);
    }

    #[test]
    fn an_empty_radio_cluster_has_no_selection() {
        let r = RadioButtons::new(Rect::new(0, 0, 20, 3), vec![]);
        assert_eq!(r.selected(), None);
    }

    #[test]
    fn selecting_a_radio_button_clears_the_others() {
        let mut r = radios();
        r.set_selected(2);
        assert_eq!(r.value(), 0b100, "exactly one bit");
        assert_eq!(r.selected(), Some(2));
    }

    #[test]
    fn space_selects_rather_than_toggles() {
        let mut r = radios();
        r.handle_event(&mut key(KB_DOWN));
        r.handle_event(&mut key(KB_SPACE));
        assert_eq!(r.selected(), Some(1));
        // Pressing again must not turn it off: something is always selected.
        r.handle_event(&mut key(KB_SPACE));
        assert_eq!(r.selected(), Some(1));
    }

    #[test]
    fn a_radio_hotkey_selects_its_button() {
        let mut r = radios();
        r.handle_event(&mut alt('u'));
        assert_eq!(r.selected(), Some(2));
    }

    #[test]
    fn shrinking_a_radio_cluster_keeps_something_selected() {
        let mut r = radios();
        r.set_selected(2);
        r.set_labels(vec!["one".into(), "two".into()]);
        assert_eq!(r.selected(), Some(0), "the stale bit fell away");
    }
}
