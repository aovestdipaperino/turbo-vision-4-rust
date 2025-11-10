// (C) 2025 - Enzo Lombardi

//! Color Dialog - dialog for selecting foreground and background colors
//!
//! Matches Borland: TColorDialog
//!
//! Provides a dialog for interactive color selection with live preview.

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::command::{CM_OK, CM_CANCEL};
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
    _fg_selector_idx: usize,
    _bg_selector_idx: usize,
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
            _fg_selector_idx: fg_selector_idx,
            _bg_selector_idx: bg_selector_idx,
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

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        self.dialog.get_palette()
    }
}

/// Builder for creating color dialogs with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::color_dialog::ColorDialogBuilder;
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision::core::palette::Attr;
/// use turbo_vision::core::palette::TvColor;
///
/// // Create a color dialog with default colors
/// let dialog = ColorDialogBuilder::new()
///     .bounds(Rect::new(10, 5, 60, 20))
///     .title("Select Colors")
///     .build();
///
/// // Create a color dialog with initial attribute
/// let initial = Attr::new(TvColor::White, TvColor::Blue);
/// let dialog = ColorDialogBuilder::new()
///     .bounds(Rect::new(10, 5, 60, 20))
///     .title("Choose Colors")
///     .initial_attr(initial)
///     .build();
/// ```
pub struct ColorDialogBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    initial_attr: Attr,
}

impl ColorDialogBuilder {
    /// Creates a new ColorDialogBuilder with default values.
    pub fn new() -> Self {
        use crate::core::palette::TvColor;
        Self {
            bounds: None,
            title: None,
            initial_attr: Attr::new(TvColor::White, TvColor::Black),
        }
    }

    /// Sets the color dialog bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the dialog title (required).
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the initial color attribute (default: White on Black).
    #[must_use]
    pub fn initial_attr(mut self, attr: Attr) -> Self {
        self.initial_attr = attr;
        self
    }

    /// Builds the ColorDialog.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title) are not set.
    pub fn build(self) -> ColorDialog {
        let bounds = self.bounds.expect("ColorDialog bounds must be set");
        let title = self.title.expect("ColorDialog title must be set");
        ColorDialog::new(bounds, &title, self.initial_attr)
    }

    /// Builds the ColorDialog as a Box.
    pub fn build_boxed(self) -> Box<ColorDialog> {
        Box::new(self.build())
    }
}

impl Default for ColorDialogBuilder {
    fn default() -> Self {
        Self::new()
    }
}
