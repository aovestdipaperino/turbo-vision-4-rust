// (C) 2025 - Enzo Lombardi

//! Application structure and event loop implementation.
//! Manages the main application window, menu bar, status line, and desktop.
//! Provides the central event loop and command dispatching system.

use crate::core::command::{CM_CANCEL, CM_COMMAND_SET_CHANGED, CM_QUIT, CommandId};
use crate::core::command_set;
use crate::core::error::Result;
use crate::core::event::{Event, EventType, KB_ALT_X, KB_ESC, KB_ESC_ESC, KB_ESC_X, KB_F10};
use crate::core::geometry::Rect;
use crate::terminal::Terminal;
use crate::views::{View, desktop::Desktop, menu_bar::MenuBar, status_line::StatusLine};
use std::time::Duration;

pub struct Application {
    pub terminal: Terminal,
    pub menu_bar: Option<MenuBar>,
    pub status_line: Option<StatusLine>,
    pub desktop: Desktop,
    pub running: bool,
    needs_redraw: bool, // Track if full redraw is needed
                        // Note: Command set is now stored in thread-local static (command_set module)
                        // This matches Borland's architecture where TView::curCommandSet is static
}

impl Application {
    /// Creates a new application instance and initializes the terminal.
    ///
    /// This function sets up the complete application structure including:
    /// - Terminal initialization in raw mode
    /// - Desktop creation with background
    /// - Global command set initialization
    ///
    /// The menu bar and status line must be set separately using
    /// [`set_menu_bar()`](Self::set_menu_bar) and
    /// [`set_status_line()`](Self::set_status_line).
    ///
    /// # Errors
    ///
    /// Returns an error if terminal initialization fails. See
    /// [`Terminal::init()`](crate::Terminal::init) for details on possible
    /// error conditions.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use turbo_vision::app::Application;
    /// use turbo_vision::core::error::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let mut app = Application::new()?;
    ///     // Set up menu bar, status line, add windows...
    ///     Ok(())
    /// }
    /// ```
    pub fn new() -> Result<Self> {
        let terminal = Terminal::init()?;
        let (width, height) = terminal.size();

        // Create Desktop with full screen bounds initially
        // Will be adjusted when menu_bar/status_line are set
        let desktop = Desktop::new(Rect::new(0, 0, width as i16, height as i16));

        // Initialize global command set
        // Matches Borland's initCommands() (tview.cc:58-68)
        command_set::init_command_set();

        let mut app = Self {
            terminal,
            menu_bar: None,
            status_line: None,
            desktop,
            running: false,
            needs_redraw: true, // Initial draw needed
        };

        // Set initial Desktop bounds (adjusts for missing menu/status)
        // Matches Borland: TProgram::initDeskTop() with no menuBar/statusLine
        app.update_desktop_bounds();

        // Initialize Desktop's palette chain now that it's in its final location
        // This sets up the owner chain so views can resolve colors through Desktop's CP_APP_COLOR palette
        app.desktop.init_palette_chain();

        Ok(app)
    }

    pub fn set_menu_bar(&mut self, menu_bar: MenuBar) {
        self.menu_bar = Some(menu_bar);
        // Update Desktop bounds to exclude menu bar
        // Matches Borland: TProgram::initDeskTop() adjusts r.a.y based on menuBar
        self.update_desktop_bounds();
    }

    pub fn set_status_line(&mut self, status_line: StatusLine) {
        self.status_line = Some(status_line);
        // Update Desktop bounds to exclude status line
        // Matches Borland: TProgram::initDeskTop() adjusts r.b.y based on statusLine
        self.update_desktop_bounds();
    }

    /// Update Desktop bounds to exclude menu bar and status line areas
    /// Matches Borland: TProgram::initDeskTop() calculates bounds based on menuBar/statusLine
    fn update_desktop_bounds(&mut self) {
        let (width, height) = self.terminal.size();
        let mut desktop_bounds = Rect::new(0, 0, width as i16, height as i16);

        // Adjust top edge for menu bar
        // Borland: if (menuBar) r.a.y += menuBar->size.y; else r.a.y++;
        if let Some(ref menu_bar) = self.menu_bar {
            desktop_bounds.a.y += menu_bar.bounds().height();
        } else {
            desktop_bounds.a.y += 1;
        }

        // Adjust bottom edge for status line
        // Borland: if (statusLine) r.b.y -= statusLine->size.y; else r.b.y--;
        if let Some(ref status_line) = self.status_line {
            desktop_bounds.b.y -= status_line.bounds().height();
        } else {
            desktop_bounds.b.y -= 1;
        }

        self.desktop.set_bounds(desktop_bounds);
    }

    /// Request a full redraw on the next frame
    /// Call this after changing the palette or other global settings
    pub fn needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Set a custom application palette and automatically trigger redraw if changed
    /// Pass None to reset to the default Borland palette
    ///
    /// This is a convenience method that combines palette setting with automatic redraw.
    /// It only triggers a redraw if the palette actually changes.
    ///
    /// # Example
    /// ```rust,no_run
    /// use turbo_vision::app::Application;
    ///
    /// let mut app = Application::new()?;
    /// // Set a custom dark theme palette
    /// let dark_palette = vec![/* 63 color bytes */];
    /// app.set_palette(Some(dark_palette));
    /// // Redraw is triggered automatically
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_palette(&mut self, palette: Option<Vec<u8>>) {
        use crate::core::palette::palettes;

        // Get the current palette to check if it's actually changing
        let current_palette = palettes::get_app_palette();
        let is_changing = match &palette {
            Some(new_palette) => new_palette != &current_palette,
            None => {
                // Check if we're currently using a custom palette
                // by comparing with the default (CP_APP_COLOR)
                current_palette != palettes::CP_APP_COLOR
            }
        };

        // Set the new palette
        palettes::set_custom_palette(palette);

        // Trigger redraw only if the palette actually changed
        if is_changing {
            self.needs_redraw = true;
        }
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

    /// Execute a view (modal or modeless)
    /// Matches Borland: TProgram::execView() (tprogram.cc:177-197)
    ///
    /// If the view has SF_MODAL flag set, runs a modal event loop.
    /// Otherwise, adds the view to the desktop and returns immediately.
    ///
    /// Returns the view's end_state (the command that closed the modal view)
    pub fn exec_view(&mut self, view: Box<dyn View>) -> CommandId {
        use crate::core::state::SF_MODAL;

        // Check if view is modal
        let is_modal = (view.state() & SF_MODAL) != 0;

        // Add view to desktop
        self.desktop.add(view);
        let view_index = self.desktop.child_count() - 1;

        if !is_modal {
            // Modeless view - just add to desktop and return
            return 0;
        }

        // Modal view - run event loop
        // Matches Borland: TProgram::execView() runs modal loop (tprogram.cc:184-194)
        loop {
            // Idle processing (broadcasts command changes, etc.)
            self.idle();

            // Update active view bounds
            self.update_active_view_bounds();

            // Draw everything
            self.draw();
            let _ = self.terminal.flush();

            // Poll for event
            if let Ok(Some(mut event)) = self.terminal.poll_event(Duration::from_millis(50)) {
                // Handle event through normal chain
                self.handle_event(&mut event);
            }

            // Check if the modal view wants to close
            // Matches Borland: TGroup::execute() checks endState (tgroup.cc:192)
            if view_index < self.desktop.child_count() {
                let end_state = self.desktop.child_at(view_index).get_end_state();
                if end_state != 0 {
                    // Modal view wants to close
                    // Remove it from desktop and return the end state
                    self.desktop.remove_child(view_index);
                    return end_state;
                }
            } else {
                // View was removed (closed externally)
                return CM_CANCEL;
            }
        }
    }

    pub fn run(&mut self) {
        self.running = true;

        // Initial draw
        self.update_active_view_bounds();
        self.draw();
        let _ = self.terminal.flush();

        while self.running {
            // Handle events first
            let had_event = if let Ok(Some(mut event)) = self.terminal.poll_event(Duration::from_millis(50)) {
                self.handle_event(&mut event);
                true
            } else {
                false
            };

            // Idle processing - broadcast command set changes
            // Matches Borland: TProgram::idle() called during event loop
            self.idle();

            // Remove closed windows (those with SF_CLOSED flag)
            // In Borland, views call CLY_destroy() to remove themselves
            // In Rust, views set SF_CLOSED and parent removes them
            let had_closed_windows = self.desktop.remove_closed_windows();
            if had_closed_windows {
                self.needs_redraw = true; // Window removal requires full redraw
            }

            // Check for moved windows and redraw affected areas (Borland's drawUnderRect pattern)
            // Matches Borland: TView::locate() checks for movement and calls drawUnderRect
            // This optimized redraw only redraws the union of old + new position
            let had_moved_windows = self.desktop.handle_moved_windows(&mut self.terminal);

            // Update active view bounds for F11 dumps
            self.update_active_view_bounds();

            // Optimized drawing strategy (matches Borland's approach):
            // - For moved windows: only redraw union rect (already done in handle_moved_windows)
            // - For content changes: full redraw when events occur
            // - No redraw on idle frames (significant performance improvement)
            //
            // This prevents redrawing every frame (60 FPS) when nothing is happening
            // Borland only redraws when views explicitly request it via draw() or on events
            if self.needs_redraw {
                // Explicit redraw requested (window closed, resize, etc.)
                self.draw();
                self.needs_redraw = false;
                let _ = self.terminal.flush();
            } else if had_moved_windows {
                // Window movement: partial redraw already done via draw_under_rect
                // Just flush the terminal buffer
                let _ = self.terminal.flush();
            } else if had_event {
                // Event occurred: do full redraw for content changes
                // This could be optimized further by tracking which views changed
                self.draw();
                let _ = self.terminal.flush();
            }
            // If no event, no movement, no close: no redraw (idle frame)
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

    pub fn draw(&mut self) {
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

    pub fn handle_event(&mut self, event: &mut Event) {
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
            && (event.key_code == 0x0003 || event.key_code == KB_F10 || event.key_code == KB_ALT_X || event.key_code == KB_ESC || event.key_code == KB_ESC_ESC || event.key_code == KB_ESC_X)
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

    /// Emit a beep sound
    /// Matches Borland: TScreen::makeBeep() - provides audio feedback for errors/alerts
    /// Commonly used in dialog validation failures and error messages
    pub fn beep(&mut self) {
        let _ = self.terminal.beep();
    }

    /// Idle processing - broadcasts command set changes
    /// Matches Borland: TProgram::idle() (tprogram.cc:248-257)
    pub fn idle(&mut self) {
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

    /// Suspend the application (for Ctrl+Z handling)
    /// Matches Borland: TProgram::suspend() - temporarily exits TUI mode
    /// Restores terminal to normal mode, allowing user to return to shell
    /// Call resume() to return to TUI mode
    pub fn suspend(&mut self) -> crate::core::error::Result<()> {
        self.terminal.suspend()
    }

    /// Resume the application after suspension (for Ctrl+Z handling)
    /// Matches Borland: TProgram::resume() - returns to TUI mode and redraws
    /// Re-enters raw mode and forces a complete screen redraw
    pub fn resume(&mut self) -> crate::core::error::Result<()> {
        self.terminal.resume()?;

        // Force complete redraw of the entire UI
        // Draw desktop (which includes all windows)
        self.desktop.draw(&mut self.terminal);

        // Draw menu bar if present
        if let Some(ref mut menu_bar) = self.menu_bar {
            menu_bar.draw(&mut self.terminal);
        }

        // Draw status line if present
        if let Some(ref mut status_line) = self.status_line {
            status_line.draw(&mut self.terminal);
        }

        self.terminal.flush()?;
        Ok(())
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        let _ = self.terminal.shutdown();
    }
}
