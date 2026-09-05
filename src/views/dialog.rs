// (C) 2025 - Enzo Lombardi

//! Dialog view - modal window for user interaction with OK/Cancel buttons.

use super::button::Button;
use super::group::Group;
use super::group::GroupLike;
use super::view::{View, ViewId};
use super::window::{Window, WindowLike};
use crate::app::ModalTick;
use crate::core::command::{CM_CANCEL, CommandId};
use crate::core::event::{Event, EventType, KB_ENTER, KB_ESC_ESC};
use crate::core::geometry::Rect;
use crate::terminal::Terminal;
use std::time::{Duration, Instant};

/// Which commands close a modal dialog (Borland: `TDialog::handleEvent`
/// ends the modal loop on cmOK, cmCancel, cmYes and cmNo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseOn {
    /// Borland's rule: `CM_OK`, `CM_CANCEL`, `CM_YES`, `CM_NO`.
    Standard,
    /// Standard plus every command carried by a `Button` added to this
    /// dialog. The default: a custom button closes the dialog with its own
    /// command, whatever its number.
    StandardAndButtons,
    /// Exactly this list.
    Commands(Vec<CommandId>),
}

pub struct Dialog {
    window: Window,
    result: CommandId,
    /// Which commands end the modal loop; see [`CloseOn`].
    close_on: CloseOn,
    /// Commands of the (non-broadcast) buttons added so far, for
    /// `CloseOn::StandardAndButtons`.
    button_commands: Vec<CommandId>,
    /// When set, `execute` closes the dialog with the given command once
    /// this much time has passed without the user closing it first.
    auto_dismiss: Option<(Duration, CommandId)>,
}

impl Dialog {
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            window: Window::new_for_dialog(bounds, title),
            result: CM_CANCEL,
            close_on: CloseOn::StandardAndButtons,
            button_commands: Vec::new(),
            auto_dismiss: None,
        }
    }

    /// Close the dialog automatically with `command` once `timeout` has
    /// elapsed in `execute`, unless the user closes it earlier.
    ///
    /// The timer starts when `execute` is entered. Only the `execute`
    /// modal loop honours it; a dialog run through `Application::exec_view`
    /// is unaffected.
    /// Choose which commands close the dialog; see [`CloseOn`].
    pub fn set_close_on(&mut self, policy: CloseOn) {
        self.close_on = policy;
    }

    pub fn close_on(&self) -> &CloseOn {
        &self.close_on
    }

    fn closes_on(&self, command: CommandId) -> bool {
        use crate::core::command::{CM_NO, CM_OK, CM_YES};
        let standard = matches!(command, CM_OK | CM_CANCEL | CM_YES | CM_NO);
        match &self.close_on {
            CloseOn::Standard => standard,
            CloseOn::StandardAndButtons => standard || self.button_commands.contains(&command),
            CloseOn::Commands(list) => list.contains(&command),
        }
    }

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

    /// Set focus to a specific child by index
    /// Matches Borland: owner->setCurrent(this, normalSelect)
    pub fn set_focus_to_child(&mut self, index: usize) {
        self.window.set_focus_to_child(index);
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
    /// Put the dialog into its modal state: `SF_MODAL`, drag limits from the
    /// desktop, position constrained to it, first focusable child focused.
    /// `execute` does this before entering the loop; a type wrapping a `Dialog`
    /// that runs `Application::execute_modal` on itself calls it directly.
    pub(crate) fn prepare_modal(&mut self, app: &mut crate::app::Application) {
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
        // Matches Borland: TView::locate() constrains position to owner bounds
        self.window.constrain_to_limits();

        // Set initial focus to the first focusable child
        // Matches Borland: TView::setState(sfVisible) calls owner->resetCurrent()
        self.set_initial_focus();
    }

    pub fn execute(&mut self, app: &mut crate::app::Application) -> CommandId {
        self.prepare_modal(app);

        // The loop itself is Application::execute_modal (Borland:
        // TGroup::execute); the closure is the only dialog-specific part.
        // Auto-dismiss: the user did not close the dialog in time, so close it
        // on their behalf with the configured command.
        let started = Instant::now();
        let auto = self.auto_dismiss;
        self.result = app.execute_modal(self, |_, _| match auto {
            Some((timeout, command)) if started.elapsed() >= timeout => ModalTick::End(command),
            _ => ModalTick::Continue,
        });

        self.result
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

impl GroupLike for Dialog {
    fn group(&self) -> &Group {
        self.window.group()
    }
    fn group_mut(&mut self) -> &mut Group {
        self.window.group_mut()
    }
    fn add_boxed(&mut self, view: Box<dyn View>) -> ViewId {
        // Remember each button's command so `CloseOn::StandardAndButtons` can
        // recognise it later. Broadcast buttons never end the dialog.
        if let Some(button) = view.as_any().downcast_ref::<Button>() {
            if !button.is_broadcast() {
                self.button_commands.push(button.command());
            }
        }
        self.window.add_boxed(view)
    }
}

impl WindowLike for Dialog {
    fn window(&self) -> &Window {
        &self.window
    }
    fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}

// TDialog overrides three TWindow methods: handleEvent, valid and getPalette
// (tdialog.cc). Everything else is the inherited TWindow behaviour, which the
// macro forwards to the window_* base implementations.
crate::impl_view_for_window!(Dialog {
    fn handle_event(&mut self, event: &mut Event) {
        // First let the window (and its children) handle the event
        // This is critical: if a focused Memo/EditorWindow handles Enter, it will clear the event
        // Borland's TDialog calls TWindow::handleEvent() FIRST (tdialog.cc line 47)
        self.window_handle_event(event);

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
                    CM_CANCEL | CM_OK | CM_YES | CM_NO if self.closes_on(event.command) => {
                        // The standard four end the modal loop (Borland:
                        // endModal(command)). On accept (OK/Yes, not No/Cancel),
                        // record every History's linked InputLine text first.
                        // Matches Borland: TButton::press() message(owner,
                        // evBroadcast, cmRecordHistory, 0), which THistory
                        // answers through its link pointer; here the owner
                        // resolves the link.
                        if event.command == CM_OK || event.command == CM_YES {
                            crate::views::history::record_history_in(self.group());
                        }
                        self.end_modal(event.command);
                        event.clear();
                    }
                    crate::core::command::CM_SHOW_HISTORY
                    | crate::core::command::CM_SHOW_DROPDOWN => {
                        // A History button was clicked, or a ComboBox asked to
                        // drop its list. Leave the event alone so the modal loop
                        // (which has terminal access) can open the popup.
                    }
                    command if self.closes_on(command) => {
                        // A command this dialog closes on (see `CloseOn`): a
                        // custom button, or an explicitly listed command.
                        // Anything else, such as a list box's own command, is
                        // left for the caller.
                        self.end_modal(command);
                        event.clear();
                    }
                    _ => {}
                }
            }
            // If not modal, let commands pass through unchanged
        }
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
            self.window_valid(command)
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        // Dialog uses gray dialog palette (Borland: TDialog::getPalette)
        Some(Palette::from_slice(palettes::CP_GRAY_DIALOG))
    }
});

impl Dialog {
    /// Returns true if the currently focused child view is a button.
    ///
    /// Used by the Enter handling to avoid firing the statically flagged
    /// default button while a different button owns the focus (Borland's
    /// cmGrabDefault semantics: the focused button *is* the current default).
    fn focused_child_is_button(&mut self) -> bool {
        self.group().focused_child().is_some_and(|child| {
            child.is_focused() && child.as_any().downcast_ref::<Button>().is_some()
        })
    }

    /// Find the default button and return its command if it's enabled
    /// Returns None if no default button found or if it's disabled
    /// Matches Borland's TButton::handleEvent() cmDefault broadcast handling (tbutton.cc lines 238-244)
    fn find_default_button_command(&self) -> Option<CommandId> {
        // Borland checks: amDefault && !(state & sfDisabled); a disabled
        // default button yields None rather than falling through to another.
        (0..self.child_count())
            .filter_map(|i| self.child_at(i).as_any().downcast_ref::<Button>())
            .find(|b| b.is_default())
            .and_then(|b| {
                if b.can_focus() {
                    Some(b.command())
                } else {
                    None
                }
            })
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
    close_on: CloseOn,
}

impl DialogBuilder {
    /// Creates a new DialogBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            modal: false,
            resizable: false,
            close_on: CloseOn::StandardAndButtons,
        }
    }

    /// Which commands close the dialog (default: the standard four plus the
    /// dialog's own buttons); see [`CloseOn`].
    #[must_use]
    pub fn close_on(mut self, policy: CloseOn) -> Self {
        self.close_on = policy;
        self
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
        dialog.set_close_on(self.close_on);

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

    #[test]
    fn standard_policy_ignores_custom_button_commands() {
        let mut d = DialogBuilder::new()
            .bounds(Rect::new(0, 0, 30, 8))
            .title("t")
            .close_on(CloseOn::Standard)
            .build();
        d.add(Button::new(Rect::new(1, 1, 10, 3), "Go", 7, false));
        d.set_state(d.state() | SF_MODAL);
        let mut ev = Event::command(7);
        d.handle_event(&mut ev);
        assert_eq!(d.end_state(), 0);
        assert_eq!(ev.what, EventType::Command);
    }

    #[test]
    fn default_policy_closes_on_commands_of_added_buttons_regardless_of_number() {
        let mut d = Dialog::new(Rect::new(0, 0, 30, 8), "t");
        d.add(Button::new(Rect::new(1, 1, 10, 3), "Go", 5000, false));
        d.set_state(d.state() | SF_MODAL);
        let mut ev = Event::command(5000);
        d.handle_event(&mut ev);
        assert_eq!(d.end_state(), 5000);
    }

    #[test]
    fn default_policy_leaves_other_child_commands_to_the_caller() {
        let mut d = Dialog::new(Rect::new(0, 0, 30, 8), "t");
        d.set_state(d.state() | SF_MODAL);
        let mut ev = Event::command(42);
        d.handle_event(&mut ev);
        assert_eq!(d.end_state(), 0, "42 is not a button of this dialog");
        assert_eq!(ev.what, EventType::Command);
    }

    #[test]
    fn default_button_is_found_by_downcast_not_by_view_hook() {
        use crate::core::command::CM_OK;
        use crate::views::static_text::StaticText;
        let mut d = Dialog::new(Rect::new(0, 0, 40, 10), "t");
        d.add(StaticText::new(Rect::new(1, 1, 10, 2), "label"));
        d.add(Button::new(Rect::new(1, 3, 12, 5), "OK", CM_OK, true));
        assert_eq!(d.find_default_button_command(), Some(CM_OK));
    }

    #[test]
    fn dialog_palette_override_reaches_the_frame() {
        use crate::core::palette::palettes;
        let mut plain = Window::new(Rect::new(0, 0, 30, 8), "x"); // Blue palette
        let mut dialog = Dialog::new(Rect::new(0, 0, 30, 8), "x");
        plain.set_focus(true);
        dialog.set_focus(true);
        let mut terminal = crate::test_util::test_terminal(80, 25);
        plain.draw(&mut terminal);
        let blue = terminal.read_cell(1, 0).unwrap().attr;
        dialog.draw(&mut terminal);
        let gray = terminal.read_cell(1, 0).unwrap().attr;
        assert_ne!(blue, gray);
        assert_eq!(
            dialog.get_palette().unwrap().get(1),
            palettes::CP_GRAY_DIALOG[0]
        );
    }
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
        // A command from a child that is not one of this dialog's buttons is
        // left for the caller (Borland: TDialog::handleEvent only ends the
        // modal loop on the standard four commands).
        {
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);

            let mut event = Event::command(1000);
            dialog.handle_event(&mut event);

            assert_eq!(
                dialog.end_state(),
                0,
                "a child's command must not close the dialog"
            );
            assert_eq!(
                event.what,
                EventType::Command,
                "and must stay pending for the caller"
            );
            assert_eq!(event.command, 1000);
        }

        // A custom button added to the dialog closes it with its own command,
        // whatever the number (`CloseOn::StandardAndButtons`, the default).
        {
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            dialog.add(Button::new(Rect::new(1, 1, 10, 3), "Go", 100, false));
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);

            let mut event = Event::command(100);
            dialog.handle_event(&mut event);

            assert_eq!(
                dialog.end_state(),
                100,
                "the button's command closes the dialog"
            );
            assert_eq!(event.what, EventType::Nothing, "and is consumed");
        }

        // An explicit list closes on exactly those commands.
        {
            use crate::core::command::CM_OK;
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            dialog.set_close_on(CloseOn::Commands(vec![999]));
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);

            let mut event = Event::command(999);
            dialog.handle_event(&mut event);
            assert_eq!(dialog.end_state(), 999);
            assert_eq!(event.what, EventType::Nothing);

            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            dialog.set_close_on(CloseOn::Commands(vec![999]));
            let current_state = dialog.state();
            dialog.set_state(current_state | SF_MODAL);
            let mut event = Event::command(CM_OK);
            dialog.handle_event(&mut event);
            assert_eq!(dialog.end_state(), 0, "CM_OK is not in the list");
        }
    }

    #[test]
    fn test_non_modal_dialog_commands() {
        let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
        // Don't set SF_MODAL - this is a non-modal dialog

        // Non-modal dialogs should not call end_modal() for any command
        let mut event = Event::command(100);
        dialog.handle_event(&mut event);

        // end_state should remain 0 because dialog is not modal
        assert_eq!(
            dialog.end_state(),
            0,
            "Non-modal dialog should not set end_state"
        );

        // Commands should pass through unchanged
        let mut event = Event::command(1000);
        dialog.handle_event(&mut event);
        assert_eq!(
            dialog.end_state(),
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
        use crate::views::input_line::InputLine;

        let _guard = crate::core::history::test_lock();
        HistoryManager::clear_all();

        let make_dialog = |text: &str| {
            let mut dialog = Dialog::new(Rect::new(0, 0, 40, 10), "Test");
            let state = dialog.state();
            dialog.set_state(state | SF_MODAL);
            let mut input = InputLine::new(Rect::new(2, 2, 28, 3), 32);
            input.set_text(text);
            let input = dialog.add_typed(input);
            dialog.add(History::new(Point::new(30, 2), 42, input));
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
        assert_eq!(dialog.end_state(), CM_OK);
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

        assert_eq!(dialog.end_state(), 0, "must not close the dialog");
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

        dialog.add(Button::new(
            Rect::new(2, 2, 12, 4),
            "OK",
            CM_OK,
            true, // flagged default
        ));
        dialog.add(Button::new(Rect::new(14, 2, 24, 4), "No", CM_NO, false));
        dialog.set_focus_to_child(1); // focus the non-default button

        let mut event = Event::keyboard(KB_ENTER);
        dialog.handle_event(&mut event);

        assert_eq!(
            dialog.end_state(),
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

        dialog.add(StaticText::new(Rect::new(2, 2, 20, 3), "Hello"));
        dialog.add(Button::new(Rect::new(2, 4, 12, 6), "OK", CM_OK, true));

        let mut event = Event::keyboard(KB_ENTER);
        dialog.handle_event(&mut event);

        assert_eq!(
            dialog.end_state(),
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
