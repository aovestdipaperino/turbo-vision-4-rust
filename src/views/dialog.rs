// (C) 2025 - Enzo Lombardi

//! Dialog view - modal window for user interaction with OK/Cancel buttons.

use super::view::{View, ViewCore, ViewId};
use super::window::Window;
use crate::core::command::{CM_CANCEL, CommandId};
use crate::core::event::{Event, EventType, KB_ENTER, KB_ESC_ESC};
use crate::core::geometry::Rect;
use crate::terminal::Terminal;
use std::time::{Duration, Instant};

pub struct Dialog {
    window: Window,
    result: CommandId,
    /// When set, `execute` closes the dialog with the given command once
    /// this much time has passed without the user closing it first.
    auto_dismiss: Option<(Duration, CommandId)>,
}

impl Dialog {
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            window: Window::new_for_dialog(bounds, title),
            result: CM_CANCEL,
            auto_dismiss: None,
        }
    }

    /// Close the dialog automatically with `command` once `timeout` has
    /// elapsed in `execute`, unless the user closes it earlier.
    ///
    /// The timer starts when `execute` is entered. Only the `execute`
    /// modal loop honours it; a dialog run through `Application::exec_view`
    /// is unaffected.
    pub fn set_auto_dismiss(&mut self, timeout: Duration, command: CommandId) {
        self.auto_dismiss = Some((timeout, command));
    }

    /// The auto-dismiss timeout and command, if one was set.
    pub fn auto_dismiss(&self) -> Option<(Duration, CommandId)> {
        self.auto_dismiss
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

    /// Set whether the dialog is resizable.
    /// Resizable dialogs show single-line bottom corners and a resize handle.
    /// By default, dialogs are not resizable (matching Borland's TDialog).
    pub fn set_resizable(&mut self, resizable: bool) {
        self.window.set_resizable(resizable);
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

        // Set initial focus to the first focusable child
        // Matches Borland: TView::setState(sfVisible) calls owner->resetCurrent()
        // which selects the first visible, selectable child when views are added
        self.set_initial_focus();

        let started = Instant::now();

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
            // Create a fresh palette token for this frame

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
            match app
                .terminal
                .poll_event(Duration::from_millis(20))
                .ok()
                .flatten()
            {
                Some(mut event) => {
                    // Handle CM_REDRAW at the application level first
                    if event.what == EventType::Broadcast
                        && event.command == crate::core::command::CM_REDRAW
                    {
                        app.handle_redraw();
                        continue;
                    }

                    // Event received - handle it immediately without calling idle()
                    // Matches magiblot: idle() is NOT called when events are present
                    self.handle_event(&mut event);

                    // If the event was converted to a command (e.g., KB_ENTER -> CM_OK),
                    // we need to process it again so the command handler runs
                    // Matches Borland: putEvent() re-queues the converted event
                    if event.what == EventType::Command {
                        self.handle_event(&mut event);
                    }

                    // A History button converted its click into CM_SHOW_HISTORY;
                    // open the popup here, where we have terminal access.
                    if event.what == EventType::Command
                        && event.command == crate::core::command::CM_SHOW_HISTORY
                    {
                        self.show_history_popup(&mut event, &mut app.terminal);
                    }

                    // Same for a ComboBox asking to drop its list down.
                    if event.what == EventType::Command
                        && event.command == crate::core::command::CM_SHOW_DROPDOWN
                    {
                        show_dropdown_popup(&mut event, &mut app.terminal);
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
                // Matches Borland: do { ... } while( !valid(endState) ) — a
                // failing validator vetoes the close and re-enters the loop
                // (Dialog::valid already exempts Cancel/No)
                if self.valid(end_state) {
                    self.result = end_state;
                    break;
                }
                self.window.set_end_state(0);
            }

            // Auto-dismiss: the user did not close the dialog in time, so
            // close it on their behalf with the configured command.
            if let Some((timeout, command)) = self.auto_dismiss {
                if started.elapsed() >= timeout {
                    self.result = command;
                    break;
                }
            }
        }

        self.result
    }

    /// Open the history popup for a `CM_SHOW_HISTORY` command event.
    ///
    /// Runs the `HistoryWindow` modally on the given terminal. If the user picks
    /// an item, it is moved to the front of the history list and a
    /// `CM_HISTORY_SELECTED` broadcast is dispatched to the dialog's children so
    /// the linked `History` view copies it into its InputLine's shared data.
    fn show_history_popup(&mut self, event: &mut Event, terminal: &mut Terminal) {
        use crate::core::command::CM_HISTORY_SELECTED;
        use crate::core::geometry::Point;
        use crate::core::history::HistoryManager;
        use crate::views::history_window::HistoryWindow;

        let history_id = event.info;
        // Place the popup just below the clicked button, shifted left so the
        // dropdown covers the input line it belongs to.
        let pos = Point::new((event.mouse.pos.x - 20).max(0), event.mouse.pos.y + 1);
        let mut window = HistoryWindow::new(pos, history_id, 30);
        if let Some(selected) = window.execute(terminal) {
            // Move the selection to the front so History views can find it.
            HistoryManager::add(history_id, selected);
            let mut sel = Event::broadcast_with_info(CM_HISTORY_SELECTED, history_id);
            self.window.handle_event(&mut sel);
        }
        event.clear();
    }
}

/// Open the drop-down list for a `CM_SHOW_DROPDOWN` command event.
///
/// The combo box registered its items under `event.info`; the popup writes the
/// user's choice straight back into that shared state, so nothing needs to be
/// broadcast afterwards. An unknown id is ignored, which is what happens when
/// the control was dropped between the click and this call.
///
/// Free function rather than a method because `Application` runs the same step
/// for combo boxes living on plain windows.
pub(crate) fn show_dropdown_popup(event: &mut Event, terminal: &mut Terminal) {
    use crate::views::combo_box::{DropdownWindow, lookup};

    if let Some(state) = lookup(event.info) {
        let (w, h) = terminal.size();
        let screen = Rect::new(0, 0, w as i16, h as i16);
        DropdownWindow::new(state, screen).execute(terminal);
    }
    event.clear();
}

impl View for Dialog {
    fn core(&self) -> &ViewCore {
        self.window.core()
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        self.window.core_mut()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        // Forwarded explicitly: the inner view's `set_bounds` cascades to its
        // children, which the `ViewCore` default would bypass.
        self.window.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.window.draw(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // First let the window (and its children) handle the event
        // This is critical: if a focused Memo/EditorWindow handles Enter, it will clear the event
        // Borland's TDialog calls TWindow::handleEvent() FIRST (tdialog.cc line 47)
        self.window.handle_event(event);

        // Now check if the event is still active after children processed it
        // If a child (like Memo/EditorWindow) handled Enter, event.what will be EventType::None
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

                // Enter key activates the *current* default button.
                // Matches Borland: cmDefault broadcast (tdialog.cc:66-70) where a
                // focused button has grabbed the default role (cmGrabDefault).
                //
                // If the focused child is a button it consumes Enter itself
                // (converting it to its own command in the Group's focused
                // phase), so this branch is only reached when the focused view
                // did not handle Enter. Guard anyway: never fire the flagged
                // default while a different button is focused.
                if event.key_code == KB_ENTER {
                    if !self.focused_child_is_button() {
                        if let Some(default_command) = self.find_default_button_command() {
                            *event = Event::command(default_command);
                            // Re-process as command (will be handled below)
                            self.handle_event(event);
                        }
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
            use crate::core::command::{CM_CANCEL, CM_NO, CM_OK, CM_YES};
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
                        // On accept (OK/Yes, not No/Cancel), broadcast CM_RECORD_HISTORY
                        // so History views record their linked InputLine data.
                        // Matches Borland: TButton::press() message(owner, evBroadcast,
                        // cmRecordHistory, 0) before emitting the command.
                        if event.command == CM_OK || event.command == CM_YES {
                            let mut record =
                                Event::broadcast(crate::core::command::CM_RECORD_HISTORY);
                            self.window.handle_event(&mut record);
                        }
                        // End the modal loop with the command
                        // Matches Borland: endModal(command)
                        self.window.end_modal(event.command);
                        event.clear();
                    }
                    crate::core::command::CM_SHOW_HISTORY
                    | crate::core::command::CM_SHOW_DROPDOWN => {
                        // A History button was clicked, or a ComboBox asked to
                        // drop its list. Leave the event alone so the modal loop
                        // in Dialog::execute() (which has terminal access) can
                        // open the popup. Must not fall through to the
                        // "< 1000 closes the dialog" rule below.
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
        if command == CM_CANCEL || command == 13
        /* CM_NO */
        {
            // Cancel/No always succeeds without validation
            return true;
        } else {
            // Validate through window (which will validate all children)
            self.window.valid(command)
        }
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
    /// Returns true if the currently focused child view is a button.
    ///
    /// Used by the Enter handling to avoid firing the statically flagged
    /// default button while a different button owns the focus (Borland's
    /// cmGrabDefault semantics: the focused button *is* the current default).
    fn focused_child_is_button(&mut self) -> bool {
        self.window
            .interior_mut()
            .focused_child()
            .is_some_and(|child| child.is_focused() && child.button_command().is_some())
    }

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
///
/// // Create a resizable dialog (e.g. for FileDialog)
/// let mut dialog = DialogBuilder::new()
///     .bounds(Rect::new(10, 5, 60, 20))
///     .title("File Open")
///     .resizable(true)
///     .build();
/// ```
pub struct DialogBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    modal: bool,
    resizable: bool,
}

impl DialogBuilder {
    /// Creates a new DialogBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            modal: false,
            resizable: false,
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

    /// Sets whether the dialog is resizable (default: false).
    /// Resizable dialogs show single-line bottom corners and a resize handle.
    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
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

        if self.resizable {
            dialog.set_resizable(true);
        }

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

    #[test]
    fn auto_dismiss_is_off_by_default_and_settable() {
        let mut dialog = Dialog::new(Rect::new(0, 0, 20, 10), "T");
        assert_eq!(dialog.auto_dismiss(), None);
        dialog.set_auto_dismiss(Duration::from_secs(3), crate::core::command::CM_OK);
        assert_eq!(
            dialog.auto_dismiss(),
            Some((Duration::from_secs(3), crate::core::command::CM_OK))
        );
    }

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
            use crate::core::command::{CM_CANCEL, CM_NO, CM_OK, CM_YES};

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

    /// History views must record their linked InputLine data when the dialog
    /// is accepted with CM_OK (via the CM_RECORD_HISTORY broadcast), but not
    /// when it is cancelled.
    #[test]
    fn test_dialog_ok_records_history() {
        use crate::core::command::CM_OK;
        use crate::core::geometry::Point;
        use crate::core::history::HistoryManager;
        use crate::views::history::History;
        use std::cell::RefCell;
        use std::rc::Rc;

        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();

        let make_dialog = |text: &str| {
            let data = Rc::new(RefCell::new(text.to_string()));
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            let state = dialog.state();
            dialog.set_state(state | SF_MODAL);
            dialog.add(Box::new(History::new(Point::new(30, 2), 42, data)));
            dialog
        };

        // Cancel does NOT record
        let mut dialog = make_dialog("not recorded");
        let mut event = Event::command(CM_CANCEL);
        dialog.handle_event(&mut event);
        assert_eq!(HistoryManager::count(42), 0, "Cancel must not record");

        // OK records the linked data
        let mut dialog = make_dialog("recorded entry");
        let mut event = Event::command(CM_OK);
        dialog.handle_event(&mut event);
        assert_eq!(
            HistoryManager::get_list(42),
            vec!["recorded entry".to_string()]
        );
        assert_eq!(dialog.get_end_state(), CM_OK);
    }

    /// CM_SHOW_HISTORY must not be swallowed by the "< 1000 closes the dialog"
    /// rule; it is left pending so the modal loop can open the popup.
    #[test]
    fn test_dialog_show_history_command_passes_through() {
        use crate::core::command::CM_SHOW_HISTORY;

        let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
        let state = dialog.state();
        dialog.set_state(state | SF_MODAL);

        let mut event = Event::command(CM_SHOW_HISTORY);
        event.info = 42;
        dialog.handle_event(&mut event);

        assert_eq!(dialog.get_end_state(), 0, "must not close the dialog");
        assert_eq!(event.what, EventType::Command, "event left for modal loop");
        assert_eq!(event.command, CM_SHOW_HISTORY);
    }

    /// Enter must press the *focused* button, not the statically flagged
    /// default (Borland cmGrabDefault semantics).
    #[test]
    fn test_enter_fires_focused_button_not_flagged_default() {
        use crate::core::command::{CM_NO, CM_OK};
        use crate::core::command_set;
        use crate::core::event::KB_ENTER;
        use crate::views::button::Button;

        command_set::enable_command(CM_OK);
        command_set::enable_command(CM_NO);

        let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
        let state = dialog.state();
        dialog.set_state(state | SF_MODAL);

        dialog.add(Box::new(Button::new(
            Rect::new(2, 2, 12, 4),
            "OK",
            CM_OK,
            true, // flagged default
        )));
        dialog.add(Box::new(Button::new(
            Rect::new(14, 2, 24, 4),
            "No",
            CM_NO,
            false,
        )));
        dialog.set_focus_to_child(1); // focus the non-default button

        let mut event = Event::keyboard(KB_ENTER);
        dialog.handle_event(&mut event);

        assert_eq!(
            dialog.get_end_state(),
            CM_NO,
            "Enter must fire the focused button's command, not the flagged default"
        );
    }

    /// Enter with a non-button focused falls back to the flagged default button.
    #[test]
    fn test_enter_on_non_button_fires_default_button() {
        use crate::core::command::CM_OK;
        use crate::core::command_set;
        use crate::core::event::KB_ENTER;
        use crate::views::button::Button;
        use crate::views::static_text::StaticText;

        command_set::enable_command(CM_OK);

        let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
        let state = dialog.state();
        dialog.set_state(state | SF_MODAL);

        dialog.add(Box::new(StaticText::new(Rect::new(2, 2, 20, 3), "Hello")));
        dialog.add(Box::new(Button::new(
            Rect::new(2, 4, 12, 6),
            "OK",
            CM_OK,
            true,
        )));

        let mut event = Event::keyboard(KB_ENTER);
        dialog.handle_event(&mut event);

        assert_eq!(
            dialog.get_end_state(),
            CM_OK,
            "Enter with a non-button focused must fire the default button"
        );
    }

    #[test]
    fn test_dialog_set_resizable() {
        let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
        // Default: not resizable
        dialog.set_resizable(true);
        // Should not panic; verify bounds still valid
        assert_eq!(dialog.bounds(), Rect::new(0, 0, 40, 10));
    }

    #[test]
    fn test_dialog_builder_resizable() {
        let dialog = DialogBuilder::new()
            .bounds(Rect::new(5, 5, 50, 20))
            .title("Resizable Dialog")
            .resizable(true)
            .build();
        assert_eq!(dialog.bounds(), Rect::new(5, 5, 50, 20));
    }

    #[test]
    fn test_dialog_builder_resizable_modal() {
        let dialog = DialogBuilder::new()
            .bounds(Rect::new(5, 5, 50, 20))
            .title("Resizable Modal")
            .resizable(true)
            .modal(true)
            .build();
        assert_eq!(dialog.bounds(), Rect::new(5, 5, 50, 20));
        assert_ne!(dialog.state() & SF_MODAL, 0, "Should be modal");
    }
}
