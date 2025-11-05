// (C) 2025 - Enzo Lombardi
// Menu Viewer - Base trait and state for menu views
//
// Matches Borland: TMenuView (menuview.h, tmenuvie.cc)
//
// This module provides the foundational infrastructure for menu-based views:
// - MenuViewerState: Shared state (current item, menu data, parent relationship)
// - MenuViewer trait: Common behavior with default implementations
//
// Architecture: Hybrid trait + helper struct approach (same as ListViewer)
//
// Borland inheritance:
//   TView → TMenuView → TMenuBar, TMenuBox
//
// Rust composition:
//   View trait + MenuViewer trait → MenuBar, MenuBox (embed MenuViewerState)

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KeyCode, KB_UP, KB_DOWN, KB_ENTER, KB_ESC, MB_LEFT_BUTTON};
use crate::core::menu_data::{Menu, MenuItem};
use super::view::View;

/// State management for menu viewer components
///
/// Matches Borland: TMenuView fields
///
/// This struct holds the common state for all menu-based views.
/// Components embed this and expose it via the MenuViewer trait.
#[derive(Clone, Debug)]
pub struct MenuViewerState {
    /// The menu being displayed
    /// Matches Borland: TMenuView::menu
    pub menu: Option<Menu>,

    /// Index of currently selected/highlighted item
    /// Matches Borland: TMenuView::current (as index)
    pub current: Option<usize>,

    /// Whether menu is in compact mode (less spacing)
    /// Matches Borland: TMenuView::compactMenu
    pub compact_menu: bool,
}

impl MenuViewerState {
    /// Create a new menu viewer state
    pub fn new() -> Self {
        Self {
            menu: None,
            current: None,
            compact_menu: false,
        }
    }

    /// Create with a menu
    pub fn with_menu(menu: Menu) -> Self {
        let has_items = !menu.items.is_empty();
        Self {
            menu: Some(menu),
            current: if has_items { Some(0) } else { None },
            compact_menu: false,
        }
    }

    /// Set the menu
    pub fn set_menu(&mut self, menu: Menu) {
        let has_items = !menu.items.is_empty();
        self.menu = Some(menu);
        self.current = if has_items {
            // Select first selectable item
            self.find_first_selectable()
        } else {
            None
        };
    }

    /// Get the menu
    pub fn get_menu(&self) -> Option<&Menu> {
        self.menu.as_ref()
    }

    /// Get the menu mutably
    pub fn get_menu_mut(&mut self) -> Option<&mut Menu> {
        self.menu.as_mut()
    }

    /// Get the currently selected item
    pub fn get_current_item(&self) -> Option<&MenuItem> {
        if let (Some(menu), Some(idx)) = (&self.menu, self.current) {
            menu.items.get(idx)
        } else {
            None
        }
    }

    /// Find first selectable item
    fn find_first_selectable(&self) -> Option<usize> {
        if let Some(menu) = &self.menu {
            for (i, item) in menu.items.iter().enumerate() {
                if item.is_selectable() {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Move to next selectable item
    ///
    /// Matches Borland: TMenuView::nextItem()
    pub fn select_next(&mut self) {
        if let Some(menu) = &self.menu {
            if menu.items.is_empty() {
                return;
            }

            let start = self.current.unwrap_or(0);
            let mut idx = (start + 1) % menu.items.len();

            // Find next selectable item
            while idx != start {
                if menu.items[idx].is_selectable() {
                    self.current = Some(idx);
                    return;
                }
                idx = (idx + 1) % menu.items.len();
            }

            // If we wrapped around and found nothing, keep current
        }
    }

    /// Move to previous selectable item
    ///
    /// Matches Borland: TMenuView::prevItem()
    pub fn select_prev(&mut self) {
        if let Some(menu) = &self.menu {
            if menu.items.is_empty() {
                return;
            }

            let start = self.current.unwrap_or(0);
            let mut idx = if start == 0 {
                menu.items.len() - 1
            } else {
                start - 1
            };

            // Find previous selectable item
            while idx != start {
                if menu.items[idx].is_selectable() {
                    self.current = Some(idx);
                    return;
                }
                if idx == 0 {
                    idx = menu.items.len() - 1;
                } else {
                    idx -= 1;
                }
            }

            // If we wrapped around and found nothing, keep current
        }
    }

    /// Find item by accelerator character
    ///
    /// Matches Borland: TMenuView::findItem(char ch)
    pub fn find_item_by_char(&self, ch: char) -> Option<usize> {
        if let Some(menu) = &self.menu {
            let ch_lower = ch.to_ascii_lowercase();
            for (i, item) in menu.items.iter().enumerate() {
                if let Some(accel) = item.get_accelerator() {
                    if accel == ch_lower && item.is_selectable() {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    /// Find item by hot key (keyboard shortcut)
    ///
    /// Matches Borland: TMenuView::hotKey(ushort keyCode)
    pub fn find_item_by_hotkey(&self, key_code: KeyCode) -> Option<usize> {
        if let Some(menu) = &self.menu {
            for (i, item) in menu.items.iter().enumerate() {
                if item.is_selectable() {
                    // Check if this item matches the key code
                    // For now, we'll just check MenuItem::Regular key_code
                    match item {
                        MenuItem::Regular { key_code: item_key, enabled: true, .. } => {
                            if *item_key == key_code {
                                return Some(i);
                            }
                        }
                        MenuItem::SubMenu { key_code: item_key, .. } => {
                            if *item_key == key_code {
                                return Some(i);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Get the number of items in the menu
    pub fn item_count(&self) -> usize {
        self.menu.as_ref().map(|m| m.items.len()).unwrap_or(0)
    }
}

impl Default for MenuViewerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for menu viewer components
///
/// Matches Borland: TMenuView virtual methods
///
/// This trait provides the common interface for all menu-based views.
/// Components implement this trait and embed MenuViewerState for shared logic.
pub trait MenuViewer: View {
    /// Get the menu viewer state (read-only)
    fn menu_state(&self) -> &MenuViewerState;

    /// Get the menu viewer state (mutable)
    fn menu_state_mut(&mut self) -> &mut MenuViewerState;

    /// Get the rectangle for a specific menu item
    ///
    /// Matches Borland: TMenuView::getItemRect()
    /// Subclasses must implement based on their layout
    fn get_item_rect(&self, item_index: usize) -> Rect;

    /// Get the currently selected item index
    fn current_item(&self) -> Option<usize> {
        self.menu_state().current
    }

    /// Set the currently selected item
    fn set_current_item(&mut self, item: Option<usize>) {
        self.menu_state_mut().current = item;
    }

    /// Get the menu
    fn get_menu(&self) -> Option<&Menu> {
        self.menu_state().get_menu()
    }

    /// Get the menu mutably
    fn get_menu_mut(&mut self) -> Option<&mut Menu> {
        self.menu_state_mut().get_menu_mut()
    }

    /// Get the current menu item
    fn get_current_menu_item(&self) -> Option<&MenuItem> {
        self.menu_state().get_current_item()
    }

    /// Find item by accelerator character
    fn find_item_by_char(&self, ch: char) -> Option<usize> {
        self.menu_state().find_item_by_char(ch)
    }

    /// Find item by hot key
    fn find_item_by_hotkey(&self, key_code: KeyCode) -> Option<usize> {
        self.menu_state().find_item_by_hotkey(key_code)
    }

    /// Handle standard menu navigation events
    ///
    /// Matches Borland: TMenuView::handleEvent() navigation logic
    /// Returns true if event was handled
    fn handle_menu_event(&mut self, event: &mut Event) -> bool {
        match event.what {
            EventType::Keyboard => {
                match event.key_code {
                    KB_UP => {
                        self.menu_state_mut().select_prev();
                        event.clear();
                        true
                    }
                    KB_DOWN => {
                        self.menu_state_mut().select_next();
                        event.clear();
                        true
                    }
                    KB_ESC => {
                        // ESC closes menu - handled by subclass
                        false
                    }
                    KB_ENTER => {
                        // Enter activates current item - handled by subclass
                        false
                    }
                    key_code => {
                        // Check for accelerator key (printable characters)
                        if (32..127).contains(&key_code) {
                            let ch = (key_code as u8 as char).to_ascii_lowercase();
                            if let Some(idx) = self.find_item_by_char(ch) {
                                self.set_current_item(Some(idx));
                                event.clear();
                                return true;
                            }
                        }
                        false
                    }
                }
            }
            EventType::MouseDown => {
                if event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    let mouse_pos = event.mouse.pos;
                    let bounds = self.bounds();

                    // Check if click is within menu bounds
                    if bounds.contains(mouse_pos) {
                        // Find which item was clicked
                        if let Some(menu) = self.get_menu() {
                            for (i, _item) in menu.items.iter().enumerate() {
                                let item_rect = self.get_item_rect(i);
                                if item_rect.contains(mouse_pos) {
                                    self.set_current_item(Some(i));
                                    event.clear();
                                    return true;
                                }
                            }
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
    use crate::core::menu_data::MenuBuilder;

    #[test]
    fn test_menu_viewer_state_creation() {
        let state = MenuViewerState::new();
        assert!(state.menu.is_none());
        assert_eq!(state.current, None);
        assert!(!state.compact_menu);
    }

    #[test]
    fn test_menu_viewer_state_with_menu() {
        let menu = MenuBuilder::new()
            .item("~O~pen", 100, 0)
            .item("~S~ave", 101, 0)
            .separator()
            .item("E~x~it", 102, 0)
            .build();

        let state = MenuViewerState::with_menu(menu);
        assert!(state.menu.is_some());
        assert_eq!(state.current, Some(0)); // First item selected
        assert_eq!(state.item_count(), 4);
    }

    #[test]
    fn test_menu_navigation() {
        let menu = MenuBuilder::new()
            .item("Item 1", 100, 0)
            .item("Item 2", 101, 0)
            .separator()
            .item("Item 3", 102, 0)
            .build();

        let mut state = MenuViewerState::with_menu(menu);
        assert_eq!(state.current, Some(0));

        // Move next
        state.select_next();
        assert_eq!(state.current, Some(1));

        // Move next (skip separator)
        state.select_next();
        assert_eq!(state.current, Some(3)); // Skipped separator at index 2

        // Move prev
        state.select_prev();
        assert_eq!(state.current, Some(1)); // Skipped separator

        // Move prev
        state.select_prev();
        assert_eq!(state.current, Some(0));
    }

    #[test]
    fn test_find_item_by_char() {
        let menu = MenuBuilder::new()
            .item("~O~pen", 100, 0)
            .item("~S~ave", 101, 0)
            .item("E~x~it", 102, 0)
            .build();

        let state = MenuViewerState::with_menu(menu);

        assert_eq!(state.find_item_by_char('o'), Some(0));
        assert_eq!(state.find_item_by_char('O'), Some(0)); // Case insensitive
        assert_eq!(state.find_item_by_char('s'), Some(1));
        assert_eq!(state.find_item_by_char('x'), Some(2));
        assert_eq!(state.find_item_by_char('z'), None); // Not found
    }

    #[test]
    fn test_empty_menu() {
        let menu = Menu::new();
        let mut state = MenuViewerState::with_menu(menu);

        assert_eq!(state.current, None);
        assert_eq!(state.item_count(), 0);

        // Navigation on empty menu should not crash
        state.select_next();
        assert_eq!(state.current, None);

        state.select_prev();
        assert_eq!(state.current, None);
    }

    #[test]
    fn test_get_current_item() {
        let menu = MenuBuilder::new()
            .item("First", 100, 0)
            .item("Second", 101, 0)
            .build();

        let state = MenuViewerState::with_menu(menu);

        let item = state.get_current_item();
        assert!(item.is_some());
        assert_eq!(item.unwrap().text(), "First");
    }
}
