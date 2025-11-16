// (C) 2025 - Enzo Lombardi

//! HistoryViewer view - displays history items in a scrollable list.
// HistoryViewer - Displays history items in a list
//
// Matches Borland: THistoryViewer (extends TListViewer)
//
// Displays the history items for a specific history ID.
// Used inside HistoryWindow to show the history list.
//
// Usage:
//   let viewer = HistoryViewer::new(bounds, history_id);
//   // Viewer will display items from HistoryManager

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::state::StateFlags;
use crate::core::history::HistoryManager;
use crate::terminal::Terminal;
use super::view::View;
use super::list_viewer::{ListViewer, ListViewerState};

/// HistoryViewer - Displays history items for a specific history ID
///
/// Extends ListViewer trait for standard list navigation.
/// Matches Borland: THistoryViewer (extends TListViewer)
pub struct HistoryViewer {
    bounds: Rect,
    history_id: u16,
    items: Vec<String>,
    list_state: ListViewerState,
    state: StateFlags,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl HistoryViewer {
    /// Create a new history viewer for the given history ID
    pub fn new(bounds: Rect, history_id: u16) -> Self {
        let items = HistoryManager::get_list(history_id);
        let mut list_state = ListViewerState::new();
        list_state.set_range(items.len());

        Self {
            bounds,
            history_id,
            items,
            list_state,
            state: 0,
            owner: None,
            owner_type: super::view::OwnerType::None,
        }
    }

    /// Refresh the history items from HistoryManager
    pub fn refresh(&mut self) {
        self.items = HistoryManager::get_list(self.history_id);
        self.list_state.set_range(self.items.len());
    }

    /// Get the currently selected history item
    pub fn get_selected_item(&self) -> Option<&str> {
        self.list_state.focused.and_then(|idx| self.items.get(idx).map(|s| s.as_str()))
    }

    /// Get the number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

impl View for HistoryViewer {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        
        use crate::core::draw::DrawBuffer;
        use super::view::write_line_to_terminal;

        let width = self.bounds.width_clamped() as usize;
        let height = self.bounds.height_clamped() as usize;

        use crate::core::palette::colors::{LISTBOX_FOCUSED, LISTBOX_NORMAL, LISTBOX_SELECTED_FOCUSED, LISTBOX_SELECTED};
        let color_normal = if self.is_focused() {
            LISTBOX_FOCUSED
        } else {
            LISTBOX_NORMAL
        };
        let color_selected = if self.is_focused() {
            LISTBOX_SELECTED_FOCUSED
        } else {
            LISTBOX_SELECTED
        };

        // Draw visible items
        for i in 0..height {
            let mut buf = DrawBuffer::new(width);
            let item_idx = self.list_state.top_item + i;

            if item_idx < self.items.len() {
                let is_selected = Some(item_idx) == self.list_state.focused;
                let color = if is_selected { color_selected } else { color_normal };

                let text = &self.items[item_idx];
                buf.move_str(0, text, color);

                // Fill rest of line with spaces
                let text_len = text.len();
                if text_len < width {
                    buf.move_char(text_len, ' ', color, width - text_len);
                }
            } else {
                // Empty line
                buf.move_char(0, ' ', color_normal, width);
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Use ListViewer trait's standard event handling
        self.handle_list_event(event);
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

    fn set_list_selection(&mut self, index: usize) {
        if index < self.items.len() {
            let visible_rows = self.bounds.height_clamped() as usize;
            self.list_state.focus_item(index, visible_rows);
        }
    }

    fn get_list_selection(&self) -> usize {
        self.list_state.focused.unwrap_or(0)
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_HISTORY_VIEWER))
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

// Implement ListViewer trait for standard navigation
impl ListViewer for HistoryViewer {
    fn list_state(&self) -> &ListViewerState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListViewerState {
        &mut self.list_state
    }

    fn get_text(&self, item: usize, _max_len: usize) -> String {
        self.items.get(item).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_viewer_creation() {
        // Clear and add test data
        HistoryManager::clear_all();
        HistoryManager::add(1, "test1".to_string());
        HistoryManager::add(1, "test2".to_string());

        let viewer = HistoryViewer::new(Rect::new(0, 0, 20, 10), 1);
        assert_eq!(viewer.item_count(), 2);
        assert_eq!(viewer.get_selected_item(), Some("test2")); // Most recent is first
    }

    #[test]
    fn test_history_viewer_refresh() {
        HistoryManager::clear_all();
        HistoryManager::add(2, "item1".to_string());

        let mut viewer = HistoryViewer::new(Rect::new(0, 0, 20, 10), 2);
        assert_eq!(viewer.item_count(), 1);

        // Add more items
        HistoryManager::add(2, "item2".to_string());
        HistoryManager::add(2, "item3".to_string());

        viewer.refresh();
        assert_eq!(viewer.item_count(), 3);
    }

    #[test]
    fn test_history_viewer_empty() {
        HistoryManager::clear_all();

        let viewer = HistoryViewer::new(Rect::new(0, 0, 20, 10), 99);
        assert_eq!(viewer.item_count(), 0);
        assert_eq!(viewer.get_selected_item(), None);
    }
}

/// Builder for creating history viewers with a fluent API.
pub struct HistoryViewerBuilder {
    bounds: Option<Rect>,
    history_id: Option<u16>,
}

impl HistoryViewerBuilder {
    pub fn new() -> Self {
        Self { bounds: None, history_id: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn history_id(mut self, history_id: u16) -> Self {
        self.history_id = Some(history_id);
        self
    }

    pub fn build(self) -> HistoryViewer {
        let bounds = self.bounds.expect("HistoryViewer bounds must be set");
        let history_id = self.history_id.expect("HistoryViewer history_id must be set");
        HistoryViewer::new(bounds, history_id)
    }

    pub fn build_boxed(self) -> Box<HistoryViewer> {
        Box::new(self.build())
    }
}

impl Default for HistoryViewerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
