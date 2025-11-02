use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_F10, KB_ALT_X, KB_ESC_X};
use crate::core::command::{CommandId, CM_QUIT, CM_COMMAND_SET_CHANGED, CM_CANCEL};
use crate::core::command_set;
use crate::terminal::Terminal;
use crate::views::{View, menu_bar::MenuBar, status_line::StatusLine, desktop::Desktop};
use std::time::Duration;

pub struct Application {
    pub terminal: Terminal,
    pub menu_bar: Option<MenuBar>,
    pub status_line: Option<StatusLine>,
    pub desktop: Desktop,
    pub running: bool,
    // Note: Command set is now stored in thread-local static (command_set module)
    // This matches Borland's architecture where TView::curCommandSet is static
}

impl Application {
    pub fn new() -> std::io::Result<Self> {
        let terminal = Terminal::init()?;
        let (width, height) = terminal.size();

        let desktop = Desktop::new(Rect::new(0, 1, width as i16, height as i16 - 1));

        // Initialize global command set
        // Matches Borland's initCommands() (tview.cc:58-68)
        command_set::init_command_set();

        Ok(Self {
            terminal,
            menu_bar: None,
            status_line: None,
            desktop,
            running: false,
        })
    }

    pub fn set_menu_bar(&mut self, menu_bar: MenuBar) {
        self.menu_bar = Some(menu_bar);
    }

    pub fn set_status_line(&mut self, status_line: StatusLine) {
        self.status_line = Some(status_line);
    }

    /// Get an event (with drawing)
    /// Matches Borland: TProgram::getEvent() (tprogram.cc:105-174)
    /// This is called by modal views' execute() methods.
    /// It handles idle processing, draws the screen, then polls for an event.
    pub fn get_event(&mut self) -> Option<Event> {
        // Idle processing - broadcast command set changes
        self.idle();

        // Update active view bounds
        self.update_active_view_bounds();

        // Draw everything (this is the key: drawing happens BEFORE getting events)
        // Matches Borland's CLY_Redraw() in getEvent
        self.draw();
        let _ = self.terminal.flush();

        // Poll for event
        self.terminal.poll_event(Duration::from_millis(50)).ok().flatten()
    }

    /// Execute a modal dialog
    /// Matches Borland: TGroup::execView() (tgroup.cc:203-239)
    pub fn exec_dialog(&mut self, mut dialog: crate::views::dialog::Dialog) -> CommandId {
        // Add to desktop as a view
        self.desktop.add(Box::new(dialog));

        // NOTE: We can't call dialog.execute() here because dialog was moved
        // This needs redesign - for now, return to the simpler pattern

        CM_CANCEL
    }

    pub fn run(&mut self) {
        self.running = true;

        // Initial draw
        self.update_active_view_bounds();
        self.draw();
        let _ = self.terminal.flush();

        while self.running {
            // Handle events first
            if let Ok(Some(mut event)) = self.terminal.poll_event(Duration::from_millis(50)) {
                self.handle_event(&mut event);
            }

            // Idle processing - broadcast command set changes
            // Matches Borland: TProgram::idle() called during event loop
            self.idle();

            // Update active view bounds for F11 dumps
            self.update_active_view_bounds();

            // Draw everything AFTER handling events
            // This ensures we display the updated state immediately
            self.draw();
            let _ = self.terminal.flush();
        }
    }

    fn update_active_view_bounds(&mut self) {
        // The active view is the topmost window on the desktop (last child with shadow)
        // Get the focused child from the desktop
        let child_count = self.desktop.child_count();
        if child_count > 0 {
            let last_child = self.desktop.child_at(child_count - 1);
            self.terminal.set_active_view_bounds(last_child.shadow_bounds());
        } else {
            self.terminal.clear_active_view_bounds();
        }
    }

    fn draw(&mut self) {
        // Draw desktop first, then menu bar on top (so dropdown appears over desktop)
        self.desktop.draw(&mut self.terminal);

        if let Some(ref mut menu_bar) = self.menu_bar {
            menu_bar.draw(&mut self.terminal);
        }

        if let Some(ref mut status_line) = self.status_line {
            status_line.draw(&mut self.terminal);
        }

        // Update cursor after drawing all views
        // Desktop contains windows/dialogs with focused controls
        self.desktop.update_cursor(&mut self.terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Menu bar gets first shot
        if let Some(ref mut menu_bar) = self.menu_bar {
            menu_bar.handle_event(event);
            if event.what == EventType::Nothing {
                return;
            }
        }

        // Desktop/windows
        self.desktop.handle_event(event);
        if event.what == EventType::Nothing {
            return;
        }

        // Status line
        if let Some(ref mut status_line) = self.status_line {
            status_line.handle_event(event);
            if event.what == EventType::Nothing {
                return;
            }
        }

        // Application-level command handling
        if event.what == EventType::Command && event.command == CM_QUIT {
            self.running = false;
            event.clear();
        }

        // Handle Ctrl+C, F10, Alt+X, and ESC+X at application level
        if event.what == EventType::Keyboard
            && (event.key_code == 0x0003
                || event.key_code == KB_F10
                || event.key_code == KB_ALT_X
                || event.key_code == KB_ESC_X)
        {
            // Treat these as quit command
            *event = Event::command(CM_QUIT);
            self.running = false;
        }
    }

    // Command Set Management
    // Delegates to global command set functions (command_set module)
    // Matches Borland's TView command set methods (tview.cc:161-389, 672-677)

    /// Check if a command is currently enabled
    /// Matches Borland: TView::commandEnabled(ushort command) (tview.cc:142-147)
    pub fn command_enabled(&self, command: CommandId) -> bool {
        command_set::command_enabled(command)
    }

    /// Enable a single command
    /// Matches Borland: TView::enableCommand(ushort command) (tview.cc:384-389)
    pub fn enable_command(&mut self, command: CommandId) {
        command_set::enable_command(command);
    }

    /// Disable a single command
    /// Matches Borland: TView::disableCommand(ushort command) (tview.cc:161-166)
    pub fn disable_command(&mut self, command: CommandId) {
        command_set::disable_command(command);
    }

    /// Idle processing - broadcasts command set changes
    /// Matches Borland: TProgram::idle() (tprogram.cc:248-257)
    fn idle(&mut self) {
        // Check if command set changed and broadcast to all views
        if command_set::command_set_changed() {
            let mut event = Event::broadcast(CM_COMMAND_SET_CHANGED);

            // Broadcast to desktop (which propagates to all children)
            self.desktop.handle_event(&mut event);

            // Also send to menu bar and status line
            if let Some(ref mut menu_bar) = self.menu_bar {
                menu_bar.handle_event(&mut event);
            }
            if let Some(ref mut status_line) = self.status_line {
                status_line.handle_event(&mut event);
            }

            command_set::clear_command_set_changed();
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        let _ = self.terminal.shutdown();
    }
}
