// (C) 2026 - Enzo Lombardi

//! Application structure and event loop implementation.
//! Manages the main application window, menu bar, status line, and desktop.
//! Provides the central event loop and command dispatching system.

use crate::core::command::{
    CM_CANCEL, CM_CASCADE, CM_COMMAND_SET_CHANGED, CM_HELP_INDEX, CM_QUIT, CM_REDRAW,
    CM_SCREENSHOT, CM_TILE, CommandId,
};
use crate::core::command_set;
use crate::core::error::Result;
use crate::core::event::{Event, EventType, KB_ALT_X, KB_CTRL_F12, KB_F1, KB_F12};
use crate::core::geometry::Rect;
use crate::terminal::Terminal;
use crate::views::help_context::HelpContext;
use crate::views::help_file::HelpFile;
use crate::views::help_window::HelpWindow;
use crate::views::view::ViewId;
use crate::views::window::WindowLike;
use crate::views::{IdleView, View, desktop::Desktop, menu_bar::MenuBar, status_line::StatusLine};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub struct Application {
    pub terminal: Terminal,
    pub menu_bar: Option<MenuBar>,
    pub status_line: Option<StatusLine>,
    pub desktop: Desktop,
    pub running: bool,
    needs_redraw: bool, // Track if full redraw is needed
    /// One-slot pending event queue (Borland: TProgram::putEvent/pending):
    /// returned by the event loops before polling the terminal
    pending_event: Option<Event>,
    /// Overlay widgets that need idle processing and are drawn on top of everything
    /// These widgets continue to animate even during modal dialogs
    /// Matches Borland: TProgram::idle() continues running during execView()
    pub(crate) overlay_widgets: Vec<Box<dyn IdleView>>,
    // Note: Command set is now stored in thread-local static (command_set module)
    // This matches Borland's architecture where TView::curCommandSet is static
    /// Help file for F1 context-sensitive help
    /// Matches Borland: TProgram::helpFile (tprogram.cc)
    help_file: Option<Rc<RefCell<HelpFile>>>,
    /// Help context mappings (context ID to topic ID)
    help_context: HelpContext,
    /// Current help context driving StatusDef switching (Borland: the
    /// focused view's helpCtx; set explicitly in this architecture)
    current_help_ctx: u16,
    /// True between a mouse press and its release. While it is set, `idle`
    /// broadcasts `CM_MOUSE_AUTO_REPEAT` so held-down controls, scrollbar
    /// arrows above all, can keep repeating without any polling of their own.
    mouse_held: bool,
}

/// Application-level hooks for [`Application::run_with`].
///
/// Borland programs subclass `TApplication` and override `handleEvent` and
/// `idle`; this trait is the Rust shape of those overrides, with defaults
/// that do nothing so a handler implements only what it needs. `()` is the
/// handler behind plain [`Application::run`].
pub trait AppHandler {
    /// Called before the menu bar, status line and desktop see the event.
    /// Translate keys into commands or consume application-level commands
    /// here (Borland: `TApplication::handleEvent` before `TProgram` passes
    /// the event to the desktop).
    fn pre_event(&mut self, _app: &mut Application, _event: &mut Event) {}

    /// Called after the desktop has seen the event and only if it is still a
    /// `Command`. Return `true` to mark the event handled.
    fn handle_command(
        &mut self,
        _app: &mut Application,
        _command: CommandId,
        _event: &Event,
    ) -> bool {
        false
    }

    /// Called on each idle tick, after [`Application::idle`].
    fn idle(&mut self, _app: &mut Application) {}

    /// Called once for every window the desktop removed after `SF_CLOSED`.
    fn window_closed(&mut self, _app: &mut Application, _id: ViewId) {}
}

impl AppHandler for () {}

/// What a modal loop's per-tick hook wants to happen next; see
/// [`Application::execute_modal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalTick {
    /// Keep running.
    Continue,
    /// Close the modal view now with this command, bypassing `valid()` the
    /// way an auto-dismiss timeout does.
    End(CommandId),
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
            running: true,
            needs_redraw: true, // Initial draw needed
            pending_event: None,
            current_help_ctx: 0,
            mouse_held: false,
            overlay_widgets: Vec::new(),
            help_file: None,
            help_context: HelpContext::new(),
        };

        // Opt-in remote key injection for testing/automation. Off unless the
        // TV_REMOTE_KEYS environment variable holds a port number.
        if let Ok(port_str) = std::env::var("TV_REMOTE_KEYS") {
            if let Ok(port) = port_str.trim().parse::<u16>() {
                if let Err(e) = app.enable_remote_input(port) {
                    log::warn!("TV_REMOTE_KEYS: failed to listen on port {port}: {e}");
                }
            }
        }

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
    /// # struct AnimatedWidget(turbo_vision::views::ViewCore);
    /// # impl turbo_vision::views::View for AnimatedWidget {
    /// #     fn core(&self) -> &turbo_vision::views::ViewCore { &self.0 }
    /// #     fn core_mut(&mut self) -> &mut turbo_vision::views::ViewCore { &mut self.0 }
    /// #     fn draw(&mut self, _: &mut turbo_vision::terminal::Terminal) {}
    /// #     fn handle_event(&mut self, _: &mut turbo_vision::core::event::Event) {}
    /// #     fn update_cursor(&self, _: &mut turbo_vision::terminal::Terminal) {}
    /// #     fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> { None }
    /// #     fn as_any(&self) -> &dyn std::any::Any { self }
    /// #     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    /// # }
    /// # impl IdleView for AnimatedWidget { fn idle(&mut self) {} }
    ///
    /// let mut app = Application::new()?;
    /// let widget = AnimatedWidget(turbo_vision::views::ViewCore::default());
    /// app.add_overlay_widget(widget);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_overlay_widget<V: IdleView + 'static>(&mut self, widget: V) {
        self.overlay_widgets.push(Box::new(widget));
    }

    /// Update Desktop bounds to exclude menu bar and status line areas
    /// Matches Borland: TProgram::initDeskTop() calculates bounds based on menuBar/statusLine
    fn update_desktop_bounds(&mut self) {
        let (width, height) = self.terminal.size();
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

    /// Request a full redraw on the next frame
    /// Call this after changing the palette or other global settings
    pub fn needs_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Handle a full screen redraw (terminal resize, palette change, etc.).
    ///
    /// Queries the actual terminal size, resizes internal buffers, and
    /// re-lays out the menu bar, status line, and desktop to match.
    pub fn handle_redraw(&mut self) {
        if let Ok((w, h)) = self.terminal.backend_size() {
            let (cur_w, cur_h) = self.terminal.size();
            if w != cur_w || h != cur_h {
                self.terminal.resize(w as u16, h as u16);

                // Re-layout menu bar and status line to the new width
                if let Some(ref mut menu_bar) = self.menu_bar {
                    let mb = menu_bar.bounds();
                    menu_bar.set_bounds(Rect::new(0, mb.a.y, w, mb.b.y));
                }
                if let Some(ref mut status_line) = self.status_line {
                    let sb = status_line.bounds();
                    status_line.set_bounds(Rect::new(0, h - sb.height(), w, h));
                }

                self.update_desktop_bounds();
            }
        }
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

    /// Poll the terminal, treating a dead backend as a quit request.
    ///
    /// A `poll_event` error means the backend connection is gone (e.g. the
    /// SSH client disconnected); swallowing it would leave the event loop
    /// spinning forever on a dead session.
    /// Queue an event to be returned before the next terminal poll.
    ///
    /// Matches Borland TProgram::putEvent: a single-slot pending event that
    /// the event loops consume first (used e.g. to re-enter a command from
    /// event handlers).
    pub fn put_event(&mut self, event: Event) {
        self.pending_event = Some(event);
    }

    fn poll_event_or_quit(&mut self) -> Option<Event> {
        if let Some(pending) = self.pending_event.take() {
            return Some(pending);
        }
        match self.terminal.poll_event(Duration::from_millis(20)) {
            Ok(event) => event,
            Err(err) => {
                log::warn!("terminal backend error, shutting down: {err}");
                self.running = false;
                None
            }
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
        // Draw everything (this is the key: drawing happens BEFORE getting events)
        // Matches Borland's CLY_Redraw() in getEvent
        self.draw();
        let _ = self.terminal.flush();

        // Poll for event with 20ms timeout (matches magiblot's eventTimeoutMs)
        // This blocks until an event arrives or timeout occurs
        match self.poll_event_or_quit() {
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
    pub fn exec_view<V: View + 'static>(&mut self, view: V) -> CommandId {
        use crate::core::state::SF_MODAL;
        let view: Box<dyn View> = Box::new(view);

        // Check if view is modal
        let is_modal = (view.state() & SF_MODAL) != 0;

        // Add view to desktop; track it by identity so other children being
        // added or removed during the modal loop can't shift it out from
        // under us (C++ TGroup::execView uses pointer identity)
        self.desktop.add(view);
        let view_id = self
            .desktop
            .top_view_id()
            .expect("view was just added to the desktop");

        if !is_modal {
            // Modeless view - just add to desktop and return
            return 0;
        }

        // Modal view - run event loop
        // Matches Borland: TProgram::execView() runs modal loop (tprogram.cc:184-194)
        // Matches magiblot: Only calls idle() when no events (true event-driven)
        loop {
            // Draw everything
            self.draw();
            let _ = self.terminal.flush();

            // Poll for event with 20ms timeout (blocks until event or timeout)
            match self.poll_event_or_quit() {
                Some(mut event) => {
                    // Event received - handle it immediately without calling idle()
                    self.handle_event(&mut event);
                }
                None => {
                    // Timeout with no events - call idle() to update animations, etc.
                    self.idle();
                }
            }

            // Check if application wants to quit (Alt+X, CM_QUIT)
            // This allows quit to work even when modal dialogs are open
            if !self.running {
                // Matches Borland: TProgram::handleEvent endModal(cmQuit) —
                // callers can distinguish app shutdown from a plain cancel
                self.desktop.remove_child_by_id(view_id);
                return CM_QUIT;
            }

            // Check if the modal view wants to close
            // Matches Borland: TGroup::execute() checks endState (tgroup.cc:192)
            // followed by the valid(endState) re-entry check
            if self.desktop.contains_id(view_id) {
                // Borland reads TGroup::endState; here the view says whether it
                // is a group at all (dynamic_cast<TGroup*>).
                let end_state = self
                    .desktop
                    .child_by_id(view_id)
                    .and_then(|child| child.as_group())
                    .map_or(0, |group| group.end_state());
                if end_state != 0 {
                    // A failing validator vetoes the close (Borland:
                    // do { ... } while( !valid(endState) ))
                    let close_ok = self
                        .desktop
                        .child_by_id_mut(view_id)
                        .map(|child| child.valid(end_state))
                        .unwrap_or(true);
                    if close_ok {
                        self.desktop.remove_child_by_id(view_id);
                        return end_state;
                    }
                    if let Some(group) = self
                        .desktop
                        .child_by_id_mut(view_id)
                        .and_then(|child| child.as_group_mut())
                    {
                        group.end_modal(0);
                    }
                }
            } else {
                // View was removed (closed externally)
                return CM_CANCEL;
            }
        }
    }

    /// Run `view` modally: the single modal loop behind `Dialog::execute`,
    /// `FileDialog::execute` and `HelpWindow::execute` (Borland:
    /// `TGroup::execute`, with `TProgram::getEvent` drawing the screen).
    ///
    /// Each iteration draws the desktop, menu bar, status line, `view` and
    /// the overlay widgets, polls one event and dispatches it to `view` (twice
    /// if the first pass turned it into a command, matching `putEvent`),
    /// opens the history and drop-down popups a child asked for, then calls
    /// `tick`. Returns the command the view ended with once `valid` accepts
    /// it, or the command `tick` returned.
    ///
    /// Because the event goes to `view`'s own `handle_event`, an override on
    /// the outer type (a `FileDialog` around a `Dialog`) is what runs, which is
    /// how such a type reacts to its children without owning the loop.
    pub fn execute_modal<V, F>(&mut self, view: &mut V, mut tick: F) -> CommandId
    where
        V: WindowLike + ?Sized,
        F: FnMut(&mut Application, &mut V) -> ModalTick,
    {
        loop {
            // Draw desktop first (clears the background), then the modal view
            // on top: a view that is not on the desktop must draw itself here.
            self.desktop.draw(&mut self.terminal);
            if let Some(ref mut menu_bar) = self.menu_bar {
                menu_bar.draw(&mut self.terminal);
            }
            if let Some(ref mut status_line) = self.status_line {
                status_line.draw(&mut self.terminal);
            }
            view.draw(&mut self.terminal);
            // Overlay widgets keep animating during modal loops
            // (Borland: TProgram::idle() continues running during execView()).
            for widget in &mut self.overlay_widgets {
                widget.draw(&mut self.terminal);
            }
            view.update_cursor(&mut self.terminal);
            let _ = self.terminal.flush();

            // 20ms timeout matches magiblot's eventTimeoutMs; idle() only runs
            // when there truly was no event.
            match self.poll_event_or_quit() {
                Some(mut event) => {
                    if event.what == EventType::Broadcast && event.command == CM_REDRAW {
                        self.handle_redraw();
                        continue;
                    }

                    view.handle_event(&mut event);

                    // A keyboard event turned into a command (Enter -> CM_OK)
                    // is re-dispatched so the command handler runs (putEvent).
                    if event.what == EventType::Command {
                        view.handle_event(&mut event);
                    }

                    // A History button converted its click into CM_SHOW_HISTORY;
                    // the popup needs the terminal, so it opens here.
                    if event.what == EventType::Command
                        && event.command == crate::core::command::CM_SHOW_HISTORY
                    {
                        self.show_history_popup_for(view, &mut event);
                    }
                    // Same for a ComboBox asking to drop its list down.
                    if event.what == EventType::Command
                        && event.command == crate::core::command::CM_SHOW_DROPDOWN
                    {
                        crate::views::dialog::show_dropdown_popup(&mut event, &mut self.terminal);
                    }
                }
                None => self.idle(),
            }

            if let ModalTick::End(command) = tick(self, view) {
                return command;
            }

            // Borland: do { ... } while( !valid(endState) ) — a failing
            // validator vetoes the close and re-enters the loop.
            let end_state = view.end_state();
            if end_state != 0 {
                if view.valid(end_state) {
                    return end_state;
                }
                view.end_modal(0);
            }
        }
    }

    /// Open the history popup a `CM_SHOW_HISTORY` command asked for and copy
    /// the selection into the input linked to the button that asked.
    fn show_history_popup_for<V: WindowLike + ?Sized>(&mut self, view: &mut V, event: &mut Event) {
        use crate::core::geometry::Point;
        use crate::core::history::HistoryManager;
        use crate::views::history_window::HistoryWindow;

        let history_id = event.info;
        // Just below the clicked button, shifted left so the dropdown covers
        // the input line it belongs to.
        let pos = Point::new((event.mouse.pos.x - 20).max(0), event.mouse.pos.y + 1);
        let mut window = HistoryWindow::new(pos, history_id, 30);
        if let Some(selected) = window.execute(&mut self.terminal) {
            // Move the selection to the front of the list, then fill the input
            // the History button is linked to (the owner resolves the link).
            HistoryManager::add(history_id, selected.clone());
            crate::views::history::apply_history_selection(view.group_mut(), history_id, &selected);
        }
        event.clear();
    }

    /// Run the event loop with no application hooks; see [`run_with`](Self::run_with).
    pub fn run(&mut self) {
        self.run_with(&mut ());
    }

    /// Run the event loop, giving `handler` a chance before and after each
    /// event, on idle, and when a window closes.
    pub fn run_with<H: AppHandler>(&mut self, handler: &mut H) {
        self.running = true;

        // Initial draw
        self.draw();
        let _ = self.terminal.flush();

        while self.running {
            // Optimized drawing strategy (matches Borland's approach):
            // Draw first, then wait for events
            // Only redraw when something changed (not every frame)
            let needs_draw = self.needs_redraw;

            if needs_draw {
                // Explicit redraw requested (window closed, resize, palette change, etc.)
                self.draw();
                self.needs_redraw = false;
                let _ = self.terminal.flush();
            }

            // Poll for event with 20ms timeout (matches magiblot's eventTimeoutMs)
            // This blocks until an event arrives or timeout occurs
            match self.poll_event_or_quit() {
                Some(mut event) => {
                    // Event received - handle it immediately without calling idle()
                    // Matches magiblot: idle() is NOT called when events are present
                    handler.pre_event(self, &mut event);
                    self.handle_event(&mut event);
                    if event.what == EventType::Command
                        && handler.handle_command(self, event.command, &event)
                    {
                        event.clear();
                    }

                    // Event occurred: do full redraw for content changes
                    // This could be optimized further by tracking which views changed
                    self.draw();
                    let _ = self.terminal.flush();
                }
                None => {
                    // Timeout with no events - call idle() to update animations, etc.
                    // Matches magiblot: idle() only called when truly idle
                    self.idle();
                    handler.idle(self);

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
            let closed = self.desktop.remove_closed_windows();
            for id in &closed {
                handler.window_closed(self, *id);
            }
            if !closed.is_empty() {
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
        // Handle CM_REDRAW before anything else — resize the terminal buffers
        // and re-layout all top-level views so subsequent drawing is correct.
        if event.what == EventType::Broadcast && event.command == CM_REDRAW {
            self.handle_redraw();
            event.clear();
            return;
        }

        // Pre-dispatch global shortcuts — these must be handled before any
        // view sees the event, because focused views (e.g. the editor) would
        // otherwise consume the key code.
        if event.what == EventType::Keyboard {
            match event.key_code {
                KB_F1 => {
                    self.show_help();
                    event.clear();
                    return;
                }
                KB_ALT_X => {
                    *event = Event::command(CM_QUIT);
                    self.running = false;
                    return;
                }
                KB_F12 => {
                    self.dump_screen_ansi();
                    event.clear();
                    return;
                }
                KB_CTRL_F12 => {
                    self.take_screenshot();
                    event.clear();
                    return;
                }
                code @ crate::core::event::KB_ALT_1..=crate::core::event::KB_ALT_9 => {
                    // Alt+digit selects the numbered window
                    // (Borland: TProgram::handleEvent -> cmSelectWindowNum)
                    let num = ((code - crate::core::event::KB_ALT_1) >> 8) as u16 + 1;
                    *event =
                        Event::broadcast_with_info(crate::core::command::CM_SELECT_WINDOW_NUM, num);
                }
                _ => {}
            }
        }

        // Track the button so idle knows whether to drive auto-repeat.
        match event.what {
            EventType::MouseDown => self.mouse_held = true,
            EventType::MouseUp => self.mouse_held = false,
            _ => {}
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
                CM_HELP_INDEX => {
                    self.show_help();
                    event.clear();
                }
                crate::core::command::CM_TOGGLE_BLOCK_MODE => {
                    self.toggle_block_edit_mode();
                    event.clear();
                }
                CM_SCREENSHOT => {
                    self.take_screenshot();
                    event.clear();
                }
                crate::core::command::CM_SHOW_HISTORY => {
                    // A History button was clicked in a window running under
                    // exec_view()/run(); open the popup here where we have
                    // terminal access, then fill the linked input.
                    use crate::core::geometry::Point;
                    use crate::core::history::HistoryManager;

                    let history_id = event.info;
                    let pos = Point::new((event.mouse.pos.x - 20).max(0), event.mouse.pos.y + 1);
                    let mut window =
                        crate::views::history_window::HistoryWindow::new(pos, history_id, 30);
                    if let Some(selected) = window.execute(&mut self.terminal) {
                        HistoryManager::add(history_id, selected.clone());
                        // The button lives in one of the desktop's windows;
                        // the search recurses into them.
                        crate::views::history::apply_history_selection(
                            self.desktop.children_mut(),
                            history_id,
                            &selected,
                        );
                    }
                    event.clear();
                }
                crate::core::command::CM_SHOW_DROPDOWN => {
                    // A ComboBox on a plain window asked to drop its list down;
                    // open it here, where the terminal is reachable.
                    crate::views::dialog::show_dropdown_popup(event, &mut self.terminal);
                }
                _ => {}
            }
        }
    }

    /// Enable the remote keyboard-input listener on the given TCP port.
    ///
    /// This is **off by default**. It is a thin wrapper around
    /// [`Terminal::enable_remote_input`](crate::terminal::Terminal::enable_remote_input):
    /// once enabled, key chords sent to `127.0.0.1:port` (e.g. `"CTRL+F12"`) are
    /// injected into the event loop as real key presses. Useful for automated
    /// testing of global shortcuts such as the Ctrl+F12 screenshot.
    ///
    /// It can also be enabled without code changes by setting the
    /// `TV_REMOTE_KEYS` environment variable to the desired port.
    ///
    /// # Errors
    ///
    /// Returns an error if the port cannot be bound.
    pub fn enable_remote_input(&mut self, port: u16) -> Result<()> {
        self.terminal.enable_remote_input(port)?;
        Ok(())
    }

    /// Save a PNG screenshot of the current screen (bound to Ctrl+F12).
    ///
    /// The file is written to the current working directory with a
    /// timestamped name like `screenshot-20260607-194800.png`. Rendering
    /// queries the current font cell size; see
    /// [`Terminal::save_screenshot_png`](crate::terminal::Terminal::save_screenshot_png).
    pub fn take_screenshot(&mut self) {
        let filename = format!(
            "screenshot-{}.png",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        );
        match self.terminal.save_screenshot_png(&filename) {
            Ok(()) => log::info!("Screenshot saved to {filename}"),
            Err(e) => log::warn!("Failed to save screenshot: {e}"),
        }
    }

    /// Save an ASCII (ANSI-colored) dump of the whole screen (bound to F12).
    ///
    /// The file is written to the current working directory with a timestamped
    /// name like `screen-20260607-194800.ans` and can be viewed with `cat` or
    /// `less -R`. See [`Terminal::dump_screen`](crate::terminal::Terminal::dump_screen).
    pub fn dump_screen_ansi(&mut self) {
        let filename = format!(
            "screen-{}.ans",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        );
        match self.terminal.dump_screen(&filename) {
            Ok(()) => log::info!("Screen dump saved to {filename}"),
            Err(e) => log::warn!("Failed to save screen dump: {e}"),
        }
    }

    // Help System Methods
    // Matches Borland: TProgram help support (tprogram.cc)

    /// Set the help file for F1 context-sensitive help
    /// Matches Borland: TApplication::helpFile initialization
    ///
    /// # Arguments
    /// * `path` - Path to a markdown help file
    ///
    /// # Returns
    /// Result indicating success or file load error
    ///
    /// # Examples
    /// ```ignore
    /// app.set_help_file("help/manual.md")?;
    /// ```
    pub fn set_help_file(&mut self, path: &str) -> std::io::Result<()> {
        let help_file = HelpFile::new(path)?;
        self.help_file = Some(Rc::new(RefCell::new(help_file)));
        Ok(())
    }

    /// Set a pre-built help file for F1 context-sensitive help
    pub fn set_help(&mut self, help_file: HelpFile) {
        self.help_file = Some(Rc::new(RefCell::new(help_file)));
    }

    /// Register a help context mapping (context ID to topic ID)
    /// This allows views to have help_context set, and F1 will open the corresponding topic
    ///
    /// # Arguments
    /// * `context_id` - Numeric context ID (assigned to views)
    /// * `topic_id` - String topic ID in the help file (e.g., "file-open")
    pub fn register_help_context(&mut self, context_id: u16, topic_id: &str) {
        self.help_context.register(context_id, topic_id);
    }

    /// Show help for a specific topic
    /// Opens the help window and displays the given topic
    pub fn show_help_topic(&mut self, topic_id: &str) {
        use crate::core::state::SF_MODAL;

        if let Some(ref help_file) = self.help_file {
            let (width, height) = self.terminal.size();
            let help_width = (width * 3 / 4).max(40).min(width - 4);
            let help_height = (height * 3 / 4).max(10).min(height - 4);
            let x = (width - help_width) / 2;
            let y = (height - help_height) / 2;

            let bounds = Rect::new(x, y, x + help_width, y + help_height);
            let mut help_window = HelpWindow::new(bounds, "Help", Rc::clone(help_file));
            help_window.show_topic(topic_id);

            // Set SF_MODAL flag so exec_view runs the modal loop
            // Matches Borland: THelpWindow is displayed modally
            let current_state = help_window.state();
            help_window.set_state(current_state | SF_MODAL);

            // Execute the help window as modal
            self.exec_view(help_window);
        }
    }

    /// Show context-sensitive help
    /// Looks up the focused view's help context and opens the appropriate topic
    /// Matches Borland: TProgram::getEvent() F1 handling
    pub fn show_help(&mut self) {
        // For now, show default topic. In future, this would:
        // 1. Get the focused view's help context
        // 2. Look up the topic ID from help_context
        // 3. Show that topic
        //
        // Since views don't have help_context field yet, we show the default topic
        let topic_id = if let Some(ref help_file) = self.help_file {
            help_file.borrow().get_default_topic().map(|t| t.id.clone())
        } else {
            None
        };

        if let Some(topic_id) = topic_id {
            self.show_help_topic(&topic_id);
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

    // Block-edit mode
    // Global flag (core::state) rather than a keyboard modifier: terminals
    // disagree on whether they deliver Alt/Option with cursor keys and drags.

    /// Is block-edit mode on? Editors start rectangular selections while it is.
    pub fn block_edit_mode(&self) -> bool {
        crate::core::state::block_edit_mode()
    }

    /// Turn block-edit mode on or off.
    pub fn set_block_edit_mode(&mut self, on: bool) {
        crate::core::state::set_block_edit_mode(on);
    }

    /// Flip block-edit mode and return the new value.
    pub fn toggle_block_edit_mode(&mut self) -> bool {
        crate::core::state::toggle_block_edit_mode()
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
            return Err(crate::core::error::TurboVisionError::invalid_input(
                format!(
                    "ESC timeout must be between 250 and 1500 milliseconds, got {}",
                    timeout_ms
                ),
            ));
        }
        self.terminal.set_esc_timeout(timeout_ms);
        Ok(())
    }

    /// Idle processing - broadcasts command set changes and updates command states
    /// Matches Borland: TProgram::idle() (tprogram.cc:248-257)
    /// Set the current help context.
    ///
    /// Matches Borland: TProgram tracks the focused view's helpCtx; since
    /// views here don't carry one, applications set it when the active
    /// window changes. The status line's TStatusDef item set follows it on
    /// the next idle tick (see StatusLine::with_defs).
    pub fn set_help_context(&mut self, help_ctx: u16) {
        self.current_help_ctx = help_ctx;
    }

    pub fn idle(&mut self) {
        // Safety net for a resize that never reached us as an event: a
        // SIGWINCH raised before crossterm's event source exists (Warp does
        // this when the alternate screen changes the pty size at startup) is
        // never delivered, leaving the app laid out for the wrong size. idle()
        // only runs when nothing else is happening, so re-checking the size
        // here is cheap and self-heals that case.
        if let Ok((w, h)) = self.terminal.backend_size() {
            let (cur_w, cur_h) = self.terminal.size();
            if w != cur_w || h != cur_h {
                self.handle_redraw();
            }
        }

        // Status line follows the current help context (Borland:
        // TProgram::idle calls statusLine->update())
        if let Some(ref mut status_line) = self.status_line {
            status_line.update(self.current_help_ctx);
        }

        // Update overlay widgets (animations, etc.)
        // These continue running even during modal dialogs
        for widget in &mut self.overlay_widgets {
            widget.idle();
        }

        // While a button is held, let views repeat their press action. Nothing
        // is sent when no button is down, so an idle app stays idle.
        if self.mouse_held {
            let mut repeat = Event::broadcast(crate::core::command::CM_MOUSE_AUTO_REPEAT);
            self.desktop.handle_event(&mut repeat);
        }

        // A plain tick for views that need a timer of their own: a tooltip's
        // hover delay, an animation. `idle` only runs when the event poll times
        // out, so this fires a few times a second while the user is not typing,
        // and not at all while they are. Views must not clear it, since a
        // broadcast stops travelling once it is consumed.
        let mut tick = Event::broadcast(crate::core::command::CM_IDLE_TICK);
        self.desktop.handle_event(&mut tick);

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

#[cfg(test)]
mod resize_tests {
    use super::*;

    #[test]
    fn execute_modal_stops_when_the_tick_says_so_and_dispatches_events_to_the_view() {
        use crate::core::command::{CM_CANCEL, CM_OK};
        use crate::core::state::SF_MODAL;
        use crate::views::button::Button;
        use crate::views::dialog::Dialog;

        let (mut app, _size, _calls) = build_test_app(80, 25);
        let mut dialog = Dialog::new(Rect::new(5, 5, 40, 12), "t");
        dialog.add(Button::new(Rect::new(2, 2, 12, 4), "OK", CM_OK, true));
        let mut ticks = 0;
        let result = app.execute_modal(&mut dialog, |_app, _d| {
            ticks += 1;
            if ticks == 3 {
                ModalTick::End(CM_CANCEL)
            } else {
                ModalTick::Continue
            }
        });
        assert_eq!(result, CM_CANCEL);
        assert_eq!(ticks, 3);

        let mut dialog = Dialog::new(Rect::new(5, 5, 40, 12), "t");
        dialog.add(Button::new(Rect::new(2, 2, 12, 4), "OK", CM_OK, true));
        dialog.set_state(dialog.state() | SF_MODAL);
        app.put_event(Event::command(CM_OK));
        let result = app.execute_modal(&mut dialog, |_, _| ModalTick::Continue);
        assert_eq!(result, CM_OK);
    }

    #[test]
    fn run_with_delivers_unhandled_commands_to_the_handler() {
        struct Recorder {
            seen: Vec<CommandId>,
        }
        impl AppHandler for Recorder {
            fn handle_command(
                &mut self,
                app: &mut Application,
                command: CommandId,
                _e: &Event,
            ) -> bool {
                self.seen.push(command);
                if command == 1234 {
                    app.running = false;
                }
                true
            }
        }
        let (mut app, _size, _calls) = build_test_app(80, 25);
        app.put_event(Event::command(1234));
        let mut rec = Recorder { seen: vec![] };
        app.run_with(&mut rec);
        assert_eq!(rec.seen, vec![1234]);
    }
    use crate::core::state::{GF_GROW_HI_X, GF_GROW_HI_Y};
    use crate::test_util::TestBackend;
    use crate::views::group::GroupLike;
    use crate::views::view::ViewCore;
    use std::cell::Cell as StdCell;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU16, Ordering};

    /// A child view that records every `set_bounds` call, standing in for a
    /// real consumer (e.g. the text scrollback re-wrap in `TextViewer`) that
    /// depends on `set_bounds` actually being called on resize.
    struct RecordingView {
        core: ViewCore,
        set_bounds_calls: Rc<StdCell<u32>>,
    }

    impl View for RecordingView {
        fn core(&self) -> &ViewCore {
            &self.core
        }

        fn core_mut(&mut self) -> &mut ViewCore {
            &mut self.core
        }

        fn set_bounds(&mut self, bounds: Rect) {
            self.core.bounds = bounds;
            self.set_bounds_calls.set(self.set_bounds_calls.get() + 1);
        }
        fn draw(&mut self, _terminal: &mut Terminal) {}
        fn handle_event(&mut self, _event: &mut Event) {}
        fn get_palette(&self) -> Option<crate::core::palette::Palette> {
            None
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    /// Wraps a plain `Group` so it can itself be added as a Desktop child;
    /// it forwards `set_bounds` down to its own children (the
    /// `RecordingView`) via `Group`'s existing grow-mode logic, so the
    /// resize cascade crosses two levels of nesting — like a window's frame
    /// containing a scrollable widget.
    struct GroupView(crate::views::group::Group);

    impl View for GroupView {
        fn core(&self) -> &ViewCore {
            self.0.core()
        }

        fn core_mut(&mut self) -> &mut ViewCore {
            self.0.core_mut()
        }

        fn set_bounds(&mut self, bounds: Rect) {
            View::set_bounds(&mut self.0, bounds);
        }

        fn draw(&mut self, terminal: &mut Terminal) {
            View::draw(&mut self.0, terminal);
        }
        fn handle_event(&mut self, event: &mut Event) {
            View::handle_event(&mut self.0, event);
        }
        fn get_palette(&self) -> Option<crate::core::palette::Palette> {
            None
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    /// Builds an Application on a `ResizableBackend`, with a menu bar,
    /// status line, and one nested `RecordingView`.
    fn build_test_app(
        width: i16,
        height: i16,
    ) -> (Application, Arc<(AtomicU16, AtomicU16)>, Rc<StdCell<u32>>) {
        let backend = TestBackend::new(
            u16::try_from(width).unwrap(),
            u16::try_from(height).unwrap(),
        );
        let size = backend.size_handle();
        let terminal = Terminal::with_backend(Box::new(backend)).unwrap();

        let desktop = Desktop::new(Rect::new(0, 0, width, height));

        let mut app = Application {
            terminal,
            menu_bar: None,
            status_line: None,
            desktop,
            running: true,
            needs_redraw: true,
            pending_event: None,
            overlay_widgets: Vec::new(),
            mouse_held: false,
            help_file: None,
            help_context: HelpContext::new(),
            current_help_ctx: 0,
        };

        app.set_menu_bar(MenuBar::new(Rect::new(0, 0, width, 1)));
        app.set_status_line(StatusLine::new(
            Rect::new(0, height - 1, width, height),
            Vec::new(),
        ));

        let set_bounds_calls = Rc::new(StdCell::new(0));
        let mut inner_group = crate::views::group::Group::new(Rect::new(0, 0, width, height - 2));
        inner_group.set_grow_mode(GF_GROW_HI_X | GF_GROW_HI_Y);
        inner_group.add(RecordingView {
            core: ViewCore {
                bounds: Rect::new(0, 0, width, height - 2),
                grow_mode: GF_GROW_HI_X | GF_GROW_HI_Y,
                ..ViewCore::default()
            },
            set_bounds_calls: Rc::clone(&set_bounds_calls),
        });

        let mut group_view = GroupView(inner_group);
        group_view.set_grow_mode(GF_GROW_HI_X | GF_GROW_HI_Y);
        app.desktop.add(group_view);

        (app, size, set_bounds_calls)
    }

    #[test]
    fn resize_resizes_desktop_to_new_bounds() {
        let (mut app, size, _calls) = build_test_app(80, 25);
        size.0.store(100, Ordering::SeqCst);
        size.1.store(40, Ordering::SeqCst);

        app.handle_redraw();

        let desktop_bounds = app.desktop.get_bounds();
        assert_eq!(desktop_bounds.width(), 100);
        // Desktop height excludes the 1-row menu bar and 1-row status line.
        assert_eq!(desktop_bounds.height(), 40 - 2);
        assert_eq!(app.terminal.size(), (100, 40));
    }

    #[test]
    fn resize_lays_out_grow_mode_children_and_reaches_nested_view() {
        let (mut app, size, calls) = build_test_app(80, 25);
        let before = calls.get();
        size.0.store(120, Ordering::SeqCst);
        size.1.store(50, Ordering::SeqCst);

        app.handle_redraw();

        // set_bounds must actually reach the nested RecordingView, not just
        // the top-level Group.
        assert!(calls.get() > before);
        let child_bounds = app.desktop.child_at(app.desktop.child_count() - 1).bounds();
        assert_eq!(child_bounds.width(), 120);
        assert_eq!(child_bounds.height(), 50 - 2);
    }

    #[test]
    fn resize_moves_status_line_to_new_bottom_row() {
        let (mut app, size, _calls) = build_test_app(80, 25);
        size.0.store(80, Ordering::SeqCst);
        size.1.store(50, Ordering::SeqCst);

        app.handle_redraw();

        let sb = app.status_line.as_ref().unwrap().bounds();
        assert_eq!(sb.a.y, 49);
        assert_eq!(sb.b.y, 50);
    }

    #[test]
    fn resize_moves_menu_bar_to_new_width() {
        let (mut app, size, _calls) = build_test_app(80, 25);
        size.0.store(120, Ordering::SeqCst);
        size.1.store(25, Ordering::SeqCst);

        app.handle_redraw();

        let mb = app.menu_bar.as_ref().unwrap().bounds();
        assert_eq!(mb.b.x, 120);
    }

    #[test]
    fn shrink_then_grow_returns_sane_geometry() {
        let (mut app, size, _calls) = build_test_app(80, 25);

        size.0.store(40, Ordering::SeqCst);
        size.1.store(12, Ordering::SeqCst);
        app.handle_redraw();
        let shrunk = app.desktop.get_bounds();
        assert_eq!(shrunk.width(), 40);
        assert_eq!(shrunk.height(), 12 - 2);
        assert!(shrunk.a.x <= shrunk.b.x);
        assert!(shrunk.a.y <= shrunk.b.y);

        size.0.store(80, Ordering::SeqCst);
        size.1.store(25, Ordering::SeqCst);
        app.handle_redraw();
        let restored = app.desktop.get_bounds();
        assert_eq!(restored.width(), 80);
        assert_eq!(restored.height(), 25 - 2);
        assert_eq!(app.terminal.size(), (80, 25));

        let child_bounds = app.desktop.child_at(app.desktop.child_count() - 1).bounds();
        assert_eq!(child_bounds.width(), 80);
        assert_eq!(child_bounds.height(), 25 - 2);
    }

    #[test]
    fn no_size_change_does_not_disturb_layout() {
        let (mut app, _size, calls) = build_test_app(80, 25);
        let before_bounds = app.desktop.get_bounds();
        let before_calls = calls.get();

        app.handle_redraw();

        assert_eq!(app.desktop.get_bounds(), before_bounds);
        assert_eq!(calls.get(), before_calls);
    }

    // --- Window participation in the resize cascade -----------------------
    //
    // Regression coverage for the bug where every `Window` reported
    // `grow_mode() == 0` (fixed) because `Window` didn't override the
    // `View` trait's default grow-mode accessors, so the desktop resize
    // cascade correctly visited windows and then did nothing to them.

    use crate::core::geometry::Point;
    use crate::views::window::Window;

    #[test]
    fn window_added_to_desktop_follows_desktop_resize() {
        let (mut app, size, _calls) = build_test_app(80, 25);
        // Establish the real (post-menu/status-line) desktop bounds before
        // sizing the window, and leave room for the window's shadow so
        // `Desktop::add`'s `constrain_to_parent_bounds` doesn't shift it.
        app.handle_redraw();
        let (shadow_x, shadow_y) = crate::core::state::shadow_size();
        let desktop_bounds = app.desktop.get_bounds();
        let window_bounds = Rect::new(
            desktop_bounds.a.x,
            desktop_bounds.a.y,
            desktop_bounds.b.x - shadow_x,
            desktop_bounds.b.y - shadow_y,
        );

        // Window::add (via the interior Group) takes bounds relative to the
        // window's interior origin, not absolute screen coordinates. Span
        // the whole interior (0,0)..(interior width, interior height), like
        // a content view (e.g. a TextViewer) that fills its window.
        let interior_w = window_bounds.width() - 2;
        let interior_h = window_bounds.height() - 2;

        let mut window = Window::new(window_bounds, "Test Window");
        let set_bounds_calls = Rc::new(StdCell::new(0));
        window.add(RecordingView {
            core: ViewCore {
                bounds: Rect::new(0, 0, interior_w, interior_h),
                grow_mode: GF_GROW_HI_X | GF_GROW_HI_Y,
                ..ViewCore::default()
            },
            set_bounds_calls: Rc::clone(&set_bounds_calls),
        });
        app.desktop.add(window);

        size.0.store(120, Ordering::SeqCst);
        size.1.store(50, Ordering::SeqCst);
        app.handle_redraw();

        let window_index = app.desktop.child_count() - 1;
        let new_desktop_bounds = app.desktop.get_bounds();
        let window = app
            .desktop
            .child_at_mut(window_index)
            .as_any_mut()
            .downcast_mut::<Window>()
            .expect("last desktop child should be the Window");

        // The window's own bounds (and therefore its frame, which shares
        // them) must have grown along with the desktop.
        let expected = Rect::new(
            new_desktop_bounds.a.x,
            new_desktop_bounds.a.y,
            new_desktop_bounds.b.x - shadow_x,
            new_desktop_bounds.b.y - shadow_y,
        );
        assert_eq!(window.bounds(), expected);

        // The interior child (a stand-in for a window's real content, e.g.
        // an editor's TextViewer) must have been re-laid-out too, not just
        // the window's own bounds: it fills the interior, whose top-left
        // (window top-left + 1) stays put and whose bottom-right (window
        // bottom-right - 1) must have grown by the same amount as the
        // window.
        assert!(set_bounds_calls.get() > 0);
        let interior_child_bounds = window.interior_mut().child_at(0).bounds();
        let expected_interior_top_left = Point::new(window_bounds.a.x + 1, window_bounds.a.y + 1);
        assert_eq!(interior_child_bounds.a, expected_interior_top_left);
        assert_eq!(
            interior_child_bounds.b,
            Point::new(expected.b.x - 1, expected.b.y - 1)
        );
    }

    #[test]
    fn window_set_grow_mode_takes_effect_and_reads_back() {
        let mut window = Window::new(Rect::new(0, 0, 20, 10), "W");

        // Deliberately not gfGrowAll — see the field doc on Window::grow_mode.
        assert_eq!(window.grow_mode(), GF_GROW_HI_X | GF_GROW_HI_Y);

        window.set_grow_mode(GF_GROW_HI_X);
        assert_eq!(window.grow_mode(), GF_GROW_HI_X);
    }

    #[test]
    fn window_with_fixed_grow_mode_stays_put_on_desktop_resize() {
        let (mut app, size, _calls) = build_test_app(80, 25);
        app.handle_redraw();
        let (shadow_x, shadow_y) = crate::core::state::shadow_size();
        let desktop_bounds = app.desktop.get_bounds();
        let window_bounds = Rect::new(
            desktop_bounds.a.x,
            desktop_bounds.a.y,
            desktop_bounds.b.x - shadow_x,
            desktop_bounds.b.y - shadow_y,
        );

        let mut window = Window::new(window_bounds, "Fixed Window");
        window.set_grow_mode(0);
        app.desktop.add(window);

        size.0.store(120, Ordering::SeqCst);
        size.1.store(50, Ordering::SeqCst);
        app.handle_redraw();

        let window_index = app.desktop.child_count() - 1;
        let window = app
            .desktop
            .child_at(window_index)
            .as_any()
            .downcast_ref::<Window>()
            .expect("last desktop child should be the Window");

        assert_eq!(window.bounds(), window_bounds);
    }
}
