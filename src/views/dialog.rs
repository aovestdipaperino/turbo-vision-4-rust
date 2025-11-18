// (C) 2025 - Enzo Lombardi

//! Dialog view - modal window for user interaction with OK/Cancel buttons.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_ESC_ESC, KB_ENTER};
use crate::core::command::{CommandId, CM_CANCEL};
use crate::terminal::Terminal;
use super::view::{View, ViewId};
use super::window::Window;
use std::time::Duration;

pub struct Dialog {
    window: Window,
    result: CommandId,
}

impl Dialog {
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            window: Window::new_for_dialog(bounds, title),
            result: CM_CANCEL,
        }
    }

    /// Create a new modal dialog for use with Application::exec_view()
    /// Matches Borland pattern: Dialog is created with SF_MODAL set, then passed to execView()
    pub fn new_modal(bounds: Rect, title: &str) -> Box<Self> {
        use crate::core::state::SF_MODAL;
        let mut dialog = Self::new(bounds, title);
        let current_state = dialog.state();
        dialog.set_state(current_state | SF_MODAL);
        Box::new(dialog)
    }

    pub fn add(&mut self, view: Box<dyn View>) -> ViewId {
        self.window.add(view)
    }

    pub fn set_initial_focus(&mut self) {
        self.window.set_initial_focus();
    }

    /// Set focus to a specific child by index
    /// Matches Borland: owner->setCurrent(this, normalSelect)
    pub fn set_focus_to_child(&mut self, index: usize) {
        self.window.set_focus_to_child(index);
    }

    /// Get the number of child views
    pub fn child_count(&self) -> usize {
        self.window.child_count()
    }

    /// Get a reference to a child view by index
    pub fn child_at(&self, index: usize) -> &dyn View {
        self.window.child_at(index)
    }

    /// Get a mutable reference to a child view by index
    pub fn child_at_mut(&mut self, index: usize) -> &mut dyn View {
        self.window.child_at_mut(index)
    }

    /// Get an immutable reference to a child by its ViewId
    /// Returns None if the ViewId is not found
    pub fn child_by_id(&self, view_id: ViewId) -> Option<&dyn View> {
        self.window.child_by_id(view_id)
    }

    /// Get a mutable reference to a child by its ViewId
    /// Returns None if the ViewId is not found
    pub fn child_by_id_mut(&mut self, view_id: ViewId) -> Option<&mut (dyn View + '_)> {
        self.window.child_by_id_mut(view_id)
    }

    /// Remove a child by its ViewId
    /// Returns true if a child was found and removed, false otherwise
    pub fn remove_by_id(&mut self, view_id: ViewId) -> bool {
        self.window.remove_by_id(view_id)
    }

    /// Set the dialog title
    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    /// Get the current end_state (0 if dialog is still running, command ID if ended)
    /// Used by custom execute() loops to check if dialog should close
    /// Matches Borland: TGroup::endState field
    pub fn get_end_state(&self) -> CommandId {
        self.window.get_end_state()
    }

    /// Execute the dialog with its own event loop (self-contained pattern)
    ///
    /// **Two execution patterns supported:**
    ///
    /// **Pattern 1: Self-contained (simpler, for direct use):**
    /// ```ignore
    /// let mut dialog = Dialog::new(bounds, "Title");
    /// dialog.add(Button::new(...));
    /// let result = dialog.execute(&mut app);  // Runs own event loop
    /// ```
    ///
    /// **Pattern 2: Centralized (Borland-style, via Application::exec_view):**
    /// ```ignore
    /// let mut dialog = Dialog::new_modal(bounds, "Title");
    /// dialog.add(Button::new(...));
    /// let result = app.exec_view(dialog);  // App runs the modal loop
    /// ```
    ///
    /// Both patterns work identically. Pattern 1 is simpler for standalone use.
    /// Pattern 2 matches Borland's TProgram::execView() architecture.
    pub fn execute(&mut self, app: &mut crate::app::Application) -> CommandId {
        use crate::core::state::SF_MODAL;

        self.result = CM_CANCEL;

        // Set modal flag - dialogs are modal by default
        // Matches Borland: TDialog in modal state (tdialog.cc)
        let old_state = self.state();
        self.set_state(old_state | SF_MODAL);

        // Set explicit drag limits from desktop bounds
        // This allows modal dialogs to be constrained even though they're not added to desktop
        // Matches Borland: TView::dragView() uses owner's bounds as limits
        let desktop_bounds = app.desktop.get_bounds();
        self.window.set_drag_limits(desktop_bounds);

        // Constrain dialog position to desktop bounds (including shadow)
        // This ensures dialog is positioned within valid area when execute() is called
        // Matches Borland: TView::locate() constrains position to owner bounds
        self.window.constrain_to_limits();

        // Event loop matching Borland's TGroup::execute() (tgroup.cc:182-195)
        // IMPORTANT: We can't just delegate to window.execute() because that would
        // call Group::handle_event(), but we need Dialog::handle_event() to be called
        // (to handle commands and call end_modal).
        //
        // In Borland, TDialog inherits from TGroup, so TGroup::execute() calls
        // TDialog::handleEvent() via virtual function dispatch.
        //
        // In Rust with composition, we must implement the execute loop here
        // and call self.handle_event() to get proper polymorphic behavior.
        loop {
            // Draw desktop first (clears the background), then draw this dialog on top
            // This is the key: dialogs that aren't on the desktop need to draw themselves
            app.desktop.draw(&mut app.terminal);

            // Draw menu bar and status line if present (so they appear on top)
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.draw(&mut app.terminal);
            }
            if let Some(ref mut status_line) = app.status_line {
                status_line.draw(&mut app.terminal);
            }

            // Draw the dialog on top of desktop/menu/status
            self.draw(&mut app.terminal);

            // Draw overlay widgets on top of everything (animations, etc.)
            // These continue to animate even during modal dialogs
            // Matches Borland: TProgram::idle() continues running during execView()
            for widget in &mut app.overlay_widgets {
                widget.draw(&mut app.terminal);
            }

            self.update_cursor(&mut app.terminal);
            let _ = app.terminal.flush();

            // Poll for event with 20ms timeout (matches magiblot's eventTimeoutMs)
            // This blocks until an event arrives or timeout occurs
            match app.terminal.poll_event(Duration::from_millis(20)).ok().flatten() {
                Some(mut event) => {
                    // Event received - handle it immediately without calling idle()
                    // Matches magiblot: idle() is NOT called when events are present
                    self.handle_event(&mut event);

                    // If the event was converted to a command (e.g., KB_ENTER -> CM_OK),
                    // we need to process it again so the command handler runs
                    // Matches Borland: putEvent() re-queues the converted event
                    if event.what == EventType::Command {
                        self.handle_event(&mut event);
                    }
                }
                None => {
                    // Timeout with no events - call idle() to update animations, etc.
                    // Matches magiblot: idle() only called when truly idle
                    app.idle();
                }
            }

            // Check if dialog should close
            // Dialog::handle_event() calls window.end_modal() which sets the Group's end_state
            let end_state = self.window.get_end_state();
            if end_state != 0 {
                self.result = end_state;
                break;
            }
        }

        self.result
    }
}

impl View for Dialog {
    fn bounds(&self) -> Rect {
        self.window.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.window.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.window.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // First let the window (and its children) handle the event
        // This is critical: if a focused Memo/Editor handles Enter, it will clear the event
        // Borland's TDialog calls TWindow::handleEvent() FIRST (tdialog.cc line 47)
        self.window.handle_event(event);

        // Now check if the event is still active after children processed it
        // If a child (like Memo/Editor) handled Enter, event.what will be EventType::None
        // This matches Borland's TDialog architecture (tdialog.cc lines 48-86)

        // Handle Keyboard events (if not already handled by children)
        // IMPORTANT: Only handle dialog-specific keys when modal!
        // Non-modal dialogs should let keyboard events pass to parent handlers
        // Matches Borland: TDialog::handleEvent() (tdialog.cc:48-86)
        if event.what == EventType::Keyboard {
            use crate::core::state::SF_MODAL;

            // Only intercept keyboard shortcuts if this dialog is modal
            if self.state() & SF_MODAL != 0 {
                // ESC ESC always closes modal dialogs with CM_CANCEL
                // Matches Borland: cmCancel on Esc-Esc (tdialog.cc:71-73)
                if event.key_code == KB_ESC_ESC {
                    *event = Event::command(CM_CANCEL);
                    // Re-process as command (will be handled below)
                    self.handle_event(event);
                    return;
                }

                // Enter key activates default button (if exists and enabled)
                // Matches Borland: cmDefault broadcast (tdialog.cc:66-70)
                if event.key_code == KB_ENTER {
                    if let Some(default_command) = self.find_default_button_command() {
                        *event = Event::command(default_command);
                        // Re-process as command (will be handled below)
                        self.handle_event(event);
                    }
                    return;
                }
            }
            // If not modal, let keyboard events pass through to default handling
        }

        // Handle command events
        // Dialogs intercept cmCancel and cmOK/cmYes/cmNo to end the modal loop
        // IMPORTANT: Custom commands from child views (like ListBox) should NOT close the dialog
        // Only the standard dialog commands should close the modal loop
        // IMPORTANT: Only intercept commands when dialog is actually modal!
        // Non-modal dialogs (added to desktop) should let commands pass through
        // Matches Borland: TDialog::handleEvent() checks for these commands
        if event.what == EventType::Command {
            use crate::core::command::{CM_CANCEL, CM_OK, CM_YES, CM_NO};
            use crate::core::state::SF_MODAL;

            // Only intercept commands if this dialog is modal
            if self.state() & SF_MODAL != 0 {
                match event.command {
                    CM_CANCEL => {
                        // Cancel button or Esc-Esc pressed
                        // End the modal loop with CM_CANCEL
                        // Matches Borland: endModal(cmCancel)
                        self.window.end_modal(CM_CANCEL);
                        event.clear();
                    }
                    CM_OK | CM_YES | CM_NO => {
                        // OK/Yes/No button pressed
                        // End the modal loop with the command
                        // Matches Borland: endModal(command)
                        self.window.end_modal(event.command);
                        event.clear();
                    }
                    _ => {
                        // Other commands - distinguish between button commands and internal commands
                        // Button commands (< 1000): Custom button commands like 1, 2, 3
                        //   These should end the modal loop and return to caller
                        // Internal commands (>= 1000): Commands from child views like CMD_FILE_SELECTED (1000)
                        //   These are used by specific dialog implementations (FileDialog, etc.)
                        //   and should NOT close the dialog - let them pass through
                        //
                        // Convention: Commands >= 1000 are internal/custom view commands
                        //            Commands < 1000 are dialog close commands
                        if event.command < 1000 {
                            // Custom button command - end modal and return to caller
                            self.window.end_modal(event.command);
                            event.clear();
                        }
                        // else: Internal command >= 1000 - pass through to caller without closing
                    }
                }
            }
            // If not modal, let commands pass through unchanged
        }
    }

    fn state(&self) -> crate::core::state::StateFlags {
        self.window.state()
    }

    fn set_state(&mut self, state: crate::core::state::StateFlags) {
        self.window.set_state(state);
    }

    fn options(&self) -> u16 {
        self.window.options()
    }

    fn set_options(&mut self, options: u16) {
        self.window.set_options(options);
    }

    fn can_focus(&self) -> bool {
        // Dialogs can receive focus
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.window.set_focus(focused);
    }

    fn update_cursor(&self, terminal: &mut Terminal) {
        self.window.update_cursor(terminal);
    }

    fn valid(&mut self, command: CommandId) -> bool {
        // Dialogs validate on OK/Yes (but not Cancel/No)
        // Matches Borland: TDialog::valid() (tdialog.cc:88-104)
        if command == CM_CANCEL || command == 13 /* CM_NO */ {
            // Cancel/No always succeeds without validation
            return true;
        } else {
            // Validate through window (which will validate all children)
            self.window.valid(command)
        }
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.window.set_owner(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.window.get_owner()
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        // Dialog uses gray dialog palette (Borland: TDialog::getPalette)
        Some(Palette::from_slice(palettes::CP_GRAY_DIALOG))
    }

    fn init_after_add(&mut self) {
        // Initialize Window's interior owner pointer now that Dialog is in final position
        // This completes the palette chain: Button → interior → Window → Desktop
        self.window.init_interior_owner();
    }

    fn constrain_to_parent_bounds(&mut self) {
        self.window.constrain_to_limits();
    }

    fn get_end_state(&self) -> crate::core::command::CommandId {
        self.window.get_end_state()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Dialog {
    /// Find the default button and return its command if it's enabled
    /// Returns None if no default button found or if it's disabled
    /// Matches Borland's TButton::handleEvent() cmDefault broadcast handling (tbutton.cc lines 238-244)
    fn find_default_button_command(&self) -> Option<CommandId> {
        for i in 0..self.child_count() {
            let child = self.child_at(i);
            if child.is_default_button() {
                // Check if the button can receive focus (i.e., not disabled)
                // Borland checks: amDefault && !(state & sfDisabled)
                if child.can_focus() {
                    return child.button_command();
                } else {
                    // Default button is disabled
                    return None;
                }
            }
        }
        None
    }
}

/// Builder for creating dialogs with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::dialog::DialogBuilder;
/// use turbo_vision::views::button::ButtonBuilder;
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision::core::command::CM_OK;
///
/// // Create a regular dialog
/// let mut dialog = DialogBuilder::new()
///     .bounds(Rect::new(10, 5, 50, 15))
///     .title("My Dialog")
///     .build();
///
/// // Create a modal dialog (boxed)
/// let dialog = DialogBuilder::new()
///     .bounds(Rect::new(10, 5, 50, 15))
///     .title("Modal Dialog")
///     .modal(true)
///     .build_boxed();
/// ```
pub struct DialogBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    modal: bool,
}

impl DialogBuilder {
    /// Creates a new DialogBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            modal: false,
        }
    }

    /// Sets the dialog bounds (required).
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

    /// Sets whether the dialog should be modal (default: false).
    /// Modal dialogs are created with SF_MODAL flag set.
    #[must_use]
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Builds the Dialog.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title) are not set.
    pub fn build(self) -> Dialog {
        let bounds = self.bounds.expect("Dialog bounds must be set");
        let title = self.title.expect("Dialog title must be set");

        let mut dialog = Dialog::new(bounds, &title);

        if self.modal {
            use crate::core::state::SF_MODAL;
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);
        }

        dialog
    }

    /// Builds the Dialog as a Box (for use with Application::exec_view).
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, title) are not set.
    pub fn build_boxed(self) -> Box<Dialog> {
        Box::new(self.build())
    }
}

impl Default for DialogBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::SF_MODAL;

    /// Regression test for FileDialog folder navigation bug (issue #73 follow-up)
    ///
    /// The bug: Dialog was calling end_modal() for ALL commands (including CMD_FILE_SELECTED = 1000),
    /// which caused FileDialog to close when double-clicking folders instead of navigating into them.
    ///
    /// The fix: Dialog now only calls end_modal() for commands < 1000 (dialog close commands).
    /// Commands >= 1000 (internal/child view commands) pass through without closing the dialog.
    ///
    /// This test verifies:
    /// 1. Internal commands (>= 1000) do NOT close modal dialogs
    /// 2. Custom button commands (< 1000) DO close modal dialogs
    #[test]
    fn test_dialog_command_handling() {
        // Test 1: Internal command (>= 1000) should NOT close dialog
        {
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);

            // Simulate an internal command like CMD_FILE_SELECTED (1000)
            let mut event = Event::command(1000);
            dialog.handle_event(&mut event);

            // Dialog should NOT close (end_state should remain 0)
            assert_eq!(
                dialog.get_end_state(),
                0,
                "Internal command (1000) should not close dialog"
            );

            // Event should still be available (not cleared)
            assert_eq!(
                event.what,
                EventType::Command,
                "Internal command event should not be cleared"
            );
            assert_eq!(
                event.command, 1000,
                "Internal command should remain unchanged"
            );
        }

        // Test 2: Custom button command (< 1000) should close dialog
        {
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);

            // Simulate a custom button command (e.g., 100)
            let mut event = Event::command(100);
            dialog.handle_event(&mut event);

            // Dialog SHOULD close (end_state should be set to the command)
            assert_eq!(
                dialog.get_end_state(),
                100,
                "Custom button command (100) should close dialog"
            );

            // Event should be cleared
            assert_eq!(
                event.what,
                EventType::Nothing,
                "Custom button command event should be cleared"
            );
        }

        // Test 3: Boundary test - command 999 should close, 1000 should not
        {
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);

            let mut event = Event::command(999);
            dialog.handle_event(&mut event);

            assert_eq!(
                dialog.get_end_state(),
                999,
                "Command 999 should close dialog (< 1000)"
            );
            assert_eq!(
                event.what,
                EventType::Nothing,
                "Command 999 event should be cleared"
            );
        }

        {
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);

            let mut event = Event::command(1000);
            dialog.handle_event(&mut event);

            assert_eq!(
                dialog.get_end_state(),
                0,
                "Command 1000 should not close dialog (>= 1000)"
            );
            assert_eq!(
                event.what,
                EventType::Command,
                "Command 1000 event should not be cleared"
            );
        }

        // Test 4: Standard commands (OK, Cancel, etc.) should still work
        {
            use crate::core::command::{CM_OK, CM_CANCEL, CM_YES, CM_NO};

            for cmd in [CM_OK, CM_CANCEL, CM_YES, CM_NO] {
                let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
                let current_state = dialog.state();
                dialog.set_state(current_state | SF_MODAL);

                let mut event = Event::command(cmd);
                dialog.handle_event(&mut event);

                assert_eq!(
                    dialog.get_end_state(),
                    cmd,
                    "Standard command {} should close dialog",
                    cmd
                );
                assert_eq!(
                    event.what,
                    EventType::Nothing,
                    "Standard command {} event should be cleared",
                    cmd
                );
            }
        }
    }

    /// Test that non-modal dialogs don't interfere with command handling
    #[test]
    fn test_non_modal_dialog_commands() {
        let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
        // Don't set SF_MODAL - this is a non-modal dialog

        // Non-modal dialogs should not call end_modal() for any command
        let mut event = Event::command(100);
        dialog.handle_event(&mut event);

        // end_state should remain 0 because dialog is not modal
        assert_eq!(
            dialog.get_end_state(),
            0,
            "Non-modal dialog should not set end_state"
        );

        // Commands should pass through unchanged
        let mut event = Event::command(1000);
        dialog.handle_event(&mut event);
        assert_eq!(
            dialog.get_end_state(),
            0,
            "Non-modal dialog should not set end_state for internal commands"
        );
    }
}
