// (C) 2025 - Enzo Lombardi

//! ListViewer - base trait and state for scrollable list view implementations.
// List Viewer - Base trait and state for scrollable list views
//
// Matches Borland: TListViewer (lstviewr.h, tlistvie.cc)
//
// This module provides the foundational infrastructure for list-based views:
// - ListViewerState: Shared state (focused item, scroll position, range)
// - ListViewer trait: Common behavior with default implementations
//
// Architecture: Hybrid trait + helper struct approach
// - Trait provides polymorphism and default implementations
// - Helper struct shares state/logic across implementations
//
// Borland inheritance:
//   TView → TListViewer → TListBox
//
// Rust composition:
//   View trait + ListViewer trait → ListBox (embeds ListViewerState)

use crate::core::event::{Event, EventType, KB_UP, KB_DOWN, KB_PGUP, KB_PGDN, KB_HOME, KB_END, KB_ENTER, MB_LEFT_BUTTON};
use super::view::View;

/// State management for list viewer components
///
/// Matches Borland: TListViewer fields
///
/// This struct holds the common state for all list-based views.
/// Components embed this and expose it via the ListViewer trait.
#[derive(Clone, Debug)]
pub struct ListViewerState {
    /// First visible item (top of viewport)
    /// Matches Borland: TListViewer::topItem
    pub top_item: usize,

    /// Currently focused item (receives keyboard input)
    /// Matches Borland: TListViewer::focused
    pub focused: Option<usize>,

    /// Total number of items in the list
    /// Matches Borland: TListViewer::range
    pub range: usize,

    /// Number of columns for multi-column lists
    /// Matches Borland: TListViewer::numCols
    pub num_cols: u16,

    /// Whether space bar selects items
    /// Matches Borland: TListViewer::handleSpace
    pub handle_space: bool,
}

impl ListViewerState {
    /// Create a new list viewer state
    pub fn new() -> Self {
        Self {
            top_item: 0,
            focused: None,
            range: 0,
            num_cols: 1,
            handle_space: true,
        }
    }

    /// Create with specific item count
    pub fn with_range(range: usize) -> Self {
        Self {
            top_item: 0,
            focused: if range > 0 { Some(0) } else { None },
            range,
            num_cols: 1,
            handle_space: true,
        }
    }

    /// Set the total number of items
    ///
    /// Matches Borland: TListViewer::setRange()
    pub fn set_range(&mut self, range: usize) {
        self.range = range;

        // Adjust focused item if out of range
        if let Some(focused) = self.focused {
            if focused >= range {
                self.focused = if range > 0 { Some(range - 1) } else { None };
            }
        } else if range > 0 {
            self.focused = Some(0);
        }

        // Adjust top_item if out of range
        if self.top_item >= range && range > 0 {
            self.top_item = range - 1;
        }
    }

    /// Focus a specific item
    ///
    /// Matches Borland: TListViewer::focusItem()
    pub fn focus_item(&mut self, item: usize, visible_rows: usize) {
        if item >= self.range {
            return;
        }

        self.focused = Some(item);

        // Scroll if needed to make item visible
        if item < self.top_item {
            // Item is above viewport - scroll up
            self.top_item = item;
        } else if item >= self.top_item + visible_rows {
            // Item is below viewport - scroll down
            self.top_item = item - visible_rows + 1;
        }
    }

    /// Focus item and center it in viewport
    ///
    /// Matches Borland: TListViewer::focusItemCentered()
    pub fn focus_item_centered(&mut self, item: usize, visible_rows: usize) {
        if item >= self.range {
            return;
        }

        self.focused = Some(item);

        // Center the item in viewport
        if visible_rows > 0 {
            let half_rows = visible_rows / 2;
            if item >= half_rows {
                self.top_item = item - half_rows;
            } else {
                self.top_item = 0;
            }

            // Don't scroll past end
            let max_top = if self.range > visible_rows {
                self.range - visible_rows
            } else {
                0
            };
            if self.top_item > max_top {
                self.top_item = max_top;
            }
        }
    }

    /// Move focus to next item
    pub fn focus_next(&mut self, visible_rows: usize) {
        if let Some(focused) = self.focused {
            if focused + 1 < self.range {
                self.focus_item(focused + 1, visible_rows);
            }
        } else if self.range > 0 {
            self.focus_item(0, visible_rows);
        }
    }

    /// Move focus to previous item
    pub fn focus_prev(&mut self, visible_rows: usize) {
        if let Some(focused) = self.focused {
            if focused > 0 {
                self.focus_item(focused - 1, visible_rows);
            }
        } else if self.range > 0 {
            self.focus_item(0, visible_rows);
        }
    }

    /// Move focus down one page
    pub fn focus_page_down(&mut self, visible_rows: usize) {
        if let Some(focused) = self.focused {
            let new_focused = (focused + visible_rows).min(self.range.saturating_sub(1));
            self.focus_item(new_focused, visible_rows);
        } else if self.range > 0 {
            self.focus_item(0, visible_rows);
        }
    }

    /// Move focus up one page
    pub fn focus_page_up(&mut self, visible_rows: usize) {
        if let Some(focused) = self.focused {
            let new_focused = focused.saturating_sub(visible_rows);
            self.focus_item(new_focused, visible_rows);
        } else if self.range > 0 {
            self.focus_item(0, visible_rows);
        }
    }

    /// Move focus to first item
    pub fn focus_first(&mut self, visible_rows: usize) {
        if self.range > 0 {
            self.focus_item(0, visible_rows);
        }
    }

    /// Move focus to last item
    pub fn focus_last(&mut self, visible_rows: usize) {
        if self.range > 0 {
            self.focus_item(self.range - 1, visible_rows);
        }
    }
}

impl Default for ListViewerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for list viewer components
///
/// Matches Borland: TListViewer virtual methods
///
/// This trait provides the common interface for all list-based views.
/// Components implement this trait and embed ListViewerState for shared logic.
pub trait ListViewer: View {
    /// Get the list viewer state (read-only)
    fn list_state(&self) -> &ListViewerState;

    /// Get the list viewer state (mutable)
    fn list_state_mut(&mut self) -> &mut ListViewerState;

    /// Get text for a specific item
    ///
    /// Matches Borland: TListViewer::getText()
    /// This is abstract in Borland - subclasses must implement
    fn get_text(&self, item: usize, max_len: usize) -> String;

    /// Check if an item is selected
    ///
    /// Matches Borland: TListViewer::isSelected()
    /// Default: focused item is selected
    fn is_selected(&self, item: usize) -> bool {
        Some(item) == self.list_state().focused
    }

    /// Select an item (for multi-select lists)
    ///
    /// Matches Borland: TListViewer::selectItem()
    /// Default: just focuses the item
    fn select_item(&mut self, item: usize) {
        let visible_rows = self.visible_rows();
        self.list_state_mut().focus_item(item, visible_rows);
    }

    /// Get the currently focused item
    fn focused_item(&self) -> Option<usize> {
        self.list_state().focused
    }

    /// Set the focused item
    fn set_focused_item(&mut self, item: Option<usize>) {
        if let Some(idx) = item {
            let visible_rows = self.visible_rows();
            self.list_state_mut().focus_item(idx, visible_rows);
        } else {
            self.list_state_mut().focused = None;
        }
    }

    /// Get the total number of items
    fn item_count(&self) -> usize {
        self.list_state().range
    }

    /// Set the total number of items
    fn set_item_count(&mut self, count: usize) {
        self.list_state_mut().set_range(count);
    }

    /// Get the first visible item
    fn top_item(&self) -> usize {
        self.list_state().top_item
    }

    /// Get the number of visible rows in viewport
    fn visible_rows(&self) -> usize {
        self.bounds().height_clamped() as usize
    }

    /// Handle standard list navigation events
    ///
    /// Matches Borland: TListViewer::handleEvent() navigation logic
    /// Returns true if event was handled
    fn handle_list_event(&mut self, event: &mut Event) -> bool {
        let visible_rows = self.visible_rows();

        match event.what {
            EventType::Keyboard => {
                let state = self.list_state_mut();
                match event.key_code {
                    KB_UP => {
                        state.focus_prev(visible_rows);
                        event.clear();
                        true
                    }
                    KB_DOWN => {
                        state.focus_next(visible_rows);
                        event.clear();
                        true
                    }
                    KB_PGUP => {
                        state.focus_page_up(visible_rows);
                        event.clear();
                        true
                    }
                    KB_PGDN => {
                        state.focus_page_down(visible_rows);
                        event.clear();
                        true
                    }
                    KB_HOME => {
                        state.focus_first(visible_rows);
                        event.clear();
                        true
                    }
                    KB_END => {
                        state.focus_last(visible_rows);
                        event.clear();
                        true
                    }
                    KB_ENTER => {
                        // Enter on focused item - subclass should handle
                        false
                    }
                    _ => false,
                }
            }
            EventType::MouseDown => {
                if event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    let mouse_pos = event.mouse.pos;
                    let bounds = self.bounds();

                    // Check if click is within bounds
                    if bounds.contains(mouse_pos) {
                        // Calculate which item was clicked
                        let relative_y = (mouse_pos.y - bounds.a.y) as usize;
                        let clicked_item = self.list_state().top_item + relative_y;

                        if clicked_item < self.item_count() {
                            self.select_item(clicked_item);
                            event.clear();
                            return true;
                        }
                    }
                }
                false
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_viewer_state_creation() {
        let state = ListViewerState::new();
        assert_eq!(state.top_item, 0);
        assert_eq!(state.focused, None);
        assert_eq!(state.range, 0);
        assert_eq!(state.num_cols, 1);
    }

    #[test]
    fn test_list_viewer_state_with_range() {
        let state = ListViewerState::with_range(10);
        assert_eq!(state.range, 10);
        assert_eq!(state.focused, Some(0));
    }

    #[test]
    fn test_set_range() {
        let mut state = ListViewerState::new();
        state.set_range(5);
        assert_eq!(state.range, 5);
        assert_eq!(state.focused, Some(0));

        // Set range to 0
        state.set_range(0);
        assert_eq!(state.range, 0);
        assert_eq!(state.focused, None);
    }

    #[test]
    fn test_focus_navigation() {
        let mut state = ListViewerState::with_range(10);
        let visible_rows = 5;

        // Start at 0
        assert_eq!(state.focused, Some(0));

        // Move down
        state.focus_next(visible_rows);
        assert_eq!(state.focused, Some(1));

        // Move up
        state.focus_prev(visible_rows);
        assert_eq!(state.focused, Some(0));

        // Can't move up from 0
        state.focus_prev(visible_rows);
        assert_eq!(state.focused, Some(0));

        // Move to end
        state.focus_last(visible_rows);
        assert_eq!(state.focused, Some(9));

        // Can't move down from end
        state.focus_next(visible_rows);
        assert_eq!(state.focused, Some(9));
    }

    #[test]
    fn test_page_navigation() {
        let mut state = ListViewerState::with_range(20);
        let visible_rows = 5;

        // Start at 0
        assert_eq!(state.focused, Some(0));

        // Page down
        state.focus_page_down(visible_rows);
        assert_eq!(state.focused, Some(5));

        // Page down again
        state.focus_page_down(visible_rows);
        assert_eq!(state.focused, Some(10));

        // Page up
        state.focus_page_up(visible_rows);
        assert_eq!(state.focused, Some(5));

        // Page up again
        state.focus_page_up(visible_rows);
        assert_eq!(state.focused, Some(0));
    }

    #[test]
    fn test_focus_item_scrolling() {
        let mut state = ListViewerState::with_range(20);
        let visible_rows = 5;

        // Focus item 0 - no scroll needed
        state.focus_item(0, visible_rows);
        assert_eq!(state.focused, Some(0));
        assert_eq!(state.top_item, 0);

        // Focus item 10 - should scroll
        state.focus_item(10, visible_rows);
        assert_eq!(state.focused, Some(10));
        assert_eq!(state.top_item, 6); // 10 - 5 + 1

        // Focus item 2 - should scroll up
        state.focus_item(2, visible_rows);
        assert_eq!(state.focused, Some(2));
        assert_eq!(state.top_item, 2);
    }

    #[test]
    fn test_focus_item_centered() {
        let mut state = ListViewerState::with_range(20);
        let visible_rows = 5;

        // Center item 10
        state.focus_item_centered(10, visible_rows);
        assert_eq!(state.focused, Some(10));
        assert_eq!(state.top_item, 8); // 10 - 5/2 = 8

        // Center item 2 (near start)
        state.focus_item_centered(2, visible_rows);
        assert_eq!(state.focused, Some(2));
        assert_eq!(state.top_item, 0);
    }
}
