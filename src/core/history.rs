// (C) 2025 - Enzo Lombardi
// History Management System
//
// Matches Borland: THistory system (histlist.h, histlist.cc)
//
// Provides centralized history management for input fields.
// Each history list is identified by a unique ID and stores
// a list of previously entered strings.
//
// Architecture:
// - HistoryList: Stores history items for a specific ID
// - HistoryManager: Global singleton managing all history lists
//
// Usage:
//   // Add to history
//   HistoryManager::add(history_id, "search query");
//
//   // Get history list
//   let items = HistoryManager::get_list(history_id);

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Maximum number of items to store in each history list
const MAX_HISTORY_ITEMS: usize = 20;

/// A list of history items for a specific history ID
#[derive(Clone, Debug)]
pub struct HistoryList {
    items: Vec<String>,
    max_items: usize,
}

impl HistoryList {
    /// Create a new empty history list
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_items: MAX_HISTORY_ITEMS,
        }
    }

    /// Create a history list with custom max items
    pub fn with_max_items(max_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_items,
        }
    }

    /// Add an item to the history
    ///
    /// If the item already exists, it's moved to the front.
    /// If the list is full, the oldest item is removed.
    pub fn add(&mut self, item: String) {
        // Don't add empty strings
        if item.is_empty() {
            return;
        }

        // Remove if already exists (we'll add it to the front)
        self.items.retain(|existing| existing != &item);

        // Add to front
        self.items.insert(0, item);

        // Trim to max size
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }
    }

    /// Get all items in the history (most recent first)
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the history is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all history items
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Get an item by index (0 = most recent)
    pub fn get(&self, index: usize) -> Option<&String> {
        self.items.get(index)
    }
}

impl Default for HistoryList {
    fn default() -> Self {
        Self::new()
    }
}

/// Global history manager
///
/// Manages all history lists by ID. Uses OnceLock for lazy initialization.
fn history_manager() -> &'static Mutex<HashMap<u16, HistoryList>> {
    static HISTORY_MANAGER: OnceLock<Mutex<HashMap<u16, HistoryList>>> = OnceLock::new();
    HISTORY_MANAGER.get_or_init(|| Mutex::new(HashMap::new()))
}

/// History manager for managing multiple history lists
pub struct HistoryManager;

impl HistoryManager {
    /// Add an item to a history list
    pub fn add(history_id: u16, item: String) {
        let mut manager = history_manager().lock().unwrap();
        let list = manager.entry(history_id).or_insert_with(HistoryList::new);
        list.add(item);
    }

    /// Get a copy of a history list
    pub fn get_list(history_id: u16) -> Vec<String> {
        let manager = history_manager().lock().unwrap();
        manager
            .get(&history_id)
            .map(|list| list.items().to_vec())
            .unwrap_or_default()
    }

    /// Check if a history list exists and has items
    pub fn has_history(history_id: u16) -> bool {
        let manager = history_manager().lock().unwrap();
        manager
            .get(&history_id)
            .map(|list| !list.is_empty())
            .unwrap_or(false)
    }

    /// Get the number of items in a history list
    pub fn count(history_id: u16) -> usize {
        let manager = history_manager().lock().unwrap();
        manager
            .get(&history_id)
            .map(|list| list.len())
            .unwrap_or(0)
    }

    /// Clear a specific history list
    pub fn clear(history_id: u16) {
        let mut manager = history_manager().lock().unwrap();
        if let Some(list) = manager.get_mut(&history_id) {
            list.clear();
        }
    }

    /// Clear all history lists
    pub fn clear_all() {
        let mut manager = history_manager().lock().unwrap();
        manager.clear();
    }

    /// Set custom max items for a history list
    pub fn set_max_items(history_id: u16, max_items: usize) {
        let mut manager = history_manager().lock().unwrap();
        let list = manager
            .entry(history_id)
            .or_insert_with(|| HistoryList::with_max_items(max_items));
        list.max_items = max_items;

        // Trim if needed
        if list.items.len() > max_items {
            list.items.truncate(max_items);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_list_add() {
        let mut list = HistoryList::new();
        assert!(list.is_empty());

        list.add("first".to_string());
        assert_eq!(list.len(), 1);
        assert_eq!(list.get(0), Some(&"first".to_string()));

        list.add("second".to_string());
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0), Some(&"second".to_string()));
        assert_eq!(list.get(1), Some(&"first".to_string()));
    }

    #[test]
    fn test_history_list_duplicate() {
        let mut list = HistoryList::new();
        list.add("item".to_string());
        list.add("other".to_string());
        list.add("item".to_string()); // Duplicate

        // Should have 2 items, with "item" at the front
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0), Some(&"item".to_string()));
        assert_eq!(list.get(1), Some(&"other".to_string()));
    }

    #[test]
    fn test_history_list_max_items() {
        let mut list = HistoryList::with_max_items(3);

        list.add("1".to_string());
        list.add("2".to_string());
        list.add("3".to_string());
        list.add("4".to_string());

        // Should only keep 3 most recent
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0), Some(&"4".to_string()));
        assert_eq!(list.get(1), Some(&"3".to_string()));
        assert_eq!(list.get(2), Some(&"2".to_string()));
    }

    #[test]
    fn test_history_list_empty_string() {
        let mut list = HistoryList::new();
        list.add("".to_string());
        assert!(list.is_empty());
    }

    #[test]
    fn test_history_manager() {
        // Clear all first to avoid test interference
        HistoryManager::clear_all();

        let history_id = 100;

        assert!(!HistoryManager::has_history(history_id));
        assert_eq!(HistoryManager::count(history_id), 0);

        HistoryManager::add(history_id, "test1".to_string());
        assert!(HistoryManager::has_history(history_id));
        assert_eq!(HistoryManager::count(history_id), 1);

        HistoryManager::add(history_id, "test2".to_string());
        assert_eq!(HistoryManager::count(history_id), 2);

        let items = HistoryManager::get_list(history_id);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "test2");
        assert_eq!(items[1], "test1");

        HistoryManager::clear(history_id);
        assert_eq!(HistoryManager::count(history_id), 0);
    }

    #[test]
    fn test_history_manager_multiple_lists() {
        HistoryManager::clear_all();

        HistoryManager::add(1, "list1_item1".to_string());
        HistoryManager::add(2, "list2_item1".to_string());
        HistoryManager::add(1, "list1_item2".to_string());

        assert_eq!(HistoryManager::count(1), 2);
        assert_eq!(HistoryManager::count(2), 1);

        let list1 = HistoryManager::get_list(1);
        let list2 = HistoryManager::get_list(2);

        assert_eq!(list1[0], "list1_item2");
        assert_eq!(list1[1], "list1_item1");
        assert_eq!(list2[0], "list2_item1");
    }
}
