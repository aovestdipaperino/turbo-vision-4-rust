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

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::{CM_HISTORY_SELECTED, CM_RECORD_HISTORY, CM_SHOW_HISTORY};
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::geometry::{Point, Rect};
use crate::core::history::HistoryManager;
use crate::terminal::Terminal;
use std::cell::RefCell;
use std::rc::Rc;

/// History - Dropdown button for accessing input history
///
/// Matches Borland: THistory. The button is linked to an InputLine by sharing
/// its `Rc<RefCell<String>>` data:
/// - On a `CM_RECORD_HISTORY` broadcast (sent by `Dialog` when it is accepted
///   with OK), the current input text is added to the history list.
/// - On click, the event is converted into a `CM_SHOW_HISTORY` command (history
///   id in `event.info`) so the owning dialog/application can open the popup.
/// - On a `CM_HISTORY_SELECTED` broadcast for this history id, the most recent
///   history item is copied back into the linked input data.
pub struct History {
    core: ViewCore,
    history_id: u16,
    /// Shared data of the linked InputLine (same Rc passed to InputLine::new)
    link: Rc<RefCell<String>>,
    pub selected_item: Option<String>, // Public so InputLine can read it
}

impl History {
    /// Create a new history button linked to an InputLine's shared data
    ///
    /// The button is 2 characters wide (shows '▼' or similar).
    /// `link` must be the same `Rc<RefCell<String>>` passed to the InputLine.
    pub fn new(pos: Point, history_id: u16, link: Rc<RefCell<String>>) -> Self {
        Self {
            core: ViewCore {
                bounds: Rect::new(pos.x, pos.y, pos.x + 2, pos.y + 1),
                state: 0,
                palette_chain: None,
                ..ViewCore::default()
            },
            history_id,
            link,
            selected_item: None,
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
            EventType::Broadcast => match event.command {
                CM_RECORD_HISTORY => {
                    // Dialog is being accepted: record the linked input text.
                    // Matches Borland: THistory handling cmRecordHistory.
                    let text = self.link.borrow().clone();
                    if !text.is_empty() {
                        HistoryManager::add(self.history_id, text);
                    }
                }
                CM_HISTORY_SELECTED if event.info == self.history_id => {
                    // A history item was picked in the popup; it was moved to the
                    // front of the list. Copy it into the linked input data.
                    if let Some(item) = HistoryManager::get_list(self.history_id).first() {
                        self.selected_item = Some(item.clone());
                        *self.link.borrow_mut() = item.clone();
                    }
                }
                _ => {}
            },
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
    link: Option<Rc<RefCell<String>>>,
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
    pub fn link(mut self, link: Rc<RefCell<String>>) -> Self {
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

    fn link(text: &str) -> Rc<RefCell<String>> {
        Rc::new(RefCell::new(text.to_string()))
    }

    #[test]
    fn test_history_button_creation() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();

        let button = History::new(Point::new(20, 5), 1, link(""));
        assert!(!button.has_items());
        assert_eq!(button.bounds().width(), 2);
    }

    #[test]
    fn test_history_button_with_items() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();
        HistoryManager::add(2, "test".to_string());

        let button = History::new(Point::new(20, 5), 2, link(""));
        assert!(button.has_items());
    }

    #[test]
    fn test_record_history_broadcast_records_linked_data() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();

        let data = link("hello world");
        let mut button = History::new(Point::new(20, 5), 3, Rc::clone(&data));

        let mut event = Event::broadcast(CM_RECORD_HISTORY);
        button.handle_event(&mut event);

        assert_eq!(HistoryManager::get_list(3), vec!["hello world".to_string()]);
        // Broadcasts are not consumed so all History views can record
        assert_eq!(event.what, EventType::Broadcast);

        // Empty input records nothing
        data.borrow_mut().clear();
        let mut event = Event::broadcast(CM_RECORD_HISTORY);
        button.handle_event(&mut event);
        assert_eq!(HistoryManager::count(3), 1);
    }

    #[test]
    fn test_click_converts_to_show_history_command() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();
        HistoryManager::add(4, "entry".to_string());

        let mut button = History::new(Point::new(20, 5), 4, link(""));
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

        let mut button = History::new(Point::new(20, 5), 5, link(""));
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
    fn test_history_selected_broadcast_updates_link() {
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();
        HistoryManager::add(6, "older".to_string());
        HistoryManager::add(6, "picked".to_string());

        let data = link("current");
        let mut button = History::new(Point::new(20, 5), 6, Rc::clone(&data));

        let mut event = Event::broadcast_with_info(CM_HISTORY_SELECTED, 6);
        button.handle_event(&mut event);
        assert_eq!(*data.borrow(), "picked");

        // Broadcast for a different history id is ignored
        *data.borrow_mut() = "unchanged".to_string();
        let mut event = Event::broadcast_with_info(CM_HISTORY_SELECTED, 7);
        button.handle_event(&mut event);
        assert_eq!(*data.borrow(), "unchanged");
    }
}
