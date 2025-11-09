// (C) 2025 - Enzo Lombardi

//! RadioButton view - mutually exclusive selection control (one choice from many).
// RadioButton - Mutually exclusive selection control
//
// Matches Borland: TRadioButtons (extends TCluster)
//
// A radio button control displays a circle with a label. Only one radio button
// in a group can be selected at a time. Radio buttons with the same group_id
// form a mutually exclusive group.
//
// Visual appearance:
//   ( ) Unselected option
//   (•) Selected option
//
// Architecture: Uses Cluster trait for shared button group behavior
//
// Usage:
//   let radio1 = RadioButton::new(
//       Rect::new(3, 5, 20, 6),
//       "Option 1",
//       1,  // group_id
//   );

use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::View;
use super::cluster::{Cluster, ClusterState};

/// RadioButton - A mutually exclusive selection control with a label
///
/// Now implements Cluster trait for standard button group behavior.
/// Matches Borland: TRadioButtons (extends TCluster)
#[derive(Debug)]
pub struct RadioButton {
    bounds: Rect,
    label: String,
    cluster_state: ClusterState,
    state: StateFlags,
    owner: Option<*const dyn View>,
}

impl RadioButton {
    /// Create a new radio button with the given bounds, label, and group ID
    ///
    /// Radio buttons with the same group_id are mutually exclusive.
    pub fn new(bounds: Rect, label: &str, group_id: u16) -> Self {
        RadioButton {
            bounds,
            label: label.to_string(),
            cluster_state: ClusterState::with_group(group_id),
            state: 0,
            owner: None,
        }
    }

    /// Set the selected state
    pub fn set_selected(&mut self, selected: bool) {
        self.cluster_state.set_value(if selected { 1 } else { 0 });
    }

    /// Get the selected state
    pub fn is_selected(&self) -> bool {
        self.cluster_state.value != 0
    }

    /// Select this radio button (should deselect others in the group)
    pub fn select(&mut self) {
        self.cluster_state.set_value(1);
    }

    /// Deselect this radio button
    pub fn deselect(&mut self) {
        self.cluster_state.set_value(0);
    }
}

impl View for RadioButton {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Use Cluster trait's standard event handling
        self.handle_cluster_event(event);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Use Cluster trait's standard drawing
        self.draw_cluster(terminal);
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

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_CLUSTER))
    }
}

// Implement Cluster trait
impl Cluster for RadioButton {
    fn cluster_state(&self) -> &ClusterState {
        &self.cluster_state
    }

    fn cluster_state_mut(&mut self) -> &mut ClusterState {
        &mut self.cluster_state
    }

    fn get_label(&self) -> &str {
        &self.label
    }

    fn get_marker(&self) -> &str {
        if self.is_selected() {
            "(•) "
        } else {
            "( ) "
        }
    }

    /// Radio buttons select (don't toggle) on space
    fn on_space_pressed(&mut self) {
        self.select();
        // TODO: Parent should deselect other radio buttons in the same group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radiobutton_creation() {
        let radio = RadioButton::new(Rect::new(0, 0, 20, 1), "Option 1", 1);
        assert!(!radio.is_selected());
        assert_eq!(radio.label, "Option 1");
        assert_eq!(radio.group_id(), 1);
    }

    #[test]
    fn test_radiobutton_select() {
        let mut radio = RadioButton::new(Rect::new(0, 0, 20, 1), "Option 1", 1);
        assert!(!radio.is_selected());

        radio.select();
        assert!(radio.is_selected());

        radio.deselect();
        assert!(!radio.is_selected());
    }

    #[test]
    fn test_radiobutton_set_selected() {
        let mut radio = RadioButton::new(Rect::new(0, 0, 20, 1), "Option 1", 1);

        radio.set_selected(true);
        assert!(radio.is_selected());

        radio.set_selected(false);
        assert!(!radio.is_selected());
    }

    #[test]
    fn test_radiobutton_group_id() {
        let radio1 = RadioButton::new(Rect::new(0, 0, 20, 1), "Option 1", 1);
        let radio2 = RadioButton::new(Rect::new(0, 1, 20, 2), "Option 2", 1);
        let radio3 = RadioButton::new(Rect::new(0, 2, 20, 3), "Option 3", 2);

        assert_eq!(radio1.group_id(), 1);
        assert_eq!(radio2.group_id(), 1);
        assert_eq!(radio3.group_id(), 2);
    }
}
