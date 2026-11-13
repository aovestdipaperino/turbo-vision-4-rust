// (C) 2025 - Enzo Lombardi

//! Turbo Vision - A modern Text User Interface (TUI) framework for Rust.
//!
//! Turbo Vision is a Rust port of the classic Borland Turbo Vision framework,
//! providing a powerful and flexible system for building terminal-based
//! applications with a rich set of widgets and an event-driven architecture.
//!
//! # Features
//!
//! - **Event-driven architecture** - Handle keyboard, mouse, and custom events
//! - **Flexible view hierarchy** - Compose UIs from reusable widget components
//! - **Built-in widgets** - Windows, dialogs, buttons, input fields, lists, menus
//! - **Syntax highlighting** - Extensible syntax highlighting system for editors
//! - **Command system** - Global command routing and enable/disable states
//! - **Clipboard support** - Copy/paste operations
//! - **History management** - Input history for text fields
//! - **Double-buffered rendering** - Flicker-free terminal updates
//! - **Mouse support** - Full mouse capture and tracking
//! - **Borland TV compatibility** - Familiar API for Turbo Vision users
//!
//! # Quick Start
//!
//! The application owns the event loop: put your state in a struct that
//! implements [`AppHandler`](app::AppHandler), react to commands there, and
//! call [`run_with`](app::Application::run_with).
//!
//! ```rust,no_run
//! use turbo_vision::prelude::*;
//! use turbo_vision::views::button::Button;
//! use turbo_vision::views::msgbox::message_box_ok;
//! use turbo_vision::views::window::Window;
//!
//! // Your own commands start at CM_USER.
//! const CM_HELLO: CommandId = CM_USER;
//!
//! struct App;
//!
//! impl AppHandler for App {
//!     fn handle_command(&mut self, app: &mut Application, command: CommandId, _: &Event) -> bool {
//!         if command == CM_HELLO {
//!             message_box_ok(app, "Hello!");
//!             return true;
//!         }
//!         false
//!     }
//! }
//!
//! fn main() -> turbo_vision::core::error::Result<()> {
//!     let mut app = Application::new()?;
//!
//!     // A window on the desktop with a button; the button sends CM_HELLO.
//!     let mut window = Window::new(Rect::new(10, 5, 50, 15), "My First Window");
//!     window.add(Button::new(Rect::new(13, 3, 25, 5), "~H~ello", CM_HELLO, true));
//!     app.desktop.add(window);
//!
//!     app.run_with(&mut App); // Alt+X quits
//!     Ok(())
//! }
//! ```
//!
//! For the Turbo Vision way of working, a component index and recipes, read
//! `AGENTS.md` in the repository.
//!
//! # Architecture
//!
//! The framework is organized into several key modules:
//!
//! ## Core Modules
//!
//! - **[`core`]** - Fundamental types and utilities
//!   - [`geometry`](core::geometry) - Point, Rect for positioning
//!   - [`event`](core::event) - Event handling and keyboard/mouse input
//!   - [`command`](core::command) - Command IDs and routing
//!   - [`error`](core::error) - Error types and Result alias
//!   - [`palette`](core::palette) - Color management
//!
//! - **[`views`]** - Built-in widgets and view components
//!   - [`View`](views::View) - Base trait for all UI components
//!   - [`Window`](views::window::Window), [`Dialog`](views::dialog::Dialog) - Containers
//!   - [`Button`](views::button::Button), [`InputLine`](views::input_line::InputLine) - Controls
//!   - [`ComboBox`](views::combo_box::ComboBox), [`Spinner`](views::spinner::Spinner) - Choice and number fields
//!   - [`CheckBoxes`](views::cluster_group::CheckBoxes), [`RadioButtons`](views::cluster_group::RadioButtons) - Multi-item clusters
//!   - [`ListBox`](views::listbox::ListBox), [`Table`](views::table::Table) - Lists and grids
//!   - [`ProgressBar`](views::progress_bar::ProgressBar) - Determinate or marquee progress
//!   - [`TabbedPane`](views::tabbed_pane::TabbedPane), [`SplitPane`](views::split_pane::SplitPane) - Layout containers
//!   - [`Tooltip`](views::tooltip::Tooltip) - Hover hints
//!   - [`EditorWindow`](views::editor::EditorWindow) - Multi-line text editor
//!   - [`MenuBar`](views::menu_bar::MenuBar), [`StatusLine`](views::status_line::StatusLine) - Navigation
//!
//! - **[`app`]** - Application infrastructure
//!   - [`Application`](app::Application) - Main application coordinator
//!
//! - **[`terminal`]** - Terminal abstraction layer
//!   - [`Terminal`](terminal::Terminal) - Crossterm-based rendering backend
//!
//! ## Application Structure
//!
//! ```text
//! Application
//! ├── Terminal (crossterm backend)
//! ├── Desktop (window manager)
//! │   ├── Background
//! │   └── Windows/Dialogs
//! │       └── Child widgets (buttons, inputs, etc.)
//! ├── MenuBar (optional)
//! └── StatusLine (optional)
//! ```
//!
//! # Examples
//!
//! ## Creating a Dialog
//!
//! ```rust,no_run
//! use turbo_vision::prelude::*;
//! use turbo_vision::views::{dialog::Dialog, button::Button, static_text::StaticText};
//!
//! # fn create_dialog() -> Box<Dialog> {
//! let dialog_bounds = Rect::new(20, 8, 60, 16);
//! let mut dialog = Dialog::new_modal(dialog_bounds, "Confirmation");
//!
//! // Add message text
//! let text = StaticText::new(
//!     Rect::new(2, 2, 38, 3),
//!     "Are you sure you want to continue?"
//! );
//! dialog.add(text);
//!
//! // Add OK button
//! let ok_button = Button::new(
//!     Rect::new(10, 4, 18, 6),
//!     "OK",
//!     turbo_vision::core::command::CM_OK,
//!     true  // default button
//! );
//! dialog.add(ok_button);
//!
//! // Add Cancel button
//! let cancel_button = Button::new(
//!     Rect::new(22, 4, 32, 6),
//!     "Cancel",
//!     turbo_vision::core::command::CM_CANCEL,
//!     false
//! );
//! dialog.add(cancel_button);
//!
//! dialog
//! # }
//! ```
//!
//! ## Handling Events
//!
//! ```rust,no_run
//! use turbo_vision::prelude::*;
//! # use turbo_vision::app::Application;
//! # use turbo_vision::core::error::Result;
//! # fn example(mut app: Application) -> Result<()> {
//!
//! app.running = true;
//! while app.running {
//!     // Draw UI
//!     app.desktop.draw(&mut app.terminal);
//!     app.terminal.flush()?;
//!
//!     // Poll for events
//!     if let Ok(Some(mut event)) = app.terminal.poll_event(
//!         std::time::Duration::from_millis(50)
//!     ) {
//!         // Let desktop handle event
//!         app.desktop.handle_event(&mut event);
//!
//!         // Check for commands
//!         if event.what == EventType::Command {
//!             match event.command {
//!                 CM_QUIT => {
//!                     app.running = false;
//!                 }
//!                 _ => {}
//!             }
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Borland Turbo Vision Compatibility
//!
//! This implementation maintains conceptual compatibility with Borland Turbo Vision
//! while modernizing the design for Rust's ownership model:
//!
//! - **`TView`** → [`View`](views::View) trait
//! - **`TWindow`** → [`Window`](views::window::Window)
//! - **`TDialog`** → [`Dialog`](views::dialog::Dialog)
//! - **`TButton`** → [`Button`](views::button::Button)
//! - **`TInputLine`** → [`InputLine`](views::input_line::InputLine)
//! - **`TProgram`** → [`Application`](app::Application)
//!
//! Event handling uses Rust's ownership system instead of raw pointers, with
//! events bubbling up through the call stack rather than using owner pointers.
//!
//! # See Also
//!
//! - [Examples](https://github.com/aovestdipaperino/turbo-vision-4-rust/tree/main/examples) - Complete working examples
//! - [Borland TV Documentation](https://github.com/magiblot/tvision) - Original reference

// Core modules
pub mod app;
pub mod core;
pub mod helpers;
pub mod i18n;
pub mod terminal;
pub mod views;

// Serving an application over SSH lives in the `tv-extensions` crate (`ssh` feature).

// Test utilities (available to the crate's own tests and, for downstream
// crates, behind the `test-util` feature)
#[cfg(any(test, feature = "test-util"))]
pub mod test_util;

#[cfg(test)]
mod unicode_tests;

// The guides' code blocks are compiled as doctests, so they cannot drift
// from the API they describe.
#[cfg(doctest)]
#[doc = include_str!("../AGENTS.md")]
pub struct AgentsGuide;

// Its plural examples need Fluent.
#[cfg(all(doctest, feature = "i18n"))]
#[doc = include_str!("../docs/user-guide/Chapter-19-Localization.md")]
pub struct LocalizationGuide;

// Re-export commonly used types
pub mod prelude {
    pub use crate::core::event::{Event, EventType, KeyCode};
    pub use crate::core::geometry::{Point, Rect};

    // Explicit command re-exports (no glob imports)
    pub use crate::core::command::{
        // Application commands
        CM_CANCEL,
        CM_CLOSE,
        CM_CLOSE_FILE,
        CM_COMMAND_SET_CHANGED,
        CM_COPY,
        CM_CUT,
        CM_DEFAULT,
        CM_FILE_DOUBLE_CLICKED,
        CM_FILE_FOCUSED,
        CM_FIND,
        CM_GOTO_LINE,
        CM_GRAB_DEFAULT,
        // Help commands
        CM_HELP_INDEX,
        // Demo commands
        // File operations
        CM_NEW,
        CM_NO,
        CM_OK,
        CM_OPEN,
        CM_PASTE,
        // Basic dialog commands
        CM_QUIT,
        CM_RECEIVED_FOCUS,
        CM_REDO,
        // Internal view system commands
        CM_REDRAW,
        CM_RELEASE_DEFAULT,
        CM_RELEASED_FOCUS,
        CM_REPLACE,
        CM_SAVE,
        CM_SAVE_ALL,
        CM_SAVE_AS,
        CM_SEARCH_AGAIN,
        CM_SELECT_ALL,
        // Edit operations
        CM_UNDO,
        CM_USER,
        CM_YES,
        // View commands
        CommandId,
    };

    pub use crate::app::{AppHandler, Application, ModalTick};
    pub use crate::core::state::{Grow, Options, State};
    pub use crate::views::msgbox::MsgBox;
    pub use crate::views::{GroupLike, Handle, View, ViewCore, WindowLike};
}
