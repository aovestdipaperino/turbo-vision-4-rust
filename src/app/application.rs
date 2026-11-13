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
use crate::core::state::State;
use crate::terminal::{CaptureKind, Terminal};
use crate::views::help_context::HelpContext;
use crate::views::help_file::HelpFile;
use crate::views::help_window::HelpWindow;
use crate::views::view::ViewId;
use crate::views::window::WindowLike;
use crate::views::{View, desktop::Desktop, menu_bar::MenuBar, status_line::StatusLine};
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
    /// One-slot pending event queue (Borland: `TProgram::putEvent/pending)`:
    /// returned by the event loops before polling the terminal
    pending_event: Option<Event>,
    /// Overlay widgets that need idle processing and are drawn on top of everything
    /// These widgets continue to animate even during modal dialogs
    /// Matches Borland: `TProgram::idle()` continues running during `execView()`
    pub(crate) overlay_widgets: Vec<Box<dyn View>>,
    // Note: Command set is now stored in thread-local static (command_set module)
    // This matches Borland's architecture where TView::curCommandSet is static
    /// Help file for F1 context-sensitive help
    /// Matches Borland: `TProgram::helpFile` (tprogram.cc)
    help_file: Option<Rc<RefCell<HelpFile>>>,
    /// Help context mappings (context ID to topic ID)
    help_context: HelpContext,
    /// Current help context driving `StatusDef` switching (Borland: the
    /// focused view's helpCtx; set explicitly in this architecture)
    current_help_ctx: u16,
    /// True between a mouse press and its release. While it is set, `idle`
    /// broadcasts `CM_MOUSE_AUTO_REPEAT` so held-down controls, scrollbar
    /// arrows above all, can keep repeating without any polling of their own.
    mouse_held: bool,
    /// Who steps the event loop; see [`is_host_driven`](Application::is_host_driven).
    driver: Driver,
}

/// Who steps an [`Application`]'s event loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Driver {
    /// The application owns a terminal and runs its own loops.
    Terminal,
    /// The backend is host-driven ([`Backend::is_host_driven`]): an embedder
    /// steps the application with [`step`](Application::step), so nothing
    /// may block waiting for input.
    ///
    /// [`Backend::is_host_driven`]: crate::terminal::Backend::is_host_driven
    Host,
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

    /// Called once for every window the desktop removed after `State::CLOSED`.
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
    /// [`Terminal::init()`](crate::terminal::Terminal::init) for details on possible
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
    #[cfg(feature = "native")]
    pub fn new() -> Result<Self> {
        Ok(Self::with_terminal(Terminal::init()?))
    }

    /// Creates an application on an already-built terminal, e.g. one over a
    /// host-driven backend that an embedder steps with [`step`](Self::step).
    ///
    /// This is everything [`new`](Self::new) does except creating the
    /// terminal. Whether the application is host-driven is read from the
    /// backend here, once, through
    /// [`Backend::is_host_driven`](crate::terminal::Backend::is_host_driven).
    ///
    /// A host-driven application also turns off the buttons' press
    /// animation ([`set_press_animation`]): the host steps it only when it
    /// has input, so a command queued to arrive after the animation would
    /// wait for the next key. The setting is per thread, so it also applies
    /// to an application that owns a terminal on the same thread.
    ///
    /// [`set_press_animation`]: crate::views::button::set_press_animation
    pub fn with_terminal(terminal: Terminal) -> Self {
        let driver = if terminal.backend_is_host_driven() {
            crate::views::button::set_press_animation(std::time::Duration::ZERO);
            Driver::Host
        } else {
            Driver::Terminal
        };
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
            driver,
        };

        // Set initial Desktop bounds (adjusts for missing menu/status)
        // Matches Borland: TProgram::initDeskTop() with no menuBar/statusLine
        app.update_desktop_bounds();

        // Initialize Desktop's palette chain now that it's in its final location
        // This sets up the owner chain so views can resolve colors through Desktop's CP_APP_COLOR palette
        app.desktop.init_palette_chain();

        app
    }

    /// Whether an embedder steps this application with [`step`](Self::step)
    /// instead of it owning a terminal and an event loop, as its backend's
    /// [`is_host_driven`](crate::terminal::Backend::is_host_driven) said
    /// when the application was built. When true, modal
    /// loops ([`exec_view`](Self::exec_view) on a modal view,
    /// [`execute_modal`](Self::execute_modal)) refuse to run and return
    /// `CM_CANCEL`.
    #[must_use]
    pub fn is_host_driven(&self) -> bool {
        self.driver == Driver::Host
    }

    /// True (after logging) when this Application is host-driven and `what`
    /// must therefore refuse to run rather than spin waiting for events the
    /// embedder can only deliver after the current call returns. Shared by
    /// every blocking-loop entry point reachable from [`step`](Self::step):
    /// `exec_view`, `execute_modal`, and the History/Dropdown popup openers.
    fn refuse_modal_if_host_driven(&self, what: &str) -> bool {
        if self.is_host_driven() {
            log::error!("{what} executed on a host-driven Application; returning CM_CANCEL");
            true
        } else {
            false
        }
    }

    /// Whether the menu bar has a submenu dropped down.
    ///
    /// A handler's `pre_event` sees keys before the menu bar does, so one
    /// that claims `Enter` or `Esc` for itself must check this first or the
    /// open menu never gets them.
    #[must_use]
    pub fn menu_is_open(&self) -> bool {
        self.menu_bar.as_ref().is_some_and(MenuBar::is_open)
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
    /// Matches Borland: `TProgram::idle()` continues running during `execView()`
    ///
    /// # Examples
    /// ```rust,no_run
    /// use turbo_vision::app::Application;
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
    /// #     fn idle(&mut self) {}
    /// # }
    ///
    /// let mut app = Application::new()?;
    /// let widget = AnimatedWidget(turbo_vision::views::ViewCore::default());
    /// app.add_overlay_widget(widget);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_overlay_widget<V: View + 'static>(&mut self, widget: V) {
        self.overlay_widgets.push(Box::new(widget));
    }

    /// Update Desktop bounds to exclude menu bar and status line areas
    /// Matches Borland: `TProgram::initDeskTop()` calculates bounds based on menuBar/statusLine
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

    /// Queue an event to be returned before the next terminal poll.
    ///
    /// Matches Borland `TProgram::putEvent`: a single-slot pending event that
    /// the event loops consume first (used e.g. to re-enter a command from
    /// event handlers).
    pub fn put_event(&mut self, event: Event) {
        self.pending_event = Some(event);
    }

    /// The next event: the one queued by [`put_event`](Self::put_event) if
    /// there is one, otherwise whatever the terminal's backend returns within
    /// 20 ms. `None` means nothing arrived.
    ///
    /// A backend error is treated as a quit request: it means the backend
    /// connection is gone (e.g. a remote client disconnected), and swallowing
    /// it would leave the event loop spinning forever on a dead session. So
    /// the error is logged, `running` is cleared and `None` is returned.
    ///
    /// This is the poll [`run_with`](Self::run_with) uses. An embedder that
    /// steps the application itself drains its events with this and hands
    /// each one to [`step`](Self::step); on a backend whose `poll_event`
    /// never waits, as a host-driven one's shouldn't, it never waits either.
    pub fn poll_event_or_quit(&mut self) -> Option<Event> {
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
    /// Matches Borland/Magiblot: `TProgram::getEvent()` (tprogram.cc:105-174)
    /// This is called by modal views' `execute()` methods.
    ///
    /// Key behavior (matches magiblot):
    /// - Draws the screen first
    /// - Blocks waiting for events (default 20ms timeout)
    /// - Only calls `idle()` when there are NO events after timeout
    /// - This gives true event-driven behavior with minimal CPU usage
    pub fn get_event(&mut self) -> Option<Event> {
        // Draw everything (this is the key: drawing happens BEFORE getting events)
        // Matches Borland's CLY_Redraw() in getEvent
        self.draw();
        let _ = self.terminal.flush();

        // Poll for event with 20ms timeout (matches magiblot's eventTimeoutMs)
        // This blocks until an event arrives or timeout occurs
        if let Some(event) = self.poll_event_or_quit() {
            // Event received - return it immediately without calling idle()
            // Matches magiblot: idle() is NOT called when events are present
            Some(event)
        } else {
            // Timeout occurred with no events - now we call idle()
            // Matches magiblot: idle() only called when truly idle
            // This is where animations update, command sets broadcast, etc.
            self.idle();
            None
        }
    }

    /// Execute a view (modal or modeless)
    /// Matches Borland: `TProgram::execView()` (tprogram.cc:177-197)
    ///
    /// If the view has `State::MODAL` flag set, runs a modal event loop.
    /// Otherwise, adds the view to the desktop and returns immediately.
    ///
    /// Returns the view's `end_state` (the command that closed the modal view)
    pub fn exec_view<V: View + 'static>(&mut self, view: V) -> CommandId {
        let view: Box<dyn View> = Box::new(view);

        // Check if view is modal
        let is_modal = view.state().contains(State::MODAL);

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

        if self.refuse_modal_if_host_driven("modal view") {
            self.desktop.remove_child_by_id(view_id);
            return CM_CANCEL;
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
                    .map_or(0, super::super::views::group::GroupLike::end_state);
                if end_state != 0 {
                    // A failing validator vetoes the close (Borland:
                    // do { ... } while( !valid(endState) ))
                    let close_ok = self
                        .desktop
                        .child_by_id_mut(view_id)
                        .is_none_or(|child| child.valid(end_state));
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
    ///
    /// While the loop runs, `view` is the selected window, so its window
    /// commands are enabled; the command set from before is put back
    /// afterwards (Borland: `TGroup::execView` saves and restores it).
    pub fn execute_modal<V, F>(&mut self, view: &mut V, tick: F) -> CommandId
    where
        V: WindowLike + ?Sized,
        F: FnMut(&mut Application, &mut V) -> ModalTick,
    {
        let saved = command_set::get_commands();
        view.window().enable_window_commands();
        let result = self.run_modal_loop(view, tick);
        command_set::set_commands(saved);
        result
    }

    /// The loop behind [`execute_modal`](Self::execute_modal).
    fn run_modal_loop<V, F>(&mut self, view: &mut V, mut tick: F) -> CommandId
    where
        V: WindowLike + ?Sized,
        F: FnMut(&mut Application, &mut V) -> ModalTick,
    {
        if self.refuse_modal_if_host_driven("modal view") {
            return CM_CANCEL;
        }

        loop {
            // Draw desktop first (clears the background), then the modal view
            // on top: a view that is not on the desktop is a child of the
            // application here, so its bounds are screen coordinates.
            self.draw_chrome();
            Self::draw_child(&mut self.terminal, view);
            // Overlay widgets keep animating during modal loops
            // (Borland: TProgram::idle() continues running during execView()).
            for widget in &mut self.overlay_widgets {
                Self::draw_child(&mut self.terminal, widget);
            }
            self.terminal.push_origin(view.bounds().a);
            view.update_cursor(&mut self.terminal);
            self.terminal.pop_origin();
            let _ = self.terminal.flush();

            // 20ms timeout matches magiblot's eventTimeoutMs; idle() only runs
            // when there truly was no event.
            match self.poll_event_or_quit() {
                Some(mut event) => {
                    if event.what == EventType::Broadcast && event.command == CM_REDRAW {
                        self.handle_redraw();
                        continue;
                    }

                    Self::dispatch_child(view, &mut event);

                    // A keyboard event turned into a command (Enter -> CM_OK)
                    // is re-dispatched so the command handler runs (putEvent).
                    if event.what == EventType::Command {
                        Self::dispatch_child(view, &mut event);
                    }

                    // A window key nobody in the view wanted (Alt+F3 closes,
                    // which a modal window turns into CM_CANCEL). Borland's
                    // status line sees keys during execView too.
                    if event.what == EventType::Keyboard {
                        if let Some(command) = default_window_command(&event) {
                            event = Event::command(command);
                            Self::dispatch_child(view, &mut event);
                        }
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
            self.draw_if_needed();

            // Poll for event with 20ms timeout (matches magiblot's eventTimeoutMs)
            // This blocks until an event arrives or timeout occurs
            let event = self.poll_event_or_quit();
            self.iterate(handler, event);
        }
    }

    /// Draw and flush a full frame if one was asked for since the last.
    fn draw_if_needed(&mut self) {
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
    }

    /// One pass of the event loop around an event the caller already has,
    /// for an embedder that steps the application itself instead of calling
    /// [`run_with`](Self::run_with).
    ///
    /// First a pending full redraw (a resize, a closed window, a palette
    /// change) is drawn and flushed, as `run_with` does before each poll.
    /// Then `Some(event)` goes through `handler.pre_event`, the menu bar,
    /// status line and desktop, and `handler.handle_command` if it is still
    /// a command, and the screen is redrawn; `None` stands for a poll that
    /// found nothing and runs [`idle`](Self::idle) and `handler.idle`
    /// instead. Last, windows that closed are removed (each one reported to
    /// `handler.window_closed`) and windows that moved are redrawn.
    ///
    /// A host-driven embedder's frame is typically: resize to
    /// [`Terminal::backend_size`] with [`handle_redraw`](Self::handle_redraw)
    /// if it differs from [`Terminal::size`], then `step(handler, Some(e))`
    /// for each event [`poll_event_or_quit`](Self::poll_event_or_quit)
    /// returns while [`running`](Self::running), one `step(handler, None)`,
    /// and a final [`draw`](Self::draw), after which the frame is in
    /// [`Terminal::buffer`]. The cursor that draw asked for is
    /// [`Terminal::cursor`]; the backend only sees it on the next
    /// [`Terminal::flush`], so a host either reads `terminal.cursor()` or
    /// flushes after the final draw. Nothing here waits for input, and on a
    /// host-driven application nothing reachable from it does either: modal
    /// views and popups refuse with `CM_CANCEL`.
    pub fn step<H: AppHandler>(&mut self, handler: &mut H, event: Option<Event>) {
        self.draw_if_needed();
        self.iterate(handler, event);
    }

    /// The half of one event-loop pass that follows the poll, around an
    /// already-polled `event` (`None` for a poll that timed out): shared by
    /// [`run_with`](Self::run_with), which calls
    /// [`draw_if_needed`](Self::draw_if_needed) before its poll, and
    /// [`step`](Self::step), which calls it first.
    fn iterate<H: AppHandler>(&mut self, handler: &mut H, event: Option<Event>) {
        // The event the caller polled; `None` means the poll timed out
        if let Some(mut event) = event {
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
        } else {
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

        // Remove closed windows (those with State::CLOSED flag)
        // In Borland, views call CLY_destroy() to remove themselves
        // In Rust, views set State::CLOSED and parent removes them
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
        self.terminal.push_origin(self.desktop.bounds().a);
        let had_moved_windows = self.desktop.handle_moved_windows(&mut self.terminal);
        self.terminal.pop_origin();
        if had_moved_windows {
            // Window movement: partial redraw already done via draw_under_rect
            // Just flush the terminal buffer
            let _ = self.terminal.flush();
        }
    }

    pub fn draw(&mut self) {
        // Desktop first, then menu bar on top (so dropdowns appear over it);
        // each top-level view draws in its own space.
        self.draw_chrome();

        // Draw overlay widgets on top of everything
        // These continue to animate even during modal dialogs
        for widget in &mut self.overlay_widgets {
            Self::draw_child(&mut self.terminal, widget);
        }

        // Update cursor after drawing all views: the focused control in the
        // desktop's windows shows it, except under an open menu, which owns
        // the keyboard and may cover that control.
        if self.menu_bar.as_ref().is_some_and(MenuBar::is_open) {
            let _ = self.terminal.hide_cursor();
        } else {
            self.terminal.push_origin(self.desktop.bounds().a);
            self.desktop.update_cursor(&mut self.terminal);
            self.terminal.pop_origin();
        }
    }

    /// Draw the desktop, menu bar and status line, each in its own space.
    fn draw_chrome(&mut self) {
        // With no menu bar or status line the desktop still leaves its row
        // free (Borland: initDeskTop), and no view draws there; blank it, or
        // a bar taken away at run time stays on the screen.
        let (width, height) = self.terminal.size();
        let desktop = self.desktop.bounds();
        let blank =
            vec![
                crate::core::draw::Cell::new(' ', crate::core::palette::Attr::from_u8(0x07));
                usize::try_from(width).unwrap_or(0)
            ];
        if self.menu_bar.is_none() {
            for y in 0..desktop.a.y {
                self.terminal.write_line(0, y, &blank);
            }
        }
        if self.status_line.is_none() {
            for y in desktop.b.y..height {
                self.terminal.write_line(0, y, &blank);
            }
        }
        Self::draw_child(&mut self.terminal, &mut self.desktop);
        if let Some(ref mut menu_bar) = self.menu_bar {
            Self::draw_child(&mut self.terminal, menu_bar);
        }
        if let Some(ref mut status_line) = self.status_line {
            Self::draw_child(&mut self.terminal, status_line);
        }
    }

    /// Draw one top-level view with its origin pushed (the application is the
    /// root, so its children's bounds are screen coordinates).
    fn draw_child(terminal: &mut Terminal, view: &mut (impl View + ?Sized)) {
        terminal.push_origin(view.bounds().a);
        view.draw(terminal);
        terminal.pop_origin();
    }

    /// Dispatch an event to one top-level view in that view's space and put
    /// the mouse position back into screen space afterwards.
    fn dispatch_child(view: &mut (impl View + ?Sized), event: &mut Event) {
        let origin = view.bounds().a;
        event.mouse.pos.x -= origin.x;
        event.mouse.pos.y -= origin.y;
        view.handle_event(event);
        event.mouse.pos.x += origin.x;
        event.mouse.pos.y += origin.y;
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
                    if !self.terminal.run_capture_hook(CaptureKind::Ansi) {
                        self.dump_screen_ansi();
                    }
                    event.clear();
                    return;
                }
                KB_CTRL_F12 => {
                    self.capture_png();
                    event.clear();
                    return;
                }
                code @ crate::core::event::KB_ALT_1..=crate::core::event::KB_ALT_9 => {
                    // Alt+digit selects the numbered window
                    // (Borland: TProgram::handleEvent -> cmSelectWindowNum)
                    let num = ((code - crate::core::event::KB_ALT_1) >> 8) + 1;
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
            Self::dispatch_child(menu_bar, event);
            if event.what == EventType::Nothing {
                return;
            }
        }

        // Desktop/windows
        Self::dispatch_child(&mut self.desktop, event);
        if event.what == EventType::Nothing {
            return;
        }

        // Status line
        if let Some(ref mut status_line) = self.status_line {
            let was_command = event.what == EventType::Command;
            Self::dispatch_child(status_line, event);
            if event.what == EventType::Nothing {
                return;
            }
            // A key or click the status line turned into a command goes round
            // again, so the menu bar and the windows see it before the
            // application does (Borland: TStatusLine::handleEvent putEvent).
            // Without this an Alt+F3 status item's CM_CLOSE never reached the
            // window it was meant to close.
            if !was_command && event.what == EventType::Command {
                self.handle_event(event);
                return;
            }
        }

        // Borland's default window keys, the bindings TProgram's status line
        // carries. They come after every view has had the key, so a focused
        // view that uses one (TabbedPane's F6) keeps it; the command then goes
        // round again the way the status line's putEvent would send it, and
        // reaches the caller's handler if no window takes it.
        if event.what == EventType::Keyboard {
            if let Some(command) = default_window_command(event) {
                *event = Event::command(command);
                self.handle_event(event);
            }
            return;
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
                    self.capture_png();
                    event.clear();
                }
                crate::core::command::CM_SHOW_HISTORY => {
                    // A History button was clicked in a window running under
                    // exec_view()/run(); open the popup here where we have
                    // terminal access, then fill the linked input.
                    use crate::core::geometry::Point;
                    use crate::core::history::HistoryManager;
                    if self.refuse_modal_if_host_driven("History popup") {
                        event.clear();
                        return;
                    }

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
                    if self.refuse_modal_if_host_driven("Dropdown popup") {
                        event.clear();
                        return;
                    }
                    crate::views::dialog::show_dropdown_popup(event, &mut self.terminal);
                }
                _ => {}
            }
        }
    }

    /// Serve Ctrl+F12 / `CM_SCREENSHOT`: the capture hook if one is
    /// installed, otherwise the built-in PNG screenshot. Without the
    /// `screenshot` feature there is no built-in one, and only the hook runs.
    fn capture_png(&mut self) {
        if self.terminal.run_capture_hook(CaptureKind::Png) {
            return;
        }
        #[cfg(feature = "screenshot")]
        self.take_screenshot();
        #[cfg(not(feature = "screenshot"))]
        log::debug!("PNG screenshots are not built in (the `screenshot` feature is off)");
    }

    /// Save a PNG screenshot of the current screen (bound to Ctrl+F12).
    ///
    /// Needs the `screenshot` feature (on by default).
    ///
    /// The file is written to the current working directory with a
    /// timestamped name like `screenshot-20260607-194800.png`. Rendering
    /// queries the current font cell size; see
    /// [`Terminal::save_screenshot_png`](crate::terminal::Terminal::save_screenshot_png).
    #[cfg(feature = "screenshot")]
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

    // Localisation

    /// Show Turbo Vision's messages, and those added with
    /// [`add_translations`](Self::add_translations), in the locale `tag`
    /// (`"ca"`, `"es-ES"`); returns the locale used first. Call it before
    /// building menus and dialogs: views keep the text they were made with.
    /// See [`crate::i18n`].
    pub fn set_locale(&mut self, tag: &str) -> String {
        let used = crate::i18n::set_locale(tag);
        self.needs_redraw();
        used
    }

    /// Use the user's locale (`LC_ALL`, `LC_MESSAGES`, `LANG`), or English;
    /// returns the locale used first. See [`set_locale`](Self::set_locale).
    pub fn set_locale_from_env(&mut self) -> String {
        let used = crate::i18n::set_locale_from_env();
        self.needs_redraw();
        used
    }

    /// Add the application's Fluent messages for `locale`, for
    /// [`crate::i18n::tr`] and the [`tr!`](crate::tr) macro.
    ///
    /// # Errors
    ///
    /// When `locale` is not a language tag or `source` does not parse.
    pub fn add_translations(
        &mut self,
        locale: &str,
        source: &str,
    ) -> std::result::Result<(), crate::i18n::TranslationError> {
        crate::i18n::add_translations(locale, source)
    }

    // Help System Methods
    // Matches Borland: TProgram help support (tprogram.cc)

    /// Set the help file for F1 context-sensitive help
    /// Matches Borland: `TApplication::helpFile` initialization
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
    ///
    /// The version for the current locale wins when it exists: with Catalan
    /// chosen, `help/manual.ca.md` is loaded instead of `help/manual.md`
    /// (see [`crate::i18n::localized_path`]).
    pub fn set_help_file(&mut self, path: &str) -> std::io::Result<()> {
        let help_file = HelpFile::new(crate::i18n::localized_path(path))?;
        self.help_file = Some(Rc::new(RefCell::new(help_file)));
        Ok(())
    }

    /// Set a pre-built help file for F1 context-sensitive help
    pub fn set_help(&mut self, help_file: HelpFile) {
        self.help_file = Some(Rc::new(RefCell::new(help_file)));
    }

    /// Register a help context mapping (context ID to topic ID)
    /// This allows views to have `help_context` set, and F1 will open the corresponding topic
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
        if let Some(ref help_file) = self.help_file {
            let (width, height) = self.terminal.size();
            let help_width = (width * 3 / 4).max(40).min(width - 4);
            let help_height = (height * 3 / 4).max(10).min(height - 4);
            let x = (width - help_width) / 2;
            let y = (height - help_height) / 2;

            let bounds = Rect::new(x, y, x + help_width, y + help_height);
            let mut help_window =
                HelpWindow::new(bounds, &crate::i18n::tr("help-title"), Rc::clone(help_file));
            help_window.show_topic(topic_id);

            // Set State::MODAL flag so exec_view runs the modal loop
            // Matches Borland: THelpWindow is displayed modally
            let current_state = help_window.state();
            help_window.set_state(current_state | State::MODAL);

            // Execute the help window as modal
            self.exec_view(help_window);
        }
    }

    /// Show context-sensitive help
    /// Looks up the focused view's help context and opens the appropriate topic
    /// Matches Borland: `TProgram::getEvent()` F1 handling
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
    /// Matches Borland: `TApplication::tile()` (tapplica.cpp:123-127)
    pub fn tile(&mut self) {
        let rect = self.get_tile_rect();
        self.desktop.tile_with_rect(rect);
    }

    /// Cascade all tileable windows in a staircase pattern
    /// Matches Borland: `TApplication::cascade()` (tapplica.cpp:75-79)
    pub fn cascade(&mut self) {
        let rect = self.get_tile_rect();
        self.desktop.cascade_with_rect(rect);
    }

    /// Get the rectangle to use for tiling/cascading operations
    /// Matches Borland: `TApplication::getTileRect()` (tapplica.cpp:94-97)
    /// Default implementation returns the full desktop extent
    /// Can be overridden to customize the tile area
    pub fn get_tile_rect(&self) -> Rect {
        self.desktop.extent()
    }

    // Command Set Management
    // Delegates to global command set functions (command_set module)
    // Matches Borland's TView command set methods (tview.cc:161-389, 672-677)

    /// Check if a command is currently enabled
    /// Matches Borland: `TView::commandEnabled(ushort` command) (tview.cc:142-147)
    pub fn command_enabled(&self, command: CommandId) -> bool {
        command_set::command_enabled(command)
    }

    /// Enable a single command
    /// Matches Borland: `TView::enableCommand(ushort` command) (tview.cc:384-389)
    pub fn enable_command(&mut self, command: CommandId) {
        command_set::enable_command(command);
    }

    /// Disable a single command
    /// Matches Borland: `TView::disableCommand(ushort` command) (tview.cc:161-166)
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
    /// Matches Borland: `TScreen::makeBeep()` - provides audio feedback for errors/alerts
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
        if !(250..=1500).contains(&timeout_ms) {
            return Err(crate::core::error::TurboVisionError::invalid_input(
                format!("ESC timeout must be between 250 and 1500 milliseconds, got {timeout_ms}"),
            ));
        }
        self.terminal.set_esc_timeout(timeout_ms);
        Ok(())
    }

    /// Idle processing - broadcasts command set changes and updates command states
    /// Matches Borland: `TProgram::idle()` (tprogram.cc:248-257)
    /// Set the current help context.
    ///
    /// Matches Borland: `TProgram` tracks the focused view's helpCtx; since
    /// views here don't carry one, applications set it when the active
    /// window changes. The status line's `TStatusDef` item set follows it on
    /// the next idle tick (see `StatusLine::with_defs`).
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
    /// Matches Borland: `TProgram::suspend()` - temporarily exits TUI mode
    /// Restores terminal to normal mode, allowing user to return to shell
    /// Call `resume()` to return to TUI mode
    pub fn suspend(&mut self) -> crate::core::error::Result<()> {
        self.terminal.suspend()
    }

    /// Resume the application after suspension (for Ctrl+Z handling)
    /// Matches Borland: `TProgram::resume()` - returns to TUI mode and redraws
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

/// The command one of Borland's default window keys stands for, or `None`
/// when `event` is not one of them.
///
/// | Key | Command |
/// |---|---|
/// | Alt+F3 | `CM_CLOSE` |
/// | F5 | `CM_ZOOM` |
/// | Ctrl+F5 | `CM_RESIZE` |
/// | F6 | `CM_NEXT` |
/// | Shift+F6 | `CM_PREV` |
///
/// Only enabled commands are returned: the selected window enables its
/// window commands (see `Window::enable_window_commands`), and an
/// application can disable any of them.
///
/// Matches the keys `TProgram::initStatusLine` binds (stdStatusKeys). The
/// terminal reports Ctrl+F5 and Shift+F6 as the plain key plus a modifier;
/// the BIOS codes `KB_CTRL_F5` and `KB_SHIFT_F6` are accepted too.
fn default_window_command(event: &Event) -> Option<CommandId> {
    use crate::core::command::{CM_CLOSE, CM_NEXT, CM_PREV, CM_RESIZE, CM_ZOOM};
    use crate::core::event::{KB_ALT_F3, KB_CTRL_F5, KB_F5, KB_F6, KB_SHIFT_F6};
    use crate::core::keys::KeyModifiers;

    // Only these three matter; anything else the backend reports is ignored.
    let m = event.key_modifiers;
    let (ctrl, shift, alt) = (
        m.contains(KeyModifiers::CONTROL),
        m.contains(KeyModifiers::SHIFT),
        m.contains(KeyModifiers::ALT),
    );
    let command = match event.key_code {
        KB_ALT_F3 => Some(CM_CLOSE),
        KB_CTRL_F5 => Some(CM_RESIZE),
        KB_SHIFT_F6 => Some(CM_PREV),
        KB_F5 if !ctrl && !shift && !alt => Some(CM_ZOOM),
        KB_F5 if ctrl && !shift && !alt => Some(CM_RESIZE),
        KB_F6 if !ctrl && !shift && !alt => Some(CM_NEXT),
        KB_F6 if shift && !ctrl && !alt => Some(CM_PREV),
        _ => None,
    };
    // Like a status-line item, a key whose command is disabled does nothing.
    command.filter(|&c| command_set::command_enabled(c))
}

impl Drop for Application {
    fn drop(&mut self) {
        let _ = self.terminal.shutdown();
    }
}

#[cfg(test)]
mod resize_tests {
    use super::*;
    use crate::core::state::Grow;

    // ---- owner-relative coordinates (docs/OWNER-COORDINATES.md) ----

    fn row_text(app: &Application, y: i16, x0: i16, x1: i16) -> String {
        (x0..x1)
            .map(|x| app.terminal.read_cell(x, y).unwrap().ch)
            .collect()
    }

    /// Desktop at (0,1) with a window at desktop (10,3) holding a button at
    /// interior (2,1); returns the app and the button's command.
    fn desktop_window_button() -> (Application, crate::core::command::CommandId) {
        use crate::views::button::Button;
        let cmd = crate::core::command::CM_USER + 50;
        let (mut app, _size, _calls) = build_test_app(80, 25);
        app.handle_redraw(); // desktop becomes (0,1)-(80,24): a non-zero origin
        assert_eq!(app.desktop.bounds().a, Point::new(0, 1));
        let mut window = Window::new(Rect::new(10, 3, 50, 13), "W");
        window.add(Button::new(Rect::new(2, 1, 12, 3), "OK", cmd, true));
        app.desktop.add(window);
        (app, cmd)
    }

    fn window_mut(app: &mut Application) -> &mut crate::views::window::Window {
        let idx = app.desktop.child_count() - 1;
        app.desktop
            .child_at_mut(idx)
            .as_any_mut()
            .downcast_mut::<crate::views::window::Window>()
            .unwrap()
    }

    #[test]
    fn a_button_draws_at_the_sum_of_its_owners_origins() {
        let (mut app, _) = desktop_window_button();
        app.draw();
        // desktop (0,1) + window (10,3) + interior (1,1) + button (2,1) = (13,6)
        assert!(
            row_text(&app, 6, 13, 23).contains("OK"),
            "{:?}",
            row_text(&app, 6, 13, 23)
        );
        // and the button never learned where that was
        let w = window_mut(&mut app);
        assert_eq!(
            w.interior_mut().child_at(0).bounds(),
            Rect::new(2, 1, 12, 3)
        );
    }

    #[test]
    fn moving_a_window_moves_the_pixels_but_not_the_child_s_bounds() {
        let (mut app, _) = desktop_window_button();
        window_mut(&mut app).set_bounds(Rect::new(20, 5, 60, 15));
        app.draw();
        assert!(
            row_text(&app, 8, 23, 33).contains("OK"),
            "{:?}",
            row_text(&app, 8, 23, 33)
        );
        assert!(!row_text(&app, 6, 13, 23).contains("OK"));
        let w = window_mut(&mut app);
        assert_eq!(
            w.interior_mut().child_at(0).bounds(),
            Rect::new(2, 1, 12, 3)
        );
    }

    #[test]
    fn a_click_reaches_the_button_through_every_owner() {
        use crate::core::event::MB_LEFT_BUTTON;
        let (mut app, cmd) = desktop_window_button();
        let at = Point::new(15, 6); // screen cell inside the button
        let mut down = Event::mouse(EventType::MouseDown, at, MB_LEFT_BUTTON, false);
        app.handle_event(&mut down);
        let mut up = Event::mouse(EventType::MouseUp, at, MB_LEFT_BUTTON, false);
        app.handle_event(&mut up);
        assert_eq!(up.what, EventType::Command, "the button fired on release");
        assert_eq!(up.command, cmd);
        // A fresh command event starts at the button's local (0,0); every
        // owner added its origin on the way back, so the root sees the
        // button's screen origin: desktop (0,1) + window (10,3) + interior
        // (1,1) + button (2,1).
        assert_eq!(up.mouse.pos, Point::new(13, 6));
    }

    #[test]
    fn dragging_a_window_by_its_title_works_even_when_the_pointer_leaves_it() {
        use crate::core::event::MB_LEFT_BUTTON;
        let (mut app, _) = desktop_window_button();
        // title row of the window: desktop y 1 + window y 3 = screen row 4
        let mut down = Event::mouse(
            EventType::MouseDown,
            Point::new(25, 4),
            MB_LEFT_BUTTON,
            false,
        );
        app.handle_event(&mut down);
        let mut mv = Event::mouse(
            EventType::MouseMove,
            Point::new(40, 10),
            MB_LEFT_BUTTON,
            false,
        );
        app.handle_event(&mut mv);
        assert_eq!(window_mut(&mut app).bounds(), Rect::new(25, 9, 65, 19));
        // jump far outside the window's new extent: still delivered to the
        // dragging window, and clamped to the desktop's left edge
        let mut mv = Event::mouse(
            EventType::MouseMove,
            Point::new(12, 6),
            MB_LEFT_BUTTON,
            false,
        );
        app.handle_event(&mut mv);
        assert_eq!(window_mut(&mut app).bounds(), Rect::new(0, 5, 40, 15));
    }

    #[test]
    fn a_history_click_reaches_the_root_with_a_screen_space_anchor() {
        use crate::core::command::CM_SHOW_HISTORY;
        use crate::core::event::MB_LEFT_BUTTON;
        use crate::core::history::HistoryManager;
        use crate::views::history::History;
        use crate::views::input_line::InputLine;
        use crate::views::window::Window;
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear(77);
        HistoryManager::add(77, "entry".to_string());

        let (mut app, _size, _calls) = build_test_app(80, 25);
        app.handle_redraw();
        let mut window = Window::new(Rect::new(10, 3, 50, 13), "W");
        let input = window.add_typed(InputLine::new(Rect::new(2, 3, 20, 4), 32));
        window.add(History::new(Point::new(21, 3), 77, input));
        app.desktop.add(window);

        // desktop (0,1) + window (10,3) + interior (1,1) + history (21,3) = (32,8)
        let at = Point::new(32, 8);
        let mut ev = Event::mouse(EventType::MouseDown, at, MB_LEFT_BUTTON, false);
        // Through the desktop only: the application would open the popup.
        crate::views::view::dispatch_to_child(&mut app.desktop, &mut ev);
        assert_eq!(ev.what, EventType::Command);
        assert_eq!(ev.command, CM_SHOW_HISTORY);
        assert_eq!(ev.info, 77);
        assert_eq!(ev.mouse.pos, at, "anchor translated back to screen space");
    }

    #[test]
    fn execute_modal_stops_when_the_tick_says_so_and_dispatches_events_to_the_view() {
        use crate::core::command::{CM_CANCEL, CM_OK};
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
        dialog.set_state(dialog.state() | State::MODAL);
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
            driver: Driver::Terminal,
        };

        app.set_menu_bar(MenuBar::new(Rect::new(0, 0, width, 1)));
        app.set_status_line(StatusLine::new(
            Rect::new(0, height - 1, width, height),
            Vec::new(),
        ));

        let set_bounds_calls = Rc::new(StdCell::new(0));
        let mut inner_group = crate::views::group::Group::new(Rect::new(0, 0, width, height - 2));
        inner_group.set_grow_mode(Grow::HI_X | Grow::HI_Y);
        inner_group.add(RecordingView {
            core: ViewCore {
                bounds: Rect::new(0, 0, width, height - 2),
                grow_mode: Grow::HI_X | Grow::HI_Y,
                ..ViewCore::default()
            },
            set_bounds_calls: Rc::clone(&set_bounds_calls),
        });

        let mut group_view = GroupView(inner_group);
        group_view.set_grow_mode(Grow::HI_X | Grow::HI_Y);
        app.desktop.add(group_view);

        (app, size, set_bounds_calls)
    }

    /// A backend that logs every poll and flush, so a test can see which of
    /// the two the loop does first.
    struct OrderBackend {
        log: Arc<std::sync::Mutex<Vec<&'static str>>>,
    }

    impl crate::terminal::Backend for OrderBackend {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn init(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn cleanup(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn size(&self) -> std::io::Result<(u16, u16)> {
            Ok((80, 25))
        }
        fn poll_event(&mut self, _timeout: Duration) -> std::io::Result<Option<Event>> {
            self.log.lock().unwrap().push("poll");
            Ok(None)
        }
        fn write_raw(&mut self, _data: &[u8]) -> std::io::Result<()> {
            Ok(())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.log.lock().unwrap().push("flush");
            Ok(())
        }
        fn show_cursor(&mut self, _x: u16, _y: u16) -> std::io::Result<()> {
            Ok(())
        }
        fn hide_cursor(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn run_with_draws_a_pending_redraw_before_it_polls() {
        struct AskRedrawOnce {
            log: Arc<std::sync::Mutex<Vec<&'static str>>>,
            idles: u32,
        }
        impl AppHandler for AskRedrawOnce {
            fn idle(&mut self, app: &mut Application) {
                self.idles += 1;
                self.log.lock().unwrap().push("idle");
                if self.idles == 1 {
                    app.needs_redraw();
                } else {
                    app.running = false;
                }
            }
        }
        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let (mut app, _size, _calls) = build_test_app(80, 25);
        app.terminal = Terminal::with_backend(Box::new(OrderBackend {
            log: Arc::clone(&log),
        }))
        .unwrap();
        app.needs_redraw = false;
        let mut handler = AskRedrawOnce {
            log: Arc::clone(&log),
            idles: 0,
        };
        app.run_with(&mut handler);

        let log = log.lock().unwrap();
        let first_idle = log.iter().position(|e| *e == "idle").unwrap();
        // The redraw the first idle asked for is flushed before the next poll,
        // not after it (a poll can wait 20 ms).
        assert_eq!(log[first_idle + 1], "flush", "log: {log:?}");
        assert_eq!(log[first_idle + 2], "poll", "log: {log:?}");
    }

    #[test]
    fn an_application_over_an_ordinary_backend_is_not_host_driven() {
        let (app, _size, _calls) = build_test_app(80, 25);
        assert!(!app.is_host_driven());
    }

    #[test]
    fn step_draws_a_pending_redraw_before_the_event() {
        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let (mut app, _size, _calls) = build_test_app(80, 25);
        app.terminal = Terminal::with_backend(Box::new(OrderBackend {
            log: Arc::clone(&log),
        }))
        .unwrap();
        app.needs_redraw = true;
        app.step(&mut (), None);
        assert!(!app.needs_redraw, "the pending redraw was drawn");
        assert_eq!(log.lock().unwrap().first(), Some(&"flush"));
        assert!(
            !log.lock().unwrap().contains(&"poll"),
            "step never polls: the caller already has the event"
        );
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
    fn a_menu_bar_taken_away_leaves_a_blank_row() {
        let (mut app, _size, _calls) = build_test_app(80, 25);
        let mut bar = MenuBar::new(Rect::new(0, 0, 80, 1));
        let menu = crate::core::menu_data::Menu::from_items(vec![]);
        bar.add_submenu(crate::views::menu_bar::SubMenu::new("~F~ile", menu));
        app.set_menu_bar(bar);
        app.draw();
        let top_row = |app: &Application| -> String {
            (0..10)
                .map(|x| app.terminal.read_cell(x, 0).map_or('?', |c| c.ch))
                .collect()
        };
        assert!(top_row(&app).contains("File"));

        app.menu_bar = None;
        app.draw();
        assert_eq!(top_row(&app), " ".repeat(10));
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
        // Window bounds are desktop-relative
        let desktop_extent = app.desktop.extent();
        let window_bounds = Rect::new(
            0,
            0,
            desktop_extent.b.x - shadow_x,
            desktop_extent.b.y - shadow_y,
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
                grow_mode: Grow::HI_X | Grow::HI_Y,
                ..ViewCore::default()
            },
            set_bounds_calls: Rc::clone(&set_bounds_calls),
        });
        app.desktop.add(window);

        size.0.store(120, Ordering::SeqCst);
        size.1.store(50, Ordering::SeqCst);
        app.handle_redraw();

        let window_index = app.desktop.child_count() - 1;
        let new_desktop_extent = app.desktop.extent();
        let window = app
            .desktop
            .child_at_mut(window_index)
            .as_any_mut()
            .downcast_mut::<Window>()
            .expect("last desktop child should be the Window");

        // The window's own bounds (and therefore its frame, which shares
        // them) must have grown along with the desktop.
        let expected = Rect::new(
            0,
            0,
            new_desktop_extent.b.x - shadow_x,
            new_desktop_extent.b.y - shadow_y,
        );
        assert_eq!(window.bounds(), expected);

        // The interior child (a stand-in for a window's real content, e.g.
        // an editor's TextViewer) must have been re-laid-out too, not just
        // the window's own bounds: it fills the interior, so in interior
        // space its top-left stays at (0, 0) and its bottom-right is the
        // window size less the frame.
        assert!(set_bounds_calls.get() > 0);
        let interior_child_bounds = window.interior_mut().child_at(0).bounds();
        assert_eq!(interior_child_bounds.a, Point::new(0, 0));
        assert_eq!(
            interior_child_bounds.b,
            Point::new(expected.width() - 2, expected.height() - 2)
        );
    }

    #[test]
    fn window_set_grow_mode_takes_effect_and_reads_back() {
        let mut window = Window::new(Rect::new(0, 0, 20, 10), "W");

        // Deliberately not gfGrowAll — see the field doc on Window::grow_mode.
        assert_eq!(window.grow_mode(), Grow::HI_X | Grow::HI_Y);

        window.set_grow_mode(Grow::HI_X);
        assert_eq!(window.grow_mode(), Grow::HI_X);
    }

    #[test]
    fn window_with_fixed_grow_mode_stays_put_on_desktop_resize() {
        let (mut app, size, _calls) = build_test_app(80, 25);
        app.handle_redraw();
        let (shadow_x, shadow_y) = crate::core::state::shadow_size();
        // Window bounds are desktop-relative
        let desktop_extent = app.desktop.extent();
        let window_bounds = Rect::new(
            0,
            0,
            desktop_extent.b.x - shadow_x,
            desktop_extent.b.y - shadow_y,
        );

        let mut window = Window::new(window_bounds, "Fixed Window");
        window.set_grow_mode(Grow::empty());
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

#[cfg(test)]
mod capture_tests {
    use super::*;

    #[test]
    fn capture_keys_run_the_capture_hook() {
        use std::sync::{Arc, Mutex};

        let mut app = Application::with_terminal(crate::test_util::test_terminal(80, 25));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = Arc::clone(&seen);
        app.terminal
            .set_capture_hook(Box::new(move |kind, _| log.lock().unwrap().push(kind)));

        for mut event in [
            Event::keyboard(KB_CTRL_F12),
            Event::keyboard(KB_F12),
            Event::command(CM_SCREENSHOT),
        ] {
            app.handle_event(&mut event);
        }
        assert_eq!(
            *seen.lock().unwrap(),
            vec![CaptureKind::Png, CaptureKind::Ansi, CaptureKind::Png]
        );
    }

    /// The PNG captures in the working directory, where the built-in
    /// screenshot would write one.
    #[cfg(not(feature = "screenshot"))]
    fn png_captures() -> Vec<std::path::PathBuf> {
        std::fs::read_dir(".")
            .unwrap()
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("screenshot-") && n.ends_with(".png"))
            })
            .collect()
    }

    #[test]
    #[cfg(not(feature = "screenshot"))]
    fn without_the_screenshot_feature_png_capture_does_nothing() {
        let mut app = Application::with_terminal(crate::test_util::test_terminal(80, 25));
        let before = png_captures();
        for mut event in [Event::keyboard(KB_CTRL_F12), Event::command(CM_SCREENSHOT)] {
            app.handle_event(&mut event);
            assert_eq!(
                event.what,
                EventType::Nothing,
                "the capture key is consumed"
            );
        }
        assert_eq!(png_captures(), before, "no PNG file is written");
    }
}

#[cfg(test)]
mod dropdown_tests {
    use super::*;
    use crate::core::event::{KB_ESC, KB_F4, MB_LEFT_BUTTON};
    use crate::core::geometry::Point;
    use crate::views::combo_box::ComboBox;
    use crate::views::dialog::Dialog;
    use crate::views::group::GroupLike;

    /// An application with a menu bar, so the desktop starts at row 1 and a
    /// mistake in any owner's origin shows up in the popup anchor.
    fn app() -> Application {
        let mut app = Application::with_terminal(crate::test_util::test_terminal(80, 25));
        app.set_menu_bar(MenuBar::new(Rect::new(0, 0, 80, 1)));
        app
    }

    fn combo(id: u16) -> ComboBox {
        ComboBox::with_items(
            Rect::new(2, 2, 22, 3),
            id,
            vec!["one".into(), "two".into(), "three".into()],
        )
    }

    /// Screen rect of the 20-cell field, measured from where its drop arrow
    /// was drawn (the arrow owns the field's last cell).
    fn drawn_field(app: &Application) -> Rect {
        let (w, h) = app.terminal.size();
        for y in 0..h {
            for x in 0..w {
                if app.terminal.read_cell(x, y).unwrap().ch == '\u{25BC}' {
                    return Rect::new(x - 19, y, x + 1, y + 1);
                }
            }
        }
        panic!("the combo box arrow was not drawn");
    }

    #[test]
    fn the_popup_opens_under_the_field_inside_a_dialog() {
        let mut app = app();
        let mut dialog = Dialog::new(Rect::new(10, 5, 50, 15), "D");
        let c = combo(9701);
        let state = c.state();
        dialog.add(c);

        // F4 opens the list; Esc closes it; Esc ends the dialog.
        let tx = app.terminal.event_injector();
        tx.send(Event::keyboard(KB_F4)).unwrap();
        tx.send(Event::keyboard(KB_ESC)).unwrap();
        tx.send(Event::keyboard(KB_ESC)).unwrap();
        assert_eq!(dialog.execute(&mut app), CM_CANCEL);

        // The last frame drawn before the dialog ended shows the field.
        let field = drawn_field(&app);
        assert!(
            field.a.x > 10 && field.a.y > 5,
            "drawn inside the dialog: {field:?}"
        );
        assert_eq!(
            state.borrow().field,
            field,
            "popup anchored on the field's screen rect"
        );
    }

    #[test]
    fn the_popup_opens_under_the_field_of_a_window_on_the_desktop() {
        let mut app = app();
        let mut dialog = Dialog::new(Rect::new(10, 5, 50, 15), "D");
        let c = combo(9702);
        let state = c.state();
        dialog.add(c);
        app.desktop.add(dialog);
        app.draw();
        let field = drawn_field(&app);

        let tx = app.terminal.event_injector();
        tx.send(Event::keyboard(KB_ESC)).unwrap(); // closes the popup
        let mut ev = Event::keyboard(KB_F4);
        app.handle_event(&mut ev);
        assert_eq!(state.borrow().field, field);

        // A click anywhere on the field anchors on the field, not the pointer.
        state.borrow_mut().field = Rect::new(0, 0, 20, 1);
        tx.send(Event::keyboard(KB_ESC)).unwrap();
        let at = Point::new(field.a.x + 5, field.a.y);
        let mut down = Event::mouse(EventType::MouseDown, at, MB_LEFT_BUTTON, false);
        app.handle_event(&mut down);
        assert_eq!(state.borrow().field, field);
    }
}

#[cfg(test)]
mod window_key_tests {
    //! Borland's default window keys (`TProgram`'s status line): Alt+F3 close,
    //! F5 zoom, Ctrl+F5 move/resize, F6 / Shift+F6 next / previous window.
    //! They are fallbacks: a focused view that wants the key keeps it.
    use super::*;
    use crate::core::event::{KB_ALT_F3, KB_F5, KB_F6, KB_RIGHT};
    use crate::core::keys::KeyModifiers;
    use crate::views::group::{Group, GroupLike};
    use crate::views::window::Window;

    fn app() -> Application {
        let mut app = Application::with_terminal(crate::test_util::test_terminal(80, 25));
        app.set_menu_bar(MenuBar::new(Rect::new(0, 0, 80, 1)));
        app
    }

    fn key(code: u16, modifiers: KeyModifiers) -> Event {
        let mut e = Event::keyboard(code);
        e.key_modifiers = modifiers;
        e
    }

    fn press(app: &mut Application, code: u16, modifiers: KeyModifiers) {
        let mut e = key(code, modifiers);
        app.handle_event(&mut e);
    }

    /// Three windows, `c` added last and so on top and focused.
    fn three_windows(app: &mut Application) -> [ViewId; 3] {
        [
            app.desktop.add(Window::new(Rect::new(1, 1, 30, 10), "A")),
            app.desktop.add(Window::new(Rect::new(5, 3, 35, 12), "B")),
            app.desktop.add(Window::new(Rect::new(9, 5, 39, 14), "C")),
        ]
    }

    fn is_closed(app: &Application, id: ViewId) -> bool {
        app.desktop
            .child_by_id(id)
            .unwrap()
            .state()
            .contains(State::CLOSED)
    }

    #[test]
    fn alt_f3_closes_the_focused_window_and_not_the_other() {
        let mut app = app();
        let back = app
            .desktop
            .add(Window::new(Rect::new(1, 1, 30, 10), "Back"));
        let front = app
            .desktop
            .add(Window::new(Rect::new(5, 3, 35, 12), "Front"));
        assert!(
            command_set::command_enabled(CM_CLOSE),
            "selecting a window enabled CM_CLOSE"
        );
        press(&mut app, KB_ALT_F3, KeyModifiers::empty());
        assert!(is_closed(&app, front), "the focused window closes");
        assert!(!is_closed(&app, back), "the other window stays open");
    }

    #[test]
    fn alt_f3_cancels_a_modal_dialog() {
        use crate::views::button::Button;
        use crate::views::dialog::Dialog;
        let mut app = app();
        let mut dialog = Dialog::new(Rect::new(5, 5, 40, 12), "t");
        dialog.add(Button::new(
            Rect::new(2, 2, 12, 4),
            "OK",
            crate::core::command::CM_OK,
            true,
        ));
        dialog.set_state(dialog.state() | State::MODAL);
        app.terminal
            .event_injector()
            .send(Event::keyboard(KB_ALT_F3))
            .unwrap();
        // Give up after a few idle ticks so a missing binding fails instead
        // of spinning forever.
        let mut ticks = 0;
        let result = app.execute_modal(&mut dialog, |_, _| {
            ticks += 1;
            if ticks > 5 {
                ModalTick::End(9999)
            } else {
                ModalTick::Continue
            }
        });
        assert_eq!(result, CM_CANCEL);
        assert!(
            !command_set::command_enabled(CM_CLOSE),
            "the command set is restored after the modal loop"
        );
    }

    #[test]
    fn f6_and_shift_f6_cycle_the_focused_window() {
        let mut app = app();
        let [_a, b, c] = three_windows(&mut app);
        assert_eq!(app.desktop.top_view_id(), Some(c));
        press(&mut app, KB_F6, KeyModifiers::empty());
        assert_eq!(app.desktop.top_view_id(), Some(b), "F6: next window");
        press(&mut app, KB_F6, KeyModifiers::SHIFT);
        assert_eq!(app.desktop.top_view_id(), Some(c), "Shift+F6: previous");
    }

    #[test]
    fn f6_in_a_tabbed_pane_switches_its_page_and_not_the_window() {
        use crate::views::tabbed_pane::TabbedPane;
        let mut app = app();
        app.desktop
            .add(Window::new(Rect::new(1, 1, 30, 10), "Back"));
        let mut window = Window::new(Rect::new(2, 2, 60, 20), "Tabs");
        let mut pane = TabbedPane::new(Rect::new(1, 1, 50, 15));
        pane.add_page("One", Group::new(pane.page_area()));
        pane.add_page("Two", Group::new(pane.page_area()));
        let pane = window.add_typed(pane);
        let front = app.desktop.add(window);
        let window_handle = crate::views::handle::Handle::<Window>::from_id(front);

        let active = |app: &Application| {
            app.desktop
                .get(window_handle)
                .unwrap()
                .get(pane)
                .unwrap()
                .active()
        };
        press(&mut app, KB_F6, KeyModifiers::empty());
        assert_eq!(active(&app), 1, "F6 switched the page");
        assert_eq!(app.desktop.top_view_id(), Some(front), "no window cycle");
        press(&mut app, KB_F6, KeyModifiers::SHIFT);
        assert_eq!(active(&app), 0, "Shift+F6 switched back");
        assert_eq!(app.desktop.top_view_id(), Some(front), "no window cycle");
    }

    #[test]
    fn f5_zooms_and_ctrl_f5_moves_the_focused_window() {
        let mut app = app();
        let [_, _, c] = three_windows(&mut app);
        let bounds = |app: &Application| app.desktop.child_by_id(c).unwrap().bounds();
        let before = bounds(&app);
        press(&mut app, KB_F5, KeyModifiers::empty());
        let desktop = app.desktop.bounds();
        assert_eq!(
            bounds(&app),
            Rect::new(0, 0, desktop.width(), desktop.height()),
            "F5 zoomed the window to the desktop"
        );
        press(&mut app, KB_F5, KeyModifiers::empty());
        assert_eq!(bounds(&app), before, "F5 again restored it");

        // Ctrl+F5 enters keyboard move mode: Right moves the window.
        press(&mut app, KB_F5, KeyModifiers::CONTROL);
        press(&mut app, KB_RIGHT, KeyModifiers::empty());
        assert_eq!(bounds(&app).a.x, before.a.x + 1, "moved one column");
    }

    /// A focused view that takes F5 for itself.
    struct WantsF5 {
        core: crate::views::view::ViewCore,
        state: crate::core::state::StateFlags,
        seen: std::rc::Rc<std::cell::Cell<u32>>,
    }

    impl View for WantsF5 {
        fn core(&self) -> &crate::views::view::ViewCore {
            &self.core
        }
        fn core_mut(&mut self) -> &mut crate::views::view::ViewCore {
            &mut self.core
        }
        fn can_focus(&self) -> bool {
            true
        }
        fn state(&self) -> crate::core::state::StateFlags {
            self.state
        }
        fn set_state(&mut self, state: crate::core::state::StateFlags) {
            self.state = state;
        }
        fn draw(&mut self, _terminal: &mut Terminal) {}
        fn handle_event(&mut self, event: &mut Event) {
            if event.what == EventType::Keyboard && event.key_code == KB_F5 {
                self.seen.set(self.seen.get() + 1);
                event.clear();
            }
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

    #[test]
    fn a_view_that_consumes_f5_keeps_it() {
        let mut app = app();
        let seen = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut window = Window::new(Rect::new(5, 3, 35, 12), "W");
        window.add(WantsF5 {
            core: crate::views::view::ViewCore {
                bounds: Rect::new(1, 1, 10, 2),
                ..Default::default()
            },
            state: State::empty(),
            seen: std::rc::Rc::clone(&seen),
        });
        let id = app.desktop.add(window);
        let before = app.desktop.child_by_id(id).unwrap().bounds();
        press(&mut app, KB_F5, KeyModifiers::empty());
        assert_eq!(seen.get(), 1, "the view got F5");
        assert_eq!(
            app.desktop.child_by_id(id).unwrap().bounds(),
            before,
            "and the window did not zoom"
        );
    }

    // ---- fix round 1: the command set, and modal windows on the desktop ----

    use crate::core::command::{CM_CLOSE, CM_NEXT, CM_PREV, CM_RESIZE, CM_ZOOM};
    const WINDOW_COMMANDS: [CommandId; 5] = [CM_CLOSE, CM_ZOOM, CM_RESIZE, CM_NEXT, CM_PREV];

    #[test]
    fn a_modal_window_on_the_desktop_is_not_cycled_away_but_zooms() {
        let mut app = app();
        let back = app
            .desktop
            .add(Window::new(Rect::new(1, 1, 30, 10), "Back"));
        let mut modal = Window::new(Rect::new(5, 3, 35, 12), "Modal");
        modal.set_state(modal.state() | State::MODAL);
        let id = app.desktop.add(modal);
        let before = app.desktop.child_by_id(id).unwrap().bounds();
        let back_before = app.desktop.child_by_id(back).unwrap().bounds();
        for mods in [KeyModifiers::empty(), KeyModifiers::SHIFT] {
            press(&mut app, KB_F6, mods);
            assert_eq!(app.desktop.top_view_id(), Some(id), "F6 {mods:?}");
            let modal = app.desktop.child_by_id(id).unwrap();
            assert!(modal.state().contains(State::ACTIVE), "still selected");
            assert_eq!(modal.bounds(), before);
        }
        press(&mut app, KB_F5, KeyModifiers::empty());
        assert_eq!(app.desktop.top_view_id(), Some(id));
        let desktop = app.desktop.bounds();
        assert_eq!(
            app.desktop.child_by_id(id).unwrap().bounds(),
            Rect::new(0, 0, desktop.width(), desktop.height()),
            "F5 zooms the modal window itself"
        );
        assert_eq!(
            app.desktop.child_by_id(back).unwrap().bounds(),
            back_before,
            "nothing behind it changes"
        );
    }

    #[test]
    fn a_modal_window_zooms_from_its_zoom_icon() {
        use crate::core::event::MB_LEFT_BUTTON;
        let mut app = app();
        let mut modal = Window::new(Rect::new(5, 3, 35, 12), "Modal");
        modal.set_state(modal.state() | State::MODAL);
        let id = app.desktop.add(modal);
        // The zoom icon sits at columns width-5..width-3 of the top frame
        // row; the desktop starts below the menu bar.
        let origin = app.desktop.bounds().a;
        let at = crate::core::geometry::Point::new(origin.x + 5 + 30 - 4, origin.y + 3);
        let mut down = Event::mouse(EventType::MouseDown, at, MB_LEFT_BUTTON, false);
        app.handle_event(&mut down);
        let mut up = Event::mouse(EventType::MouseUp, at, MB_LEFT_BUTTON, false);
        app.handle_event(&mut up);
        let desktop = app.desktop.bounds();
        assert_eq!(
            app.desktop.child_by_id(id).unwrap().bounds(),
            Rect::new(0, 0, desktop.width(), desktop.height())
        );
    }

    /// Those of `ids` whose window has `State::ACTIVE`.
    fn active_windows(app: &Application, ids: &[ViewId]) -> Vec<ViewId> {
        ids.iter()
            .copied()
            .filter(|&id| {
                app.desktop
                    .child_by_id(id)
                    .unwrap()
                    .state()
                    .contains(State::ACTIVE)
            })
            .collect()
    }

    /// After a switch, Alt+F3 closes the window that is on top, and exactly
    /// that window is the active one.
    fn alt_f3_closes_the_top(app: &mut Application, ids: &[ViewId]) {
        let top = app.desktop.top_view_id().unwrap();
        assert_eq!(
            active_windows(app, ids),
            vec![top],
            "exactly the top is active"
        );
        press(app, KB_ALT_F3, KeyModifiers::empty());
        for &id in ids {
            assert_eq!(is_closed(app, id), id == top, "{id:?} (top {top:?})");
        }
    }

    #[test]
    fn after_f6_alt_f3_closes_the_new_top_window() {
        let mut app = app();
        let ids = three_windows(&mut app);
        press(&mut app, KB_F6, KeyModifiers::empty());
        assert_eq!(app.desktop.top_view_id(), Some(ids[1]));
        alt_f3_closes_the_top(&mut app, &ids);
    }

    #[test]
    fn after_shift_f6_alt_f3_closes_the_new_top_window() {
        let mut app = app();
        let ids = three_windows(&mut app);
        press(&mut app, KB_F6, KeyModifiers::SHIFT);
        assert_eq!(app.desktop.top_view_id(), Some(ids[0]));
        alt_f3_closes_the_top(&mut app, &ids);
    }

    #[test]
    fn after_alt_number_alt_f3_closes_the_chosen_window() {
        use crate::core::event::KB_ALT_1;
        let mut app = app();
        let mut ids = Vec::new();
        for (n, x) in [(1u8, 1), (2, 5), (3, 9)] {
            let mut w = Window::new(Rect::new(x, x, x + 30, x + 9), "N");
            w.set_number(n);
            ids.push(app.desktop.add(w));
        }
        press(&mut app, KB_ALT_1, KeyModifiers::empty());
        assert_eq!(app.desktop.top_view_id(), Some(ids[0]));
        alt_f3_closes_the_top(&mut app, &ids);
    }

    #[test]
    fn bring_to_front_selects_the_window() {
        let mut app = app();
        let ids = three_windows(&mut app);
        app.desktop.bring_to_front(ids[0]);
        alt_f3_closes_the_top(&mut app, &ids);
    }

    #[test]
    fn clicking_a_window_still_selects_it() {
        use crate::core::event::MB_LEFT_BUTTON;
        let mut app = app();
        let ids = three_windows(&mut app);
        // Window A's top-left interior cell, clear of B and C.
        let origin = app.desktop.bounds().a;
        let at = crate::core::geometry::Point::new(origin.x + 2, origin.y + 2);
        let mut down = Event::mouse(EventType::MouseDown, at, MB_LEFT_BUTTON, false);
        app.handle_event(&mut down);
        let mut up = Event::mouse(EventType::MouseUp, at, MB_LEFT_BUTTON, false);
        app.handle_event(&mut up);
        assert_eq!(app.desktop.top_view_id(), Some(ids[0]));
        alt_f3_closes_the_top(&mut app, &ids);
    }

    #[test]
    fn window_commands_follow_a_switch() {
        let mut app = app();
        let mut fixed = Window::new(Rect::new(1, 1, 30, 10), "Fixed");
        fixed.set_zoomable(false);
        app.desktop.add(fixed);
        app.desktop
            .add(Window::new(Rect::new(5, 3, 35, 12), "Zoomable"));
        assert!(command_set::command_enabled(CM_ZOOM));
        press(&mut app, KB_F6, KeyModifiers::empty());
        assert!(
            !command_set::command_enabled(CM_ZOOM),
            "the non-zoomable window is selected now"
        );
        press(&mut app, KB_F6, KeyModifiers::empty());
        assert!(command_set::command_enabled(CM_ZOOM));
    }

    #[test]
    fn window_commands_follow_the_selected_window() {
        let mut app = app();
        for cmd in WINDOW_COMMANDS {
            assert!(!command_set::command_enabled(cmd), "{cmd} off at startup");
        }
        let id = app.desktop.add(Window::new(Rect::new(1, 1, 30, 10), "W"));
        for cmd in WINDOW_COMMANDS {
            assert!(command_set::command_enabled(cmd), "{cmd} on once selected");
        }
        press(&mut app, KB_ALT_F3, KeyModifiers::empty());
        assert!(is_closed(&app, id));
        app.desktop.remove_closed_windows();
        for cmd in WINDOW_COMMANDS {
            assert!(
                !command_set::command_enabled(cmd),
                "{cmd} off again once the last window is gone"
            );
        }
    }

    #[test]
    fn a_disabled_cm_close_turns_alt_f3_off() {
        let mut app = app();
        let id = app.desktop.add(Window::new(Rect::new(1, 1, 30, 10), "W"));
        app.disable_command(CM_CLOSE);
        press(&mut app, KB_ALT_F3, KeyModifiers::empty());
        assert!(!is_closed(&app, id), "Alt+F3 honours the command set");
    }

    #[test]
    fn f5_does_not_zoom_a_window_that_cannot_zoom() {
        let mut app = app();
        let mut window = Window::new(Rect::new(1, 1, 30, 10), "Fixed");
        window.set_zoomable(false);
        let id = app.desktop.add(window);
        assert!(!command_set::command_enabled(CM_ZOOM));
        let before = app.desktop.child_by_id(id).unwrap().bounds();
        press(&mut app, KB_F5, KeyModifiers::empty());
        assert_eq!(app.desktop.child_by_id(id).unwrap().bounds(), before);
    }

    #[test]
    fn a_status_line_alt_f3_item_closes_the_window() {
        use crate::core::status_data::StatusItemBuilder;
        let mut app = app();
        app.set_status_line(StatusLine::new(
            Rect::new(0, 24, 80, 25),
            vec![
                StatusItemBuilder::new()
                    .text("~Alt+F3~ Close")
                    .key("Alt+F3")
                    .command(CM_CLOSE)
                    .build(),
            ],
        ));
        let id = app.desktop.add(Window::new(Rect::new(1, 1, 30, 10), "W"));
        press(&mut app, KB_ALT_F3, KeyModifiers::empty());
        assert!(
            is_closed(&app, id),
            "the status line's CM_CLOSE reaches the window"
        );
    }

    #[test]
    fn alt_number_does_not_switch_windows_under_a_modal_one() {
        use crate::core::event::KB_ALT_1;
        let mut app = app();
        let mut numbered = Window::new(Rect::new(1, 1, 30, 10), "One");
        numbered.set_number(1);
        app.desktop.add(numbered);
        let mut modal = Window::new(Rect::new(5, 3, 35, 12), "Modal");
        modal.set_state(modal.state() | State::MODAL);
        let id = app.desktop.add(modal);
        press(&mut app, KB_ALT_1, KeyModifiers::empty());
        assert_eq!(
            app.desktop.top_view_id(),
            Some(id),
            "the modal window stays on top"
        );
        assert!(
            app.desktop
                .child_by_id(id)
                .unwrap()
                .state()
                .contains(State::ACTIVE),
            "and stays selected"
        );
    }
}

#[cfg(test)]
mod cursor_tests {
    //! The terminal cursor follows the focused text control: the desktop
    //! hands `update_cursor` down to its windows.
    use super::*;
    use crate::views::button::Button;
    use crate::views::dialog::Dialog;
    use crate::views::group::GroupLike;
    use crate::views::input_line::InputLine;
    use crate::views::window::Window;
    use std::sync::{Arc, Mutex};

    type Cursor = Arc<Mutex<Option<(u16, u16)>>>;

    /// An application with a menu bar, so the desktop starts at row 1, and
    /// the handle its backend records the cursor through.
    fn app() -> (Application, Cursor) {
        let backend = crate::test_util::TestBackend::new(80, 25);
        let cursor = backend.cursor_handle();
        let terminal = Terminal::with_backend(Box::new(backend)).unwrap();
        let mut app = Application::with_terminal(terminal);
        app.set_menu_bar(MenuBar::new(Rect::new(0, 0, 80, 1)));
        (app, cursor)
    }

    fn input(text: &str) -> InputLine {
        let mut line = InputLine::new(Rect::new(2, 2, 22, 3), 40);
        line.set_text(text);
        line
    }

    /// Where `text` was drawn on screen.
    fn drawn_at(app: &Application, text: &str) -> (u16, u16) {
        let (w, h) = app.terminal.size();
        for y in 0..h {
            let row: String = (0..w)
                .map(|x| app.terminal.read_cell(x, y).unwrap().ch)
                .collect();
            if let Some(i) = row.find(text) {
                return (row[..i].chars().count() as u16, y as u16);
            }
        }
        panic!("{text:?} was not drawn");
    }

    /// What the terminal asked of the backend, in order.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Op {
        Write,
        Show(u16, u16),
        Hide,
        Flush,
    }

    struct LogBackend {
        log: Arc<Mutex<Vec<Op>>>,
    }

    impl crate::terminal::Backend for LogBackend {
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn init(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn cleanup(&mut self) -> std::io::Result<()> {
            Ok(())
        }
        fn size(&self) -> std::io::Result<(u16, u16)> {
            Ok((80, 25))
        }
        fn poll_event(&mut self, _: std::time::Duration) -> std::io::Result<Option<Event>> {
            Ok(None)
        }
        fn write_raw(&mut self, _: &[u8]) -> std::io::Result<()> {
            self.log.lock().unwrap().push(Op::Write);
            Ok(())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.log.lock().unwrap().push(Op::Flush);
            Ok(())
        }
        fn show_cursor(&mut self, x: u16, y: u16) -> std::io::Result<()> {
            self.log.lock().unwrap().push(Op::Show(x, y));
            Ok(())
        }
        fn hide_cursor(&mut self) -> std::io::Result<()> {
            self.log.lock().unwrap().push(Op::Hide);
            Ok(())
        }
    }

    /// An application over a [`LogBackend`] with a window holding a focused
    /// input line "hello", drawn and flushed once.
    fn logged_app() -> (Application, Arc<Mutex<Vec<Op>>>, (u16, u16)) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let terminal = Terminal::with_backend(Box::new(LogBackend {
            log: Arc::clone(&log),
        }))
        .unwrap();
        let mut app = Application::with_terminal(terminal);
        app.set_menu_bar(MenuBar::new(Rect::new(0, 0, 80, 1)));
        let mut window = Window::new(Rect::new(10, 5, 50, 15), "W");
        window.add(Box::new(input("hello")));
        app.desktop.add(window);
        app.draw();
        app.terminal.flush().unwrap();
        let (x, y) = drawn_at(&app, "hello");
        (app, log, (x + 5, y))
    }

    /// Every flush that wrote cells hid the cursor before the first write
    /// and showed it at `at` after the last one.
    fn assert_cursor_put_back(log: &[Op], at: (u16, u16)) {
        let mut start = 0;
        let mut checked = 0;
        for (i, op) in log.iter().enumerate() {
            if *op != Op::Flush {
                continue;
            }
            let segment = &log[start..i];
            start = i + 1;
            let Some(first) = segment.iter().position(|o| *o == Op::Write) else {
                continue;
            };
            let last = segment.iter().rposition(|o| *o == Op::Write).unwrap();
            assert_eq!(
                segment[..first]
                    .iter()
                    .rev()
                    .find(|o| matches!(o, Op::Show(..) | Op::Hide)),
                Some(&Op::Hide),
                "the cursor is hidden while cells are written: {log:?}"
            );
            assert_eq!(
                segment[last..]
                    .iter()
                    .rev()
                    .find(|o| matches!(o, Op::Show(..) | Op::Hide)),
                Some(&Op::Show(at.0, at.1)),
                "the cursor is put back after the cells: {log:?}"
            );
            checked += 1;
        }
        assert!(checked > 0, "no flush wrote cells: {log:?}");
    }

    #[test]
    fn the_cursor_is_put_back_after_a_flush_writes_cells() {
        let (mut app, log, at) = logged_app();
        assert_cursor_put_back(&log.lock().unwrap(), at);

        // A later frame that changes cells: typing (which replaces the
        // selected text) moves the cursor to just after the "!".
        log.lock().unwrap().clear();
        let mut key = Event::keyboard('!' as u16);
        app.handle_event(&mut key);
        app.draw();
        app.terminal.flush().unwrap();
        let (x, y) = drawn_at(&app, "!");
        assert_cursor_put_back(&log.lock().unwrap(), (x + 1, y));
    }

    #[test]
    fn a_flush_with_no_changed_cells_leaves_the_cursor_alone() {
        let (mut app, log, _) = logged_app();
        log.lock().unwrap().clear();
        app.terminal.flush().unwrap();
        assert_eq!(*log.lock().unwrap(), vec![Op::Flush]);
    }

    /// An overlay that draws a different digit each time.
    struct Ticker {
        core: crate::views::view::ViewCore,
        n: u8,
    }

    impl View for Ticker {
        fn core(&self) -> &crate::views::view::ViewCore {
            &self.core
        }
        fn core_mut(&mut self) -> &mut crate::views::view::ViewCore {
            &mut self.core
        }
        fn draw(&mut self, terminal: &mut Terminal) {
            self.n = (self.n + 1) % 10;
            let attr = crate::core::palette::Attr::from_u8(0x1F);
            terminal.write_cell(
                78,
                0,
                crate::core::draw::Cell::new((b'0' + self.n) as char, attr),
            );
        }
        fn handle_event(&mut self, _: &mut Event) {}
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

    #[test]
    fn an_idle_overlay_redraw_puts_the_cursor_back() {
        let (mut app, log, at) = logged_app();
        app.add_overlay_widget(Ticker {
            core: crate::views::view::ViewCore {
                bounds: Rect::new(78, 0, 79, 1),
                ..Default::default()
            },
            n: 0,
        });
        app.needs_redraw = false;
        log.lock().unwrap().clear();
        app.step(&mut (), None);
        assert_cursor_put_back(&log.lock().unwrap(), at);
    }

    #[test]
    fn a_focused_input_line_shows_the_cursor() {
        let (mut app, cursor) = app();
        let mut window = Window::new(Rect::new(10, 5, 50, 15), "W");
        window.add(Box::new(input("hello")));
        app.desktop.add(window);
        app.draw();
        app.terminal.flush().unwrap();

        let (x, y) = drawn_at(&app, "hello");
        assert!(x > 10 && y > 5, "drawn inside the window: {:?}", (x, y));
        assert_eq!(*cursor.lock().unwrap(), Some((x + 5, y)));
    }

    #[test]
    fn a_focused_input_line_on_a_tab_page_shows_the_cursor() {
        use crate::views::group::Group;
        use crate::views::tabbed_pane::TabbedPane;
        let (mut app, cursor) = app();
        let mut pane = TabbedPane::new(Rect::new(1, 1, 37, 9));
        let mut page = Group::new(pane.page_area());
        page.add(Box::new(input("tabbed")));
        pane.add_page("~O~ne", page);
        let mut window = Window::new(Rect::new(10, 5, 50, 17), "W");
        window.add(Box::new(pane));
        app.desktop.add(window);
        app.draw();
        app.terminal.flush().unwrap();

        let (x, y) = drawn_at(&app, "tabbed");
        assert_eq!(*cursor.lock().unwrap(), Some((x + 6, y)));
    }

    #[test]
    fn a_focused_input_line_in_a_split_pane_shows_the_cursor() {
        use crate::core::event::MB_LEFT_BUTTON;
        use crate::core::geometry::Point;
        use crate::views::group::Group;
        use crate::views::split_pane::{Orientation, SplitPane};
        let (mut app, cursor) = app();
        let mut split = SplitPane::new(Rect::new(1, 1, 37, 9), Orientation::Vertical, 17);
        let mut left = Group::new(split.first_area());
        left.add(Box::new(InputLine::new(Rect::new(1, 1, 15, 2), 20)));
        left.child_at_mut(0)
            .as_any_mut()
            .downcast_mut::<InputLine>()
            .unwrap()
            .set_text("left");
        let mut right = Group::new(split.second_area());
        right.add(Box::new(InputLine::new(Rect::new(1, 3, 15, 4), 20)));
        right
            .child_at_mut(0)
            .as_any_mut()
            .downcast_mut::<InputLine>()
            .unwrap()
            .set_text("right");
        split.set_panes(left, right);
        let mut window = Window::new(Rect::new(10, 5, 50, 17), "W");
        window.add(Box::new(split));
        app.desktop.add(window);
        app.draw();
        app.terminal.flush().unwrap();

        // The first half holds the focus to begin with.
        let (x, y) = drawn_at(&app, "left");
        assert_eq!(*cursor.lock().unwrap(), Some((x + 4, y)));

        // A click in the second half moves the focus, and the cursor, there.
        let (rx, ry) = drawn_at(&app, "right");
        let at = Point::new(rx as i16 + 1, ry as i16);
        let mut down = Event::mouse(EventType::MouseDown, at, MB_LEFT_BUTTON, false);
        app.handle_event(&mut down);
        let mut up = Event::mouse(EventType::MouseUp, at, 0, false);
        app.handle_event(&mut up);
        app.draw();
        app.terminal.flush().unwrap();
        let (rx, ry) = drawn_at(&app, "right");
        let shown = cursor.lock().unwrap().expect("the cursor is shown");
        assert_eq!(shown.1, ry, "cursor on the right input line: {shown:?}");
        assert!(
            (rx..=rx + 5).contains(&shown.0),
            "cursor within the right input line: {shown:?}"
        );
    }

    #[test]
    fn an_open_menu_hides_the_cursor() {
        use crate::core::event::{KB_ESC, KB_F10};
        use crate::core::menu_data::MenuBuilder;
        use crate::views::menu_bar::SubMenu;
        let (mut app, cursor) = app();
        let mut bar = MenuBar::new(Rect::new(0, 0, 80, 1));
        bar.add_submenu(SubMenu::new(
            "~F~ile",
            MenuBuilder::new().item("~O~pen", 100).build(),
        ));
        app.set_menu_bar(bar);
        let mut window = Window::new(Rect::new(10, 5, 50, 15), "W");
        window.add(Box::new(input("hello")));
        app.desktop.add(window);
        app.draw();
        app.terminal.flush().unwrap();
        let (x, y) = drawn_at(&app, "hello");
        assert_eq!(*cursor.lock().unwrap(), Some((x + 5, y)));

        let mut f10 = Event::keyboard(KB_F10);
        app.handle_event(&mut f10);
        assert!(app.menu_bar.as_ref().unwrap().is_open());
        app.draw();
        app.terminal.flush().unwrap();
        assert_eq!(*cursor.lock().unwrap(), None, "no cursor under the menu");

        let mut esc = Event::keyboard(KB_ESC);
        app.handle_event(&mut esc);
        assert!(!app.menu_bar.as_ref().unwrap().is_open());
        app.draw();
        app.terminal.flush().unwrap();
        assert_eq!(
            *cursor.lock().unwrap(),
            Some((x + 5, y)),
            "back once it closes"
        );
    }

    #[test]
    fn no_focused_text_control_hides_the_cursor() {
        let (mut app, cursor) = app();
        app.terminal.show_cursor(1, 1).unwrap();
        app.terminal.flush().unwrap();
        assert_eq!(*cursor.lock().unwrap(), Some((1, 1)));
        let mut window = Window::new(Rect::new(10, 5, 50, 15), "W");
        window.add(Box::new(Button::new(
            Rect::new(2, 2, 12, 4),
            "OK",
            crate::core::command::CM_OK,
            true,
        )));
        app.desktop.add(window);
        app.draw();
        app.terminal.flush().unwrap();
        assert_eq!(*cursor.lock().unwrap(), None);
    }

    #[test]
    fn a_modal_dialog_on_the_desktop_owns_the_cursor() {
        let (mut app, cursor) = app();
        let mut back = Window::new(Rect::new(1, 1, 40, 12), "Back");
        back.add(Box::new(input("behind")));
        app.desktop.add(back);
        let mut dialog = Dialog::new(Rect::new(30, 8, 70, 18), "Modal");
        dialog.add(input("front!"));
        dialog.set_state(dialog.state() | State::MODAL);
        app.desktop.add(dialog);
        app.draw();
        app.terminal.flush().unwrap();

        let (x, y) = drawn_at(&app, "front!");
        assert_eq!(*cursor.lock().unwrap(), Some((x + 6, y)));
    }

    #[test]
    fn execute_modal_puts_the_cursor_in_the_view_once() {
        let (mut app, cursor) = app();
        let mut dialog = Dialog::new(Rect::new(20, 6, 60, 16), "Run");
        dialog.add(input("abc"));
        // What `Dialog::execute` does before the loop.
        dialog.set_initial_focus();
        let result = app.execute_modal(&mut dialog, |_, _| ModalTick::End(CM_CANCEL));
        assert_eq!(result, CM_CANCEL);

        let (x, y) = drawn_at(&app, "abc");
        assert!(x > 20 && y > 6, "drawn inside the dialog: {:?}", (x, y));
        assert_eq!(*cursor.lock().unwrap(), Some((x + 3, y)));
    }

    /// Run a popup over a focused input line whose cursor is at `at`, with
    /// `key` queued to close it, and check that the cursor stayed hidden
    /// while the popup drew and comes back at `at` once it is gone.
    fn assert_hidden_under_popup<R>(key: u16, popup: impl FnOnce(&mut Terminal) -> R) -> R {
        let (mut app, log, at) = logged_app();
        log.lock().unwrap().clear();
        app.terminal
            .event_injector()
            .send(Event::keyboard(key))
            .unwrap();

        let result = popup(&mut app.terminal);
        let during = log.lock().unwrap().clone();
        assert!(
            during.contains(&Op::Write),
            "the popup drew something: {during:?}"
        );
        let last_flush = during.iter().rposition(|o| *o == Op::Flush).unwrap();
        assert_eq!(
            during[..last_flush]
                .iter()
                .rev()
                .find(|o| matches!(o, Op::Show(..) | Op::Hide)),
            Some(&Op::Hide),
            "the cursor is hidden under the popup: {during:?}"
        );
        assert!(
            !during[..last_flush]
                .iter()
                .any(|o| matches!(o, Op::Show(..))),
            "no cursor while the popup is up: {during:?}"
        );

        assert_eq!(app.terminal.cursor(), Some(at), "the cursor is remembered");
        app.terminal.flush().unwrap();
        let log = log.lock().unwrap();
        assert_eq!(
            log.iter()
                .rev()
                .find(|o| matches!(o, Op::Show(..) | Op::Hide)),
            Some(&Op::Show(at.0, at.1)),
            "the cursor is back at the input line: {log:?}"
        );
        result
    }

    fn menu_box() -> crate::views::menu_box::MenuBox {
        use crate::core::geometry::Point;
        use crate::core::menu_data::MenuBuilder;
        crate::views::menu_box::MenuBox::new(
            Point::new(30, 2),
            MenuBuilder::new().item("~O~pen", 100).build(),
        )
    }

    fn dropdown() -> crate::views::combo_box::DropdownWindow {
        use crate::views::combo_box::{ComboBox, DropdownWindow};
        let combo = ComboBox::with_items(
            Rect::new(30, 2, 50, 3),
            930,
            vec!["one".into(), "two".into()],
        );
        DropdownWindow::new(combo.state(), Rect::new(0, 0, 80, 25))
    }

    #[test]
    fn a_menu_box_hides_the_cursor_and_puts_it_back() {
        use crate::core::event::{KB_ENTER, KB_ESC};
        assert_eq!(
            assert_hidden_under_popup(KB_ESC, |t| menu_box().execute(t)),
            0
        );
        assert_eq!(
            assert_hidden_under_popup(KB_ENTER, |t| menu_box().execute(t)),
            100
        );
    }

    #[test]
    fn a_combo_dropdown_hides_the_cursor_and_puts_it_back() {
        use crate::core::event::{KB_ENTER, KB_ESC};
        assert_eq!(
            assert_hidden_under_popup(KB_ESC, |t| dropdown().execute(t)),
            None
        );
        assert!(assert_hidden_under_popup(KB_ENTER, |t| dropdown().execute(t)).is_some());
    }

    #[test]
    fn a_history_popup_hides_the_cursor_and_puts_it_back() {
        use crate::core::event::{KB_ENTER, KB_ESC};
        use crate::core::geometry::Point;
        use crate::core::history::HistoryManager;
        use crate::views::history_window::HistoryWindow;
        let _guard = crate::core::history::test_lock();
        HistoryManager::clear(931);
        HistoryManager::add(931, "earlier".to_string());
        let history = || HistoryWindow::new(Point::new(30, 2), 931, 30);
        assert_eq!(
            assert_hidden_under_popup(KB_ESC, |t| history().execute(t)),
            None
        );
        assert_eq!(
            assert_hidden_under_popup(KB_ENTER, |t| history().execute(t)),
            Some("earlier".to_string())
        );
    }

    #[test]
    fn write_raw_puts_a_shown_cursor_back() {
        let (mut app, log, at) = logged_app();
        log.lock().unwrap().clear();
        // Kitty graphics bytes that move the real cursor.
        app.terminal.write_raw(b"\x1b_Ga=T\x1b\\").unwrap();
        assert_eq!(
            *log.lock().unwrap(),
            vec![Op::Write, Op::Show(at.0, at.1), Op::Flush]
        );
    }

    #[test]
    fn write_raw_leaves_a_hidden_cursor_hidden() {
        let (mut app, log, _) = logged_app();
        app.terminal.hide_cursor().unwrap();
        app.terminal.flush().unwrap();
        log.lock().unwrap().clear();
        app.terminal.write_raw(b"\x1b_Ga=d\x1b\\").unwrap();
        assert_eq!(*log.lock().unwrap(), vec![Op::Write, Op::Flush]);
    }

    /// Type `ch` into the focused input line and draw and flush the frame.
    fn type_frame(app: &mut Application, ch: char) {
        let mut key = Event::keyboard(ch as u16);
        app.handle_event(&mut key);
        app.draw();
        app.terminal.flush().unwrap();
    }

    #[test]
    fn a_typing_frame_shows_the_cursor_once_after_the_cells() {
        let (mut app, log, _) = logged_app();
        log.lock().unwrap().clear();
        type_frame(&mut app, '!');
        let (x, y) = drawn_at(&app, "!");
        assert_eq!(
            *log.lock().unwrap(),
            vec![Op::Hide, Op::Write, Op::Show(x + 1, y), Op::Flush]
        );
    }

    #[test]
    fn a_frame_where_only_the_cursor_moved_just_moves_it() {
        use crate::core::event::KB_LEFT;
        let (mut app, log, _) = logged_app();
        type_frame(&mut app, '!');
        let (x, y) = drawn_at(&app, "!");
        log.lock().unwrap().clear();

        let mut left = Event::keyboard(KB_LEFT);
        app.handle_event(&mut left);
        app.draw();
        app.terminal.flush().unwrap();
        assert_eq!(*log.lock().unwrap(), vec![Op::Show(x, y), Op::Flush]);
    }

    #[test]
    fn a_frame_with_no_change_emits_only_the_flush() {
        let (mut app, log, _) = logged_app();
        log.lock().unwrap().clear();
        app.draw();
        app.terminal.flush().unwrap();
        assert_eq!(*log.lock().unwrap(), vec![Op::Flush]);
    }

    #[test]
    fn the_cursor_reaches_the_backend_only_on_flush() {
        let (mut app, log, _) = logged_app();
        log.lock().unwrap().clear();
        app.terminal.show_cursor(1, 1).unwrap();
        app.terminal.hide_cursor().unwrap();
        app.terminal.show_cursor(2, 2).unwrap();
        assert_eq!(*log.lock().unwrap(), vec![], "nothing before the flush");
        assert_eq!(app.terminal.cursor(), Some((2, 2)));
        app.terminal.flush().unwrap();
        assert_eq!(*log.lock().unwrap(), vec![Op::Show(2, 2), Op::Flush]);

        log.lock().unwrap().clear();
        app.terminal.hide_cursor().unwrap();
        app.terminal.flush().unwrap();
        assert_eq!(*log.lock().unwrap(), vec![Op::Hide, Op::Flush]);
    }
}
