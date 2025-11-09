// (C) 2025 - Enzo Lombardi

//! SortedListBox view - automatically sorted list with binary search support.
// SortedListBox - A sorted list with binary search capability
//
// Matches Borland: TSortedListBox (extends TListBox)
//
// A sorted listbox maintains items in sorted order and provides efficient
// binary search for finding items by prefix or exact match.
//
// Architecture: Extends ListBox functionality with sorting
//
// Usage:
//   let sorted = SortedListBox::new(Rect::new(5, 5, 35, 15), 1001);
//   sorted.add_item("Zebra");
//   sorted.add_item("Apple");
//   sorted.add_item("Banana");
//   // Items are automatically sorted: Apple, Banana, Zebra
//
//   // Binary search for item starting with "B"
//   if let Some(idx) = sorted.find_prefix("B") {
//       sorted.set_selection(idx);
//   }

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::state::StateFlags;
use crate::core::command::CommandId;
use crate::terminal::Terminal;
use super::view::View;
use super::list_viewer::{ListViewer, ListViewerState};

/// SortedListBox - A list that maintains items in sorted order
///
/// Extends ListBox with automatic sorting and binary search.
/// Matches Borland: TSortedListBox (extends TListBox)
pub struct SortedListBox {
    bounds: Rect,
    items: Vec<String>,
    list_state: ListViewerState,
    state: StateFlags,
    _on_select_command: CommandId,
    case_sensitive: bool,
    owner: Option<*const dyn View>,
}

impl SortedListBox {
    /// Create a new sorted list box
    pub fn new(bounds: Rect, on_select_command: CommandId) -> Self {
        Self {
            bounds,
            items: Vec::new(),
            list_state: ListViewerState::new(),
            state: 0,
            _on_select_command: on_select_command,
            case_sensitive: false,
            owner: None,
        }
    }

    /// Set whether sorting is case-sensitive (default: false)
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        if self.case_sensitive != case_sensitive {
            self.case_sensitive = case_sensitive;
            self.sort_items();
        }
    }

    /// Set the items in the list (will be automatically sorted)
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.sort_items();
        self.list_state.set_range(self.items.len());
    }

    /// Add an item to the list (maintains sorted order)
    pub fn add_item(&mut self, item: String) {
        // Use binary search to find insertion point
        let insertion_point = self.find_insertion_point(&item);
        self.items.insert(insertion_point, item);
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
        self.list_state.focused.and_then(|idx| self.items.get(idx).map(|s| s.as_str()))
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

    /// Find item by exact match (case-insensitive by default)
    ///
    /// Returns the index of the item if found using binary search.
    pub fn find_exact(&self, text: &str) -> Option<usize> {
        if self.case_sensitive {
            self.items.binary_search_by(|item| item.as_str().cmp(text)).ok()
        } else {
            self.items.binary_search_by(|item| {
                item.to_lowercase().as_str().cmp(&text.to_lowercase())
            }).ok()
        }
    }

    /// Find first item starting with the given prefix
    ///
    /// Returns the index of the first item that starts with the prefix.
    /// Uses binary search for efficiency.
    pub fn find_prefix(&self, prefix: &str) -> Option<usize> {
        if prefix.is_empty() {
            return if self.items.is_empty() { None } else { Some(0) };
        }

        if self.case_sensitive {
            self.find_prefix_case_sensitive(prefix)
        } else {
            self.find_prefix_case_insensitive(prefix)
        }
    }

    /// Helper for case-sensitive prefix search
    fn find_prefix_case_sensitive(&self, prefix: &str) -> Option<usize> {
        let compare_fn = |item: &String| -> std::cmp::Ordering {
            let item_prefix = &item[..prefix.len().min(item.len())];
            item_prefix.cmp(prefix)
        };

        match self.items.binary_search_by(compare_fn) {
            Ok(idx) => {
                // Found exact prefix match, walk backwards to find the first match
                let mut first_idx = idx;
                while first_idx > 0 && compare_fn(&self.items[first_idx - 1]) == std::cmp::Ordering::Equal {
                    first_idx -= 1;
                }
                Some(first_idx)
            }
            Err(insertion_point) => {
                // Check if the item at insertion_point starts with prefix
                if insertion_point < self.items.len() && self.items[insertion_point].starts_with(prefix) {
                    Some(insertion_point)
                } else {
                    None
                }
            }
        }
    }

    /// Helper for case-insensitive prefix search
    fn find_prefix_case_insensitive(&self, prefix: &str) -> Option<usize> {
        let prefix_lower = prefix.to_lowercase();

        let compare_fn = |item: &String| -> std::cmp::Ordering {
            let item_prefix = &item[..prefix_lower.len().min(item.len())];
            item_prefix.to_lowercase().as_str().cmp(&prefix_lower)
        };

        match self.items.binary_search_by(compare_fn) {
            Ok(idx) => {
                // Found exact prefix match, walk backwards to find the first match
                let mut first_idx = idx;
                while first_idx > 0 && compare_fn(&self.items[first_idx - 1]) == std::cmp::Ordering::Equal {
                    first_idx -= 1;
                }
                Some(first_idx)
            }
            Err(insertion_point) => {
                // Check if the item at insertion_point starts with prefix
                if insertion_point < self.items.len() {
                    let item = &self.items[insertion_point];
                    if item.to_lowercase().starts_with(&prefix_lower) {
                        Some(insertion_point)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Focus on the first item starting with the given prefix
    ///
    /// Returns true if an item was found and focused.
    pub fn focus_prefix(&mut self, prefix: &str) -> bool {
        if let Some(idx) = self.find_prefix(prefix) {
            self.set_selection(idx);
            true
        } else {
            false
        }
    }

    // Private helper methods

    /// Sort all items according to current settings
    fn sort_items(&mut self) {
        if self.case_sensitive {
            self.items.sort();
        } else {
            self.items.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        }
    }

    /// Find the insertion point for a new item using binary search
    fn find_insertion_point(&self, item: &str) -> usize {
        if self.case_sensitive {
            self.items.binary_search_by(|probe| probe.as_str().cmp(item))
                .unwrap_or_else(|idx| idx)
        } else {
            self.items.binary_search_by(|probe| {
                probe.to_lowercase().as_str().cmp(&item.to_lowercase())
            }).unwrap_or_else(|idx| idx)
        }
    }
}

impl View for SortedListBox {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        use crate::core::palette::{Attr, TvColor};
        use crate::core::draw::DrawBuffer;
        use super::view::write_line_to_terminal;

        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        let color_normal = if self.is_focused() {
            Attr::new(TvColor::Black, TvColor::White)
        } else {
            Attr::new(TvColor::Black, TvColor::LightGray)
        };
        let color_selected = if self.is_focused() {
            Attr::new(TvColor::White, TvColor::Cyan)
        } else {
            Attr::new(TvColor::White, TvColor::Blue)
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

        // TODO: Add incremental search on keyboard input
        // When user types letters, find and focus on matching items
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
        None  // SortedListBox uses hardcoded listbox colors
    }
}

// Implement ListViewer trait for standard navigation
impl ListViewer for SortedListBox {
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
    fn test_sorted_listbox_creation() {
        let listbox = SortedListBox::new(Rect::new(0, 0, 20, 10), 1000);
        assert_eq!(listbox.item_count(), 0);
        assert_eq!(listbox.get_selection(), None);
    }

    #[test]
    fn test_sorted_listbox_add_items_maintains_order() {
        let mut listbox = SortedListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.add_item("Zebra".to_string());
        listbox.add_item("Apple".to_string());
        listbox.add_item("Banana".to_string());
        listbox.add_item("Mango".to_string());

        assert_eq!(listbox.item_count(), 4);
        assert_eq!(listbox.items[0], "Apple");
        assert_eq!(listbox.items[1], "Banana");
        assert_eq!(listbox.items[2], "Mango");
        assert_eq!(listbox.items[3], "Zebra");
    }

    #[test]
    fn test_sorted_listbox_set_items_sorts() {
        let mut listbox = SortedListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "Dog".to_string(),
            "Cat".to_string(),
            "Ant".to_string(),
            "Bear".to_string(),
        ]);

        assert_eq!(listbox.item_count(), 4);
        assert_eq!(listbox.items[0], "Ant");
        assert_eq!(listbox.items[1], "Bear");
        assert_eq!(listbox.items[2], "Cat");
        assert_eq!(listbox.items[3], "Dog");
    }

    #[test]
    fn test_sorted_listbox_find_exact() {
        let mut listbox = SortedListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
            "Date".to_string(),
        ]);

        assert_eq!(listbox.find_exact("Banana"), Some(1));
        assert_eq!(listbox.find_exact("banana"), Some(1)); // Case-insensitive by default
        assert_eq!(listbox.find_exact("Cherry"), Some(2));
        assert_eq!(listbox.find_exact("Grape"), None);
    }

    #[test]
    fn test_sorted_listbox_find_prefix() {
        let mut listbox = SortedListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "Apple".to_string(),
            "Apricot".to_string(),
            "Banana".to_string(),
            "Berry".to_string(),
            "Cherry".to_string(),
        ]);

        // Should find first item starting with "Ap"
        assert_eq!(listbox.find_prefix("Ap"), Some(0)); // Apple
        assert_eq!(listbox.find_prefix("B"), Some(2)); // Banana
        assert_eq!(listbox.find_prefix("Be"), Some(3)); // Berry
        assert_eq!(listbox.find_prefix("C"), Some(4)); // Cherry
        assert_eq!(listbox.find_prefix("D"), None); // No match
    }

    #[test]
    fn test_sorted_listbox_focus_prefix() {
        let mut listbox = SortedListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "Apple".to_string(),
            "Apricot".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
        ]);

        assert!(listbox.focus_prefix("B"));
        assert_eq!(listbox.get_selection(), Some(2));
        assert_eq!(listbox.get_selected_item(), Some("Banana"));

        assert!(listbox.focus_prefix("Ap"));
        assert_eq!(listbox.get_selection(), Some(0));
        assert_eq!(listbox.get_selected_item(), Some("Apple"));

        assert!(!listbox.focus_prefix("Z"));
        // Selection should remain unchanged
        assert_eq!(listbox.get_selection(), Some(0));
    }

    #[test]
    fn test_sorted_listbox_case_sensitive() {
        let mut listbox = SortedListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_case_sensitive(true);
        listbox.set_items(vec![
            "apple".to_string(),
            "Apple".to_string(),
            "APPLE".to_string(),
            "banana".to_string(),
        ]);

        // With case-sensitive sorting: APPLE, Apple, apple, banana
        assert_eq!(listbox.items[0], "APPLE");
        assert_eq!(listbox.items[1], "Apple");
        assert_eq!(listbox.items[2], "apple");
        assert_eq!(listbox.items[3], "banana");

        // Case-sensitive search
        assert_eq!(listbox.find_exact("Apple"), Some(1));
        assert_eq!(listbox.find_exact("apple"), Some(2));
        assert_eq!(listbox.find_exact("APPLE"), Some(0));
    }

    #[test]
    fn test_sorted_listbox_case_insensitive_default() {
        let mut listbox = SortedListBox::new(Rect::new(0, 0, 20, 10), 1000);
        listbox.set_items(vec![
            "ZEBRA".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
        ]);

        // Case-insensitive: apple, Banana, ZEBRA (sorted alphabetically ignoring case)
        assert_eq!(listbox.items[0], "apple");
        assert_eq!(listbox.items[1], "Banana");
        assert_eq!(listbox.items[2], "ZEBRA");
    }
}
