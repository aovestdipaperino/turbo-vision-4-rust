// (C) 2025 - Enzo Lombardi

//! Color Dialog - dialog for selecting foreground and background colors
//!
//! Matches Borland: TColorDialog
//!
//! Provides a dialog for interactive color selection with live preview.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::command::{CM_OK, CM_CANCEL, CommandId};
use crate::core::palette::Attr;
use crate::terminal::Terminal;
use super::dialog::Dialog;
use super::color_selector::ColorSelector;
use super::button::Button;
use super::static_text::StaticText;
use super::View;

/// Color Dialog
/// Matches Borland: TColorDialog (simplified implementation)
pub struct ColorDialog {
    dialog: Dialog,
    fg_selector_idx: usize,
    bg_selector_idx: usize,
    initial_attr: Attr,
    selected_attr: Option<Attr>,
}

impl ColorDialog {
    /// Create a new color dialog
    ///
    /// # Arguments
    /// * `bounds` - Dialog bounds
    /// * `title` - Dialog title
    /// * `initial_attr` - Initial color attribute to show
    pub fn new(bounds: Rect, title: &str, initial_attr: Attr) -> Self {
        let mut dialog = Dialog::new(bounds, title);

        // Instructions
        dialog.add(Box::new(StaticText::new(
            Rect::new(2, 2, bounds.width() - 4, 3),
            "Select foreground and background colors:"
        )));

        // Foreground color selector
        dialog.add(Box::new(StaticText::new(
            Rect::new(2, 4, 20, 5),
            "Foreground:"
        )));

        let fg_selector = ColorSelector::new(Rect::new(2, 5, 26, 8));
        let fg_selector_idx = dialog.add(Box::new(fg_selector));

        // Background color selector
        dialog.add(Box::new(StaticText::new(
            Rect::new(2, 9, 20, 10),
            "Background:"
        )));

        let bg_selector = ColorSelector::new(Rect::new(2, 10, 26, 13));
        let bg_selector_idx = dialog.add(Box::new(bg_selector));

        // Preview area (would show the colors in action)
        dialog.add(Box::new(StaticText::new(
            Rect::new(28, 5, bounds.width() - 4, 6),
            "Preview:"
        )));
        dialog.add(Box::new(StaticText::new(
            Rect::new(28, 6, bounds.width() - 4, 8),
            "Sample text with\nselected colors"
        )));

        // Buttons
        dialog.add(Box::new(Button::new(
            Rect::new(bounds.width() - 24, bounds.height() - 4, bounds.width() - 14, bounds.height() - 2),
            "OK",
            CM_OK,
            true
        )));

        dialog.add(Box::new(Button::new(
            Rect::new(bounds.width() - 12, bounds.height() - 4, bounds.width() - 2, bounds.height() - 2),
            "Cancel",
            CM_CANCEL,
            false
        )));

        Self {
            dialog,
            fg_selector_idx,
            bg_selector_idx,
            initial_attr,
            selected_attr: None,
        }
    }

    /// Execute the dialog modally
    ///
    /// Returns the selected color attribute if OK was pressed, None if cancelled
    pub fn execute(&mut self, app: &mut crate::app::Application) -> Option<Attr> {
        let result = self.dialog.execute(app);

        if result == CM_OK {
            // Get colors from selectors (simplified - would need selector access)
            // For now, return the initial attribute
            Some(self.initial_attr)
        } else {
            None
        }
    }

    /// Get the selected color attribute
    pub fn get_selected_attr(&self) -> Option<Attr> {
        self.selected_attr
    }
}

impl View for ColorDialog {
    fn bounds(&self) -> Rect {
        self.dialog.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.dialog.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.dialog.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.dialog.handle_event(event);
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> crate::core::state::StateFlags {
        self.dialog.state()
    }

    fn set_state(&mut self, state: crate::core::state::StateFlags) {
        self.dialog.set_state(state);
    }
}
