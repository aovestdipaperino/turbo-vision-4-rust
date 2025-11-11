// (C) 2025 - Enzo Lombardi

//! ListBox view - scrollable list with single selection support.

use super::list_viewer::{ListViewer, ListViewerState};
use super::view::{write_line_to_terminal, View};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_ENTER, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::{LISTBOX_FOCUSED, LISTBOX_NORMAL, LISTBOX_SELECTED};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;

/// ListBox - A scrollable list of selectable items
///
/// Now implements ListViewer trait for standard navigation behavior.
/// Matches Borland: TListBox (extends TListViewer)
pub struct ListBox {
    bounds: Rect,
    items: Vec<String>,
    list_state: ListViewerState, // Embedded state from ListViewer
    state: StateFlags,
    on_select_command: CommandId,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl ListBox {
    /// Create a new list box
    pub fn new(bounds: Rect, on_select_command: CommandId) -> Self {
        Self {
            bounds,
            items: Vec::new(),
            list_state: ListViewerState::new(),
            state: 0,
            on_select_command,
            owner: None,
            owner_type: super::view::OwnerType::None,
        }
    }

    /// Set the items in the list
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.list_state.set_range(self.items.len());
    }

    /// Add an item to the list
    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
        self.list_state.set_range(self.items.len());
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.list_state.set_range(0);
    }

    /// Get the currently selected item index
    pub fn get_selection(&self) -> Option<usize> {
        self.list_state.focused
    }

    /// Get the currently selected item text
    pub fn get_selected_item(&self) -> Option<&str> {
        self.list_state
            .focused
            .and_then(|idx| self.items.get(idx).map(|s| s.as_str()))
    }

    /// Set the selected item by index
    pub fn set_selection(&mut self, index: usize) {
        if index < self.items.len() {
            let visible_rows = self.bounds.height() as usize;
            self.list_state.focus_item(index, visible_rows);
        }
    }

    /// Get the number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    // Convenience methods for compatibility with existing code
    // These delegate to ListViewerState methods

    /// Move selection up (convenience method)
    pub fn select_prev(&mut self) {
        let visible_rows = self.bounds.height() as usize;
        self.list_state.focus_prev(visible_rows);
    }

    /// Move selection down (convenience method)
    pub fn select_next(&mut self) {
        let visible_rows = self.bounds.height() as usize;
        self.list_state.focus_next(visible_rows);
    }

    /// Select first item (convenience method)
    pub fn select_first(&mut self) {
        let visible_rows = self.bounds.height() as usize;
        self.list_state.focus_first(visible_rows);
    }

    /// Select last item (convenience method)
    pub fn select_last(&mut self) {
        let visible_rows = self.bounds.height() as usize;
        self.list_state.focus_last(visible_rows);
    }

    /// Page up (convenience method)
    pub fn page_up(&mut self) {
        let visible_rows = self.bounds.height() as usize;
        self.list_state.focus_page_up(visible_rows);
    }

    /// Page down (convenience method)
    pub fn page_down(&mut self) {
        let visible_rows = self.bounds.height() as usize;
        self.list_state.focus_page_down(visible_rows);
    }
}

impl View for ListBox {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        // ListBox palette indices:
        // 1: Normal, 2: Focused, 3: Selected, 4: Divider
        let color_normal = if self.is_focused() {
            self.map_color(LISTBOX_FOCUSED) // Focused
        } else {
            self.map_color(LISTBOX_NORMAL) // Normal
        };
        let color_selected = self.map_color(LISTBOX_SELECTED); // Selected

        // Draw visible items
        for i in 0..height {
            let mut buf = DrawBuffer::new(width);
            let item_idx = self.list_state.top_item + i;

            if item_idx < self.items.len() {
                let is_selected = Some(item_idx) == self.list_state.focused;
                let color = if is_selected {
                    color_selected
                } else {
                    color_normal
                };

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
        // First try standard list navigation (from ListViewer trait)
        if self.handle_list_event(event) {
            return;
        }

        // Handle ListBox-specific events
        match event.what {
            EventType::Keyboard => {
                if event.key_code == KB_ENTER {
                    // Enter on selected item generates command
                    *event = Event::command(self.on_select_command);
                }
            }
            EventType::MouseDown => {
                let mouse_pos = event.mouse.pos;

                // Check if click is within the listbox bounds
                if self.bounds.contains(mouse_pos) && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    // Double-click triggers selection command (matching Borland's TListViewer)
                    if event.mouse.double_click {
                        *event = Event::command(self.on_select_command);
                    }
                    // Single click is handled by handle_list_event above
                }
            }
            EventType::MouseWheelUp => {
                let mouse_pos = event.mouse.pos;
                if self.bounds.contains(mouse_pos) {
                    self.select_prev();
                    event.clear();
                }
            }
            EventType::MouseWheelDown => {
                let mouse_pos = event.mouse.pos;
                if self.bounds.contains(mouse_pos) {
                    self.select_next();
                    event.clear();
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

    fn set_list_selection(&mut self, index: usize) {
        self.set_selection(index);
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
        Some(Palette::from_slice(palettes::CP_LISTBOX))
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

// Implement ListViewer trait
impl ListViewer for ListBox {
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

/// Builder for creating listboxes with a fluent API.
pub struct ListBoxBuilder {
    bounds: Option<Rect>,
    on_select_command: CommandId,
}

impl ListBoxBuilder {
    pub fn new() -> Self {
        Self { bounds: None, on_select_command: 0 }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn on_select_command(mut self, command: CommandId) -> Self {
        self.on_select_command = command;
        self
    }

    pub fn build(self) -> ListBox {
        let bounds = self.bounds.expect("ListBox bounds must be set");
        ListBox::new(bounds, self.on_select_command)
    }

    pub fn build_boxed(self) -> Box<ListBox> {
        Box::new(self.build())
    }
}

impl Default for ListBoxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listbox_creation() {
        let listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        assert_eq!(listbox.item_count(), 0);
        assert_eq!(listbox.get_selection(), None);
    }

    #[test]
    fn test_listbox_add_items() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.add_item("Item 1".to_string());
        listbox.add_item("Item 2".to_string());
        listbox.add_item("Item 3".to_string());

        assert_eq!(listbox.item_count(), 3);
        assert_eq!(listbox.get_selection(), Some(0));
        assert_eq!(listbox.get_selected_item(), Some("Item 1"));
    }

    #[test]
    fn test_listbox_set_items() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        let items = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];
        listbox.set_items(items);

        assert_eq!(listbox.item_count(), 3);
        assert_eq!(listbox.get_selection(), Some(0));
    }

    #[test]
    fn test_listbox_navigation() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string(),
        ]);

        assert_eq!(listbox.get_selection(), Some(0));

        listbox.select_next();
        assert_eq!(listbox.get_selection(), Some(1));

        listbox.select_next();
        assert_eq!(listbox.get_selection(), Some(2));

        listbox.select_next(); // Should stay at 2 (last item)
        assert_eq!(listbox.get_selection(), Some(2));

        listbox.select_prev();
        assert_eq!(listbox.get_selection(), Some(1));

        listbox.select_first();
        assert_eq!(listbox.get_selection(), Some(0));

        listbox.select_last();
        assert_eq!(listbox.get_selection(), Some(2));
    }

    #[test]
    fn test_listbox_set_selection() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ]);

        listbox.set_selection(2);
        assert_eq!(listbox.get_selection(), Some(2));
        assert_eq!(listbox.get_selected_item(), Some("C"));

        listbox.set_selection(10); // Out of bounds, should be ignored
        assert_eq!(listbox.get_selection(), Some(2)); // Should not change
    }

    #[test]
    fn test_listbox_clear() {
        let mut listbox = ListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec!["Item 1".to_string(), "Item 2".to_string()]);
        assert_eq!(listbox.item_count(), 2);

        listbox.clear();
        assert_eq!(listbox.item_count(), 0);
        assert_eq!(listbox.get_selection(), None);
    }
}
