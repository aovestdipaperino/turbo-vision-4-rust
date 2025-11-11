// (C) 2025 - Enzo Lombardi

//! CheckBox view - boolean selection control with checkable items.
// CheckBox - Boolean selection control
//
// Matches Borland: TCheckBoxes (extends TCluster)
//
// A checkbox control displays a box with a label. The box can be checked or unchecked
// by clicking on it or pressing Space when focused.
//
// Visual appearance:
//   [ ] Unchecked option
//   [X] Checked option
//
// Architecture: Uses Cluster trait for shared button group behavior
//
// Usage:
//   let checkbox = CheckBox::new(
//       Rect::new(3, 5, 20, 6),
//       "Enable feature",
//   );

use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::view::View;
use super::cluster::{Cluster, ClusterState};

/// CheckBox - A boolean selection control with a label
///
/// Now implements Cluster trait for standard button group behavior.
/// Matches Borland: TCheckBoxes (extends TCluster)
#[derive(Debug)]
pub struct CheckBox {
    bounds: Rect,
    label: String,
    cluster_state: ClusterState,
    state: StateFlags,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl CheckBox {
    /// Create a new checkbox with the given bounds and label
    pub fn new(bounds: Rect, label: &str) -> Self {
        CheckBox {
            bounds,
            label: label.to_string(),
            cluster_state: ClusterState::new(),
            state: 0,
            owner: None,
            owner_type: super::view::OwnerType::None,
        }
    }

    /// Set the checked state
    pub fn set_checked(&mut self, checked: bool) {
        self.cluster_state.set_value(if checked { 1 } else { 0 });
    }

    /// Get the checked state
    pub fn is_checked(&self) -> bool {
        self.cluster_state.value != 0
    }

    /// Toggle the checked state
    pub fn toggle(&mut self) {
        self.cluster_state.toggle();
    }
}

impl View for CheckBox {
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

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

// Implement Cluster trait
impl Cluster for CheckBox {
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
        if self.is_checked() {
            "[X] "
        } else {
            "[ ] "
        }
    }

    /// Checkboxes toggle on space (default behavior)
    fn on_space_pressed(&mut self) {
        self.toggle();
    }
}

/// Builder for creating checkboxes with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::checkbox::CheckBoxBuilder;
/// use turbo_vision::core::geometry::Rect;
///
/// // Create an unchecked checkbox
/// let checkbox = CheckBoxBuilder::new()
///     .bounds(Rect::new(3, 5, 30, 6))
///     .label("Enable feature")
///     .build();
///
/// // Create a pre-checked checkbox
/// let checkbox = CheckBoxBuilder::new()
///     .bounds(Rect::new(3, 6, 30, 7))
///     .label("Auto-save")
///     .checked(true)
///     .build();
/// ```
pub struct CheckBoxBuilder {
    bounds: Option<Rect>,
    label: Option<String>,
    checked: bool,
}

impl CheckBoxBuilder {
    /// Creates a new CheckBoxBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            label: None,
            checked: false,
        }
    }

    /// Sets the checkbox bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the checkbox label (required).
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets whether the checkbox is initially checked (default: false).
    #[must_use]
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Builds the CheckBox.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, label) are not set.
    pub fn build(self) -> CheckBox {
        let bounds = self.bounds.expect("CheckBox bounds must be set");
        let label = self.label.expect("CheckBox label must be set");

        let mut checkbox = CheckBox::new(bounds, &label);
        if self.checked {
            checkbox.set_checked(true);
        }
        checkbox
    }

    /// Builds the CheckBox as a Box.
    pub fn build_boxed(self) -> Box<CheckBox> {
        Box::new(self.build())
    }
}

impl Default for CheckBoxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_creation() {
        let checkbox = CheckBox::new(Rect::new(0, 0, 20, 1), "Test option");
        assert!(!checkbox.is_checked());
        assert_eq!(checkbox.label, "Test option");
    }

    #[test]
    fn test_checkbox_toggle() {
        let mut checkbox = CheckBox::new(Rect::new(0, 0, 20, 1), "Test");
        assert!(!checkbox.is_checked());

        checkbox.toggle();
        assert!(checkbox.is_checked());

        checkbox.toggle();
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn test_checkbox_set_checked() {
        let mut checkbox = CheckBox::new(Rect::new(0, 0, 20, 1), "Test");

        checkbox.set_checked(true);
        assert!(checkbox.is_checked());

        checkbox.set_checked(false);
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn test_checkbox_builder() {
        let checkbox = CheckBoxBuilder::new()
            .bounds(Rect::new(3, 5, 30, 6))
            .label("Test Feature")
            .build();

        assert_eq!(checkbox.label, "Test Feature");
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn test_checkbox_builder_checked() {
        let checkbox = CheckBoxBuilder::new()
            .bounds(Rect::new(3, 5, 30, 6))
            .label("Auto-save")
            .checked(true)
            .build();

        assert!(checkbox.is_checked());
    }
}
