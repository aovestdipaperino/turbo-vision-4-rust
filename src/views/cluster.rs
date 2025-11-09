// (C) 2025 - Enzo Lombardi

//! Cluster - base trait and state management for grouped button controls.
// Cluster - Base trait and state for button group controls
//
// Matches Borland: TCluster (cluster.h, tcluster.cc)
//
// This module provides the foundational infrastructure for button group controls:
// - ClusterState: Shared state (selection, items, group management)
// - Cluster trait: Common behavior with default implementations
//
// Architecture: Hybrid trait + helper struct approach (same as ListViewer/MenuViewer)
//
// Borland inheritance:
//   TView → TCluster → TCheckBoxes, TRadioButtons
//
// Rust composition:
//   View trait + Cluster trait → CheckBox, RadioButton (embed ClusterState)

use crate::core::event::{Event, EventType};
use crate::core::palette::Attr;
use super::view::View;

/// State management for cluster (button group) components
///
/// Matches Borland: TCluster fields
///
/// This struct holds the common state for all button group controls.
/// Components embed this and expose it via the Cluster trait.
#[derive(Clone, Debug)]
pub struct ClusterState {
    /// Current selection value
    /// For CheckBox: 0 = unchecked, 1 = checked
    /// For RadioButton: index of selected button in group
    pub value: u32,

    /// Group ID for radio button groups
    /// Radio buttons with same group_id are mutually exclusive
    pub group_id: u16,

    /// Whether to enable keyboard selection with space
    pub enable_keyboard: bool,
}

impl ClusterState {
    /// Create a new cluster state
    pub fn new() -> Self {
        Self {
            value: 0,
            group_id: 0,
            enable_keyboard: true,
        }
    }

    /// Create with a specific group ID (for radio buttons)
    pub fn with_group(group_id: u16) -> Self {
        Self {
            value: 0,
            group_id,
            enable_keyboard: true,
        }
    }

    /// Check if a specific item is selected
    pub fn is_selected(&self, item_value: u32) -> bool {
        self.value == item_value
    }

    /// Set the selection value
    pub fn set_value(&mut self, value: u32) {
        self.value = value;
    }

    /// Toggle selection (for checkboxes)
    pub fn toggle(&mut self) {
        self.value = if self.value == 0 { 1 } else { 0 };
    }
}

impl Default for ClusterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for button group (cluster) components
///
/// Matches Borland: TCluster virtual methods
///
/// This trait provides the common interface for all button group controls.
/// Components implement this trait and embed ClusterState for shared logic.
pub trait Cluster: View {
    /// Get the cluster state (read-only)
    fn cluster_state(&self) -> &ClusterState;

    /// Get the cluster state (mutable)
    fn cluster_state_mut(&mut self) -> &mut ClusterState;

    /// Get the label text for display
    fn get_label(&self) -> &str;

    /// Get the marker string for this control
    ///
    /// Examples:
    /// - CheckBox unchecked: "[ ] "
    /// - CheckBox checked: "[X] "
    /// - RadioButton unselected: "( ) "
    /// - RadioButton selected: "(•) "
    fn get_marker(&self) -> &str;

    /// Get the current selection value
    fn get_value(&self) -> u32 {
        self.cluster_state().value
    }

    /// Set the selection value
    fn set_value(&mut self, value: u32) {
        self.cluster_state_mut().set_value(value);
    }

    /// Check if currently selected/checked
    fn is_selected(&self) -> bool {
        self.cluster_state().value != 0
    }

    /// Toggle selection (for checkboxes)
    fn toggle(&mut self) {
        self.cluster_state_mut().toggle();
    }

    /// Get the group ID
    fn group_id(&self) -> u16 {
        self.cluster_state().group_id
    }

    /// Get colors based on focus state
    ///
    /// Returns (normal_color, hotkey_color)
    fn get_colors(&self) -> (Attr, Attr) {
        // Cluster palette indices:
        // 1: Normal (unfocused), 2: Focused, 3: Shortcut
        if self.is_focused() {
            (self.map_color(2), self.map_color(3))
        } else {
            (self.map_color(1), self.map_color(3))
        }
    }

    /// Handle standard cluster events
    ///
    /// Matches Borland: TCluster::handleEvent() keyboard logic
    /// Returns true if event was handled
    fn handle_cluster_event(&mut self, event: &mut Event) -> bool {
        if event.what == EventType::Keyboard && self.is_focused() {
            if self.cluster_state().enable_keyboard {
                // Space key toggles/selects
                if event.key_code == ' ' as u16 {
                    self.on_space_pressed();
                    event.clear();
                    return true;
                }
            }
        }
        false
    }

    /// Called when space key is pressed
    ///
    /// Default: toggle for checkboxes, select for radio buttons
    /// Subclasses can override for custom behavior
    fn on_space_pressed(&mut self) {
        // Default behavior: toggle
        self.toggle();
    }

    /// Draw the cluster control with marker and label
    ///
    /// Provides common drawing logic for all cluster controls
    fn draw_cluster(&self, terminal: &mut crate::terminal::Terminal) {
        use crate::core::draw::DrawBuffer;
        use crate::views::view::write_line_to_terminal;

        let bounds = self.bounds();
        let width = bounds.width() as usize;
        let mut buffer = DrawBuffer::new(width);

        let (color, hotkey_color) = self.get_colors();

        // Draw marker (checkbox/radio button)
        let marker = self.get_marker();
        buffer.move_str(0, marker, color);

        // Draw label with hotkey support
        let label = self.get_label();
        buffer.move_str_with_shortcut(marker.len(), label, color, hotkey_color);

        write_line_to_terminal(terminal, bounds.a.x, bounds.a.y, &buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_state_creation() {
        let state = ClusterState::new();
        assert_eq!(state.value, 0);
        assert_eq!(state.group_id, 0);
        assert!(state.enable_keyboard);
    }

    #[test]
    fn test_cluster_state_with_group() {
        let state = ClusterState::with_group(5);
        assert_eq!(state.value, 0);
        assert_eq!(state.group_id, 5);
    }

    #[test]
    fn test_cluster_state_selection() {
        let mut state = ClusterState::new();

        assert!(!state.is_selected(1));
        state.set_value(1);
        assert!(state.is_selected(1));
        assert!(!state.is_selected(2));
    }

    #[test]
    fn test_cluster_state_toggle() {
        let mut state = ClusterState::new();

        assert_eq!(state.value, 0);
        state.toggle();
        assert_eq!(state.value, 1);
        state.toggle();
        assert_eq!(state.value, 0);
    }
}
