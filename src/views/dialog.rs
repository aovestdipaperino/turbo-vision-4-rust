// (C) 2025 - Enzo Lombardi

//! Dialog view - modal window for user interaction with OK/Cancel buttons.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_ESC_ESC, KB_ENTER};
use crate::core::command::{CommandId, CM_CANCEL};
use crate::terminal::Terminal;
use super::view::View;
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

    pub fn add(&mut self, view: Box<dyn View>) -> usize {
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
            self.draw(&mut app.terminal);
            self.update_cursor(&mut app.terminal);
            let _ = app.terminal.flush();

            // Poll for event
            if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)).ok().flatten() {
                // Handle the event - this calls Dialog::handle_event()
                // which will call end_modal if needed
                self.handle_event(&mut event);

                // If the event was converted to a command (e.g., KB_ENTER -> CM_OK),
                // we need to process it again so the command handler runs
                // Matches Borland: putEvent() re-queues the converted event
                if event.what == EventType::Command {
                    self.handle_event(&mut event);
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

        // Restore previous state (clear modal flag)
        self.set_state(old_state);

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

    fn update_cursor(&self, terminal: &mut Terminal) {
        self.window.update_cursor(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        use crate::core::state::SF_MODAL;

        // First let the window (and its children) handle the event
        // This is critical: if a focused Memo/Editor handles Enter, it will clear the event
        // Borland's TDialog calls TWindow::handleEvent() FIRST (tdialog.cc line 47)
        self.window.handle_event(event);

        // Now check if the event is still active after children processed it
        // If a child (like Memo/Editor) handled Enter, event.what will be EventType::None
        // This matches Borland's TDialog architecture (tdialog.cc lines 48-86)
        match event.what {
            EventType::Keyboard => {
                match event.key_code {
                    KB_ESC_ESC => {
                        // Double ESC generates cancel command (lines 53-58)
                        *event = Event::command(CM_CANCEL);
                    }
                    KB_ENTER => {
                        // Enter key activates default button (lines 60-66)
                        // Borland converts to evBroadcast + cmDefault and re-queues
                        // We simplify by directly activating the default button
                        if let Some(cmd) = self.find_default_button_command() {
                            *event = Event::command(cmd);
                        } else {
                            event.clear();
                        }
                    }
                    _ => {}
                }
            }
            EventType::Command => {
                // Check for commands that should end modal dialogs
                // Matches Borland: TDialog::handleEvent() (tdialog.cc lines 70-84)
                // In Borland, ANY command that reaches the dialog (not handled by children)
                // will end the modal loop. This allows custom command IDs from buttons.
                if (self.state() & SF_MODAL) != 0 {
                    // End the modal loop with the command ID as the result
                    // Borland: endModal(event.message.command); clearEvent(event);
                    self.window.end_modal(event.command);
                    event.clear();
                }
            }
            _ => {}
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
        // Dialogs can receive focus (delegates to window)
        self.window.can_focus()
    }

    fn set_focus(&mut self, focused: bool) {
        // Delegate focus to the underlying window
        self.window.set_focus(focused);
    }

    fn get_end_state(&self) -> CommandId {
        self.window.get_end_state()
    }

    fn set_end_state(&mut self, command: CommandId) {
        self.window.set_end_state(command);
    }

    /// Validate dialog before closing with given command
    /// Matches Borland: TDialog::valid(ushort command)
    /// - cmCancel always allowed (user can always cancel)
    /// - Other commands validated through window (which validates children)
    fn valid(&mut self, command: CommandId) -> bool {
        if command == CM_CANCEL {
            // Can always cancel
            true
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
