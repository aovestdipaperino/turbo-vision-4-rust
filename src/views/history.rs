// (C) 2025 - Enzo Lombardi

//! History view - dropdown button control for accessing input line history.
// History - Dropdown button for InputLine history
//
// Matches Borland: THistory (history dropdown button)
//
// A small button (shows '▼') attached to the right side of an InputLine.
// When clicked, displays a HistoryWindow with previous entries.
//
// Usage:
//   let data = Rc::new(RefCell::new(String::new()));
//   let input = InputLine::new(bounds, 255, Rc::clone(&data));
//   let history = History::new(Point::new(x, y), history_id, Rc::clone(&data));
//   // Position it to the right of the InputLine

use super::group::GroupLike;
use super::handle::Handle;
use super::input_line::InputLine;
use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CM_SHOW_HISTORY;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::geometry::{Point, Rect};
use crate::core::history::HistoryManager;
use crate::terminal::Terminal;

/// History - Dropdown button for accessing input history
///
/// Matches Borland: THistory. The button is linked to an `InputLine` by a
/// typed [`Handle`] (Borland: the `link` pointer):
/// - When the owning dialog is accepted with OK, [`record_history_in`] adds
///   the linked input's text to the history list (Borland: cmRecordHistory).
/// - On click, the event is converted into a `CM_SHOW_HISTORY` command (history
///   id in `event.info`) so the owning dialog/application can open the popup.
/// - After the popup, [`apply_history_selection`] copies the choice back into
///   the linked input.
///
/// A child cannot reach a sibling on its own, so both steps run in the owner,
/// which resolves the handle through its child list.
pub struct History {
    core: ViewCore,
    history_id: u16,
    /// The input this button records and fills (Borland: `THistory::link`).
    link: Handle<InputLine>,
}

impl History {
    /// Create a new history button linked to an `InputLine` in the same group.
    ///
    /// The button is 2 characters wide (shows '▼' or similar).
    pub fn new(pos: Point, history_id: u16, link: Handle<InputLine>) -> Self {
        Self {
            core: ViewCore {
                bounds: Rect::new(pos.x, pos.y, pos.x + 2, pos.y + 1),
                state: 0,
                palette_chain: None,
                ..ViewCore::default()
            },
            history_id,
            link,
        }
    }

    /// Check if this history list has any items
    pub fn has_items(&self) -> bool {
        HistoryManager::has_history(self.history_id)
    }

    /// The history list id this button is attached to
    pub fn history_id(&self) -> u16 {
        self.history_id
    }

    /// The input this button is linked to.
    pub fn link(&self) -> Handle<InputLine> {
        self.link
    }
}

/// Record the linked input text of every `History` in `group`, recursing into
/// nested groups (Borland: `THistory` handling `cmRecordHistory` through its
/// `link` pointer). `Dialog` calls this when it is accepted with OK or Yes.
pub(crate) fn record_history_in(group: &dyn GroupLike) {
    for i in 0..group.child_count() {
        let child = group.child_at(i);
        if let Some(history) = child.as_any().downcast_ref::<History>() {
            let text = group
                .child_by_id(history.link().id())
                .and_then(|v| v.as_any().downcast_ref::<InputLine>())
                .map(|input| input.text().to_string());
            if let Some(text) = text.filter(|t| !t.is_empty()) {
                HistoryManager::add(history.history_id(), text);
            }
        } else if let Some(nested) = child.as_group() {
            record_history_in(nested);
        }
    }
}

/// Copy `item`, picked in the history popup for `history_id`, into the input
/// linked to the matching `History` button. Returns whether a target was found.
pub(crate) fn apply_history_selection(
    group: &mut dyn GroupLike,
    history_id: u16,
    item: &str,
) -> bool {
    let mut target = None;
    for i in 0..group.child_count() {
        if let Some(history) = group.child_at(i).as_any().downcast_ref::<History>() {
            if history.history_id() == history_id {
                target = Some(history.link().id());
                break;
            }
        }
    }
    if let Some(id) = target {
        if let Some(input) = group
            .child_by_id_mut(id)
            .and_then(|v| v.as_any_mut().downcast_mut::<InputLine>())
        {
            input.set_text(item);
            return true;
        }
    }
    for i in 0..group.child_count() {
        if let Some(nested) = group.child_at_mut(i).as_group_mut() {
            if apply_history_selection(nested, history_id, item) {
                return true;
            }
        }
    }
    false
}

impl View for History {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let mut buf = DrawBuffer::new(2);

        // Draw down arrow: ▼ (or use 'v' for ASCII-only)
        let arrow = if self.has_items() { "▼" } else { " " };

        use crate::core::palette::colors::{BUTTON_NORMAL, BUTTON_SELECTED};
        let color = if self.is_focused() {
            BUTTON_SELECTED
        } else {
            BUTTON_NORMAL
        };

        buf.move_str(0, arrow, color);

        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseDown => {
                if self.core.bounds.contains(event.mouse.pos)
                    && event.mouse.buttons & MB_LEFT_BUTTON != 0
                {
                    if self.has_items() {
                        // Convert the click into a CM_SHOW_HISTORY command so the
                        // owning dialog/application (which has terminal access)
                        // can open the popup. Mouse position is preserved so the
                        // popup can be placed near the button.
                        event.what = EventType::Command;
                        event.command = CM_SHOW_HISTORY;
                        event.info = self.history_id;
                    } else {
                        event.clear();
                    }
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        false // History button doesn't take focus
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_HISTORY))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating history buttons with a fluent API.
pub struct HistoryBuilder {
    pos: Option<Point>,
    history_id: Option<u16>,
    link: Option<Handle<InputLine>>,
}

impl HistoryBuilder {
    pub fn new() -> Self {
        Self {
            pos: None,
            history_id: None,
            link: None,
        }
    }

    /// Sets the linked InputLine shared data (required).
    #[must_use]
    pub fn link(mut self, link: Handle<InputLine>) -> Self {
        self.link = Some(link);
        self
    }

    #[must_use]
    pub fn pos(mut self, pos: Point) -> Self {
        self.pos = Some(pos);
        self
    }

    #[must_use]
    pub fn history_id(mut self, history_id: u16) -> Self {
        self.history_id = Some(history_id);
        self
    }

    pub fn build(self) -> History {
        let pos = self.pos.expect("History pos must be set");
        let history_id = self.history_id.expect("History history_id must be set");
        let link = self.link.expect("History link must be set");
        History::new(pos, history_id, link)
    }

    pub fn build_boxed(self) -> Box<History> {
        Box::new(self.build())
    }
}

impl Default for HistoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::group::Group;
    use crate::views::view::ViewId;

    /// A group holding an input with `text` and a History linked to it.
    fn group_with(text: &str, history_id: u16) -> (Group, Handle<InputLine>) {
        let mut group = Group::new(Rect::new(0, 0, 40, 10));
        let mut input = InputLine::new(Rect::new(2, 5, 20, 6), 32);
        input.set_text(text);
        let input = group.add_typed(input);
        group.add(History::new(Point::new(20, 5), history_id, input));
        (group, input)
    }

    #[test]
    fn test_history_button_creation() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();

        let button = History::new(Point::new(20, 5), 1, Handle::from_id(ViewId::new()));
        assert!(!button.has_items());
        assert_eq!(button.bounds().width(), 2);
    }

    #[test]
    fn test_history_button_with_items() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();
        HistoryManager::add(2, "test".to_string());

        let button = History::new(Point::new(20, 5), 2, Handle::from_id(ViewId::new()));
        assert!(button.has_items());
    }

    #[test]
    fn record_history_in_records_the_linked_input_text() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();

        let (mut group, input) = group_with("hello world", 3);
        record_history_in(&group);
        assert_eq!(HistoryManager::get_list(3), vec!["hello world".to_string()]);

        // Empty input records nothing
        group.get_mut(input).unwrap().set_text("");
        record_history_in(&group);
        assert_eq!(HistoryManager::count(3), 1);
    }

    #[test]
    fn test_click_converts_to_show_history_command() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();
        HistoryManager::add(4, "entry".to_string());

        let mut button = History::new(Point::new(20, 5), 4, Handle::from_id(ViewId::new()));
        let mut event = Event::mouse(
            EventType::MouseDown,
            Point::new(20, 5),
            MB_LEFT_BUTTON,
            false,
        );
        button.handle_event(&mut event);

        assert_eq!(event.what, EventType::Command);
        assert_eq!(event.command, CM_SHOW_HISTORY);
        assert_eq!(event.info, 4);
        // Mouse position preserved so the popup can be placed near the button
        assert_eq!(event.mouse.pos, Point::new(20, 5));
    }

    #[test]
    fn test_click_with_empty_history_is_consumed() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();

        let mut button = History::new(Point::new(20, 5), 5, Handle::from_id(ViewId::new()));
        let mut event = Event::mouse(
            EventType::MouseDown,
            Point::new(21, 5),
            MB_LEFT_BUTTON,
            false,
        );
        button.handle_event(&mut event);
        assert_eq!(event.what, EventType::Nothing);
    }

    #[test]
    fn apply_history_selection_fills_the_linked_input() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();

        let (mut group, input) = group_with("current", 6);
        assert!(apply_history_selection(&mut group, 6, "picked"));
        assert_eq!(group.get(input).unwrap().text(), "picked");

        // A different history id finds no target and changes nothing
        assert!(!apply_history_selection(&mut group, 7, "other"));
        assert_eq!(group.get(input).unwrap().text(), "picked");
    }
}
