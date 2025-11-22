// (C) 2025 - Enzo Lombardi

//! Application structure and event loop implementation.
//! Manages the main application window, menu bar, status line, and desktop.
//! Provides the central event loop and command dispatching system.

use crate::core::command::{CM_CANCEL, CM_CASCADE, CM_COMMAND_SET_CHANGED, CM_QUIT, CM_TILE, CommandId};
use crate::core::command_set;
use crate::core::error::Result;
use crate::core::event::{Event, EventType, KB_ALT_X};
use crate::core::geometry::Rect;
use crate::terminal::Terminal;
use crate::views::{IdleView, View, desktop::Desktop, menu_bar::MenuBar, status_line::StatusLine};
use std::time::Duration;

pub struct Application {
    pub terminal: Terminal,
    pub menu_bar: Option<MenuBar>,
    pub status_line: Option<StatusLine>,
    pub desktop: Desktop,
    pub running: bool,
    needs_redraw: bool, // Track if full redraw is needed
    /// Overlay widgets that need idle processing and are drawn on top of everything
    /// These widgets continue to animate even during modal dialogs
    /// Matches Borland: TProgram::idle() continues running during execView()
    pub(crate) overlay_widgets: Vec<Box<dyn IdleView>>,
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
        let desktop = Desktop::new(Rect::new(0, 0, width, height));

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
            overlay_widgets: Vec::new(),
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

    /// Add an overlay widget that needs idle processing and is drawn on top of everything
    /// These widgets continue to animate even during modal dialogs
    /// Matches Borland: TProgram::idle() continues running during execView()
    ///
    /// # Examples
    /// ```rust,no_run
    /// use turbo_vision::app::Application;
    /// # use turbo_vision::views::IdleView;
    /// # struct AnimatedWidget;
    /// # impl turbo_vision::views::View for AnimatedWidget {
    /// #     fn bounds(&self) -> turbo_vision::core::geometry::Rect { unimplemented!() }
    /// #     fn set_bounds(&mut self, _: turbo_vision::core::geometry::Rect) {}
    /// #     fn draw(&mut self, _: &mut turbo_vision::terminal::Terminal) {}
    /// #     fn handle_event(&mut self, _: &mut turbo_vision::core::event::Event) {}
    /// #     fn update_cursor(&self, _: &mut turbo_vision::terminal::Terminal) {}
    /// #     fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> { None }
    /// # }
    /// # impl IdleView for AnimatedWidget { fn idle(&mut self) {} }
    ///
    /// let mut app = Application::new()?;
    /// let widget = AnimatedWidget { /* ... */ };
    /// app.add_overlay_widget(Box::new(widget));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_overlay_widget(&mut self, widget: Box<dyn IdleView>) {
        self.overlay_widgets.push(widget);
    }

    /// Update Desktop bounds to exclude menu bar and status line areas
    /// Matches Borland: TProgram::initDeskTop() calculates bounds based on menuBar/statusLine
    fn update_desktop_bounds(&mut self) {
        let (width, height) = self.terminal.size();
        // Use actual terminal size, no minimum enforcement at bounds level
        // Clipping will handle terminals smaller than 80x25

        let mut desktop_bounds = Rect::new(0, 0, width, height);

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

    /// Handle terminal resize event
    /// Updates all view bounds (menu bar, desktop, status line) to match new terminal size
    fn handle_resize(&mut self) {
        let (width, height) = self.terminal.size();
        // Use actual terminal size, no minimum enforcement

        // Update menu bar bounds if present
        if let Some(ref mut menu_bar) = self.menu_bar {
            menu_bar.set_bounds(Rect::new(0, 0, width, 1));
        }

        // Update status line bounds if present
        if let Some(ref mut status_line) = self.status_line {
            status_line.set_bounds(Rect::new(0, height - 1, width, height));
        }

        // Update desktop bounds (accounting for menu bar and status line)
        self.update_desktop_bounds();

        // Force full redraw after resize
        self.needs_redraw = true;
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
    /// Matches Borland/Magiblot: TProgram::getEvent() (tprogram.cc:105-174)
    /// This is called by modal views' execute() methods.
    ///
    /// Key behavior (matches magiblot):
    /// - Draws the screen first
    /// - Blocks waiting for events (default 20ms timeout)
    /// - Only calls idle() when there are NO events after timeout
    /// - This gives true event-driven behavior with minimal CPU usage
    pub fn get_event(&mut self) -> Option<Event> {
        // Update active view bounds
        self.update_active_view_bounds();

        // Draw everything (this is the key: drawing happens BEFORE getting events)
        // Matches Borland's CLY_Redraw() in getEvent
        self.draw();
        let _ = self.terminal.flush();

        // Poll for event with 20ms timeout (matches magiblot's eventTimeoutMs)
        // This blocks until an event arrives or timeout occurs
        match self.terminal.poll_event(Duration::from_millis(20)).ok().flatten() {
            Some(event) => {
                // Event received - return it immediately without calling idle()
                // Matches magiblot: idle() is NOT called when events are present
                Some(event)
            }
            None => {
                // Timeout occurred with no events - now we call idle()
                // Matches magiblot: idle() only called when truly idle
                // This is where animations update, command sets broadcast, etc.
                self.idle();
                None
            }
        }
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
        // Matches magiblot: Only calls idle() when no events (true event-driven)
        loop {
            // Update active view bounds
            self.update_active_view_bounds();

            // Draw everything
            self.draw();
            let _ = self.terminal.flush();

            // Poll for event with 20ms timeout (blocks until event or timeout)
            match self.terminal.poll_event(Duration::from_millis(20)).ok().flatten() {
                Some(mut event) => {
                    // Event received - handle it immediately without calling idle()
                    self.handle_event(&mut event);
                }
                None => {
                    // Timeout with no events - call idle() to update animations, etc.
                    self.idle();
                }
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
        // Log initial state to stderr
        let (term_w, term_h) = self.terminal.size();
        let desktop_bounds = self.desktop.bounds();
        eprintln!("=== APPLICATION START ===");
        eprintln!("Terminal buffer size: {}x{}", term_w, term_h);
        eprintln!("Desktop bounds: ({}, {}) to ({}, {}) [size: {}x{}]",
                  desktop_bounds.a.x, desktop_bounds.a.y,
                  desktop_bounds.b.x, desktop_bounds.b.y,
                  desktop_bounds.width(), desktop_bounds.height());
        eprintln!("Menu bar: {:?}", self.menu_bar.as_ref().map(|m| m.bounds()));
        eprintln!("Status line: {:?}", self.status_line.as_ref().map(|s| s.bounds()));
        eprintln!("Origin (0,0) in app coordinates maps to screen coordinate (0,0)");
        eprintln!("Desktop starts at app coordinate ({}, {})\n", desktop_bounds.a.x, desktop_bounds.a.y);

        self.running = true;

        // Initial draw
        self.update_active_view_bounds();
        self.draw();
        let _ = self.terminal.flush();

        while self.running {
            // Optimized drawing strategy (matches Borland's approach):
            // Draw first, then wait for events
            // Only redraw when something changed (not every frame)
            let needs_draw = self.needs_redraw;

            if needs_draw {
                // Explicit redraw requested (window closed, resize, palette change, etc.)
                self.update_active_view_bounds();
                self.draw();
                self.needs_redraw = false;
                let _ = self.terminal.flush();
            }

            // Poll for event with 20ms timeout (matches magiblot's eventTimeoutMs)
            // This blocks until an event arrives or timeout occurs
            match self.terminal.poll_event(Duration::from_millis(20)).ok().flatten() {
                Some(mut event) => {
                    // Event received - handle it immediately without calling idle()
                    // Matches magiblot: idle() is NOT called when events are present
                    self.handle_event(&mut event);

                    // Event occurred: do full redraw for content changes
                    // This could be optimized further by tracking which views changed
                    self.update_active_view_bounds();
                    self.draw();
                    let _ = self.terminal.flush();
                }
                None => {
                    // Timeout with no events - call idle() to update animations, etc.
                    // Matches magiblot: idle() only called when truly idle
                    self.idle();

                    // After idle, draw overlay widgets (animations) if any
                    // Don't redraw everything, just flush overlay widget changes
                    if !self.overlay_widgets.is_empty() {
                        for widget in &mut self.overlay_widgets {
                            widget.draw(&mut self.terminal);
                        }
                        let _ = self.terminal.flush();
                    }
                }
            }

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
            if had_moved_windows {
                // Window movement: partial redraw already done via draw_under_rect
                // Just flush the terminal buffer
                let _ = self.terminal.flush();
            }
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

        // Draw overlay widgets on top of everything
        // These continue to animate even during modal dialogs
        for widget in &mut self.overlay_widgets {
            widget.draw(&mut self.terminal);
        }

        // Update cursor after drawing all views
        // Desktop contains windows/dialogs with focused controls
        self.desktop.update_cursor(&mut self.terminal);
    }

    pub fn handle_event(&mut self, event: &mut Event) {
        // Handle resize events at application level first
        if event.what == EventType::Resize {
            self.handle_resize();
            event.clear();
            return;
        }

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
        if event.what == EventType::Command {
            match event.command {
                CM_QUIT => {
                    self.running = false;
                    event.clear();
                }
                CM_TILE => {
                    self.tile();
                    event.clear();
                }
                CM_CASCADE => {
                    self.cascade();
                    event.clear();
                }
                _ => {}
            }
        }

        // Handle Alt+X (or ESC+X) at application level
        if event.what == EventType::Keyboard && (event.key_code == KB_ALT_X) {
            // Treat these as quit command
            *event = Event::command(CM_QUIT);
            self.running = false;
        }
    }

    // Window Management Methods
    // Matches Borland: TApplication tile/cascade methods (tapplica.cpp:75-127)

    /// Tile all tileable windows in a grid pattern
    /// Matches Borland: TApplication::tile() (tapplica.cpp:123-127)
    pub fn tile(&mut self) {
        let rect = self.get_tile_rect();
        self.desktop.tile_with_rect(rect);
    }

    /// Cascade all tileable windows in a staircase pattern
    /// Matches Borland: TApplication::cascade() (tapplica.cpp:75-79)
    pub fn cascade(&mut self) {
        let rect = self.get_tile_rect();
        self.desktop.cascade_with_rect(rect);
    }

    /// Get the rectangle to use for tiling/cascading operations
    /// Matches Borland: TApplication::getTileRect() (tapplica.cpp:94-97)
    /// Default implementation returns the full desktop extent
    /// Can be overridden to customize the tile area
    pub fn get_tile_rect(&self) -> Rect {
        self.desktop.get_bounds()
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

    /// Set the ESC timeout in milliseconds
    ///
    /// This controls how long the terminal waits after ESC to detect ESC+letter sequences
    /// for macOS Alt key emulation.
    ///
    /// # Arguments
    /// * `timeout_ms` - Timeout in milliseconds, must be between 250 and 1500
    ///
    /// # Errors
    /// Returns an error if the timeout is not between 250 and 1500 milliseconds
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use turbo_vision::app::Application;
    /// # use turbo_vision::core::error::Result;
    /// # fn main() -> Result<()> {
    /// let mut app = Application::new()?;
    /// app.set_esc_timeout(750)?;  // Set to 750ms
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_esc_timeout(&mut self, timeout_ms: u64) -> Result<()> {
        if timeout_ms < 250 || timeout_ms > 1500 {
            return Err(crate::core::error::TurboVisionError::invalid_input(format!(
                "ESC timeout must be between 250 and 1500 milliseconds, got {}",
                timeout_ms
            )));
        }
        self.terminal.set_esc_timeout(timeout_ms);
        Ok(())
    }

    /// Idle processing - broadcasts command set changes and updates command states
    /// Matches Borland: TProgram::idle() (tprogram.cc:248-257)
    pub fn idle(&mut self) {
        // Check for terminal resize (fallback for terminals that don't send resize events)
        if let Ok((actual_width, actual_height)) = Terminal::query_size() {
            let (current_width, current_height) = self.terminal.size();

            if actual_width != current_width || actual_height != current_height {
                // Log resize event
                eprintln!("=== RESIZE EVENT ===");
                eprintln!("Terminal size change: {}x{} -> {}x{}",
                          current_width, current_height,
                          actual_width, actual_height);

                // Terminal was resized but we didn't get an event - handle it now
                // Resize the terminal buffers to actual size
                self.terminal.resize(actual_width.max(0) as u16, actual_height.max(0) as u16);

                // Update all view bounds (with minimum size enforcement)
                self.handle_resize();

                // Log state after resize
                let (term_w, term_h) = self.terminal.size();
                let desktop_bounds = self.desktop.bounds();

                eprintln!("=== AFTER RESIZE ===");
                eprintln!("Terminal buffer size: {}x{}", term_w, term_h);
                eprintln!("Desktop bounds: ({}, {}) to ({}, {}) [size: {}x{}]",
                          desktop_bounds.a.x, desktop_bounds.a.y,
                          desktop_bounds.b.x, desktop_bounds.b.y,
                          desktop_bounds.width(), desktop_bounds.height());
                eprintln!("Menu bar: {:?}", self.menu_bar.as_ref().map(|m| m.bounds()));
                eprintln!("Status line: {:?}", self.status_line.as_ref().map(|s| s.bounds()));

                // Log all desktop children (windows/dialogs)
                let child_count = self.desktop.child_count();
                eprintln!("Desktop children: {}", child_count);
                for i in 0..child_count {
                    let child = self.desktop.child_at(i);
                    let bounds = child.bounds();
                    eprintln!("  Child {}: ({}, {}) to ({}, {}) [size: {}x{}]",
                              i, bounds.a.x, bounds.a.y, bounds.b.x, bounds.b.y,
                              bounds.width(), bounds.height());
                }
                eprintln!();

                // IMPORTANT: Redraw immediately after resize
                // Otherwise the screen won't update until the next event
                self.update_active_view_bounds();
                self.draw();
                let _ = self.terminal.flush();
            }
        }

        // Update overlay widgets (animations, etc.)
        // These continue running even during modal dialogs
        for widget in &mut self.overlay_widgets {
            widget.idle();
        }

        // Update tile/cascade command states based on desktop state
        // Matches Borland: TVDemo::idle() checks deskTop->firstThat(isTileable, 0)
        if self.desktop.has_tileable_windows() {
            command_set::enable_command(CM_TILE);
            command_set::enable_command(CM_CASCADE);
        } else {
            command_set::disable_command(CM_TILE);
            command_set::disable_command(CM_CASCADE);
        }

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
