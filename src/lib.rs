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
//! ```rust,no_run
//! use turbo_vision::prelude::*;
//!
//! fn main() -> turbo_vision::core::error::Result<()> {
//!     // Create application with terminal
//!     let mut app = Application::new()?;
//!
//!     // Create a simple window
//!     let mut window = turbo_vision::views::window::Window::new(
//!         Rect::new(10, 5, 50, 15),
//!         "My First Window"
//!     );
//!
//!     // Add a button
//!     let button = turbo_vision::views::button::Button::new(
//!         Rect::new(15, 5, 25, 7),
//!         "Click Me",
//!         turbo_vision::core::command::CM_OK,
//!         false
//!     );
//!     window.add(Box::new(button));
//!
//!     // Add window to desktop
//!     app.desktop.add(Box::new(window));
//!
//!     // Run event loop
//!     app.running = true;
//!     while app.running {
//!         app.desktop.draw(&mut app.terminal);
//!         app.terminal.flush()?;
//!
//!         if let Ok(Some(mut event)) = app.terminal.poll_event(
//!             std::time::Duration::from_millis(50)
//!         ) {
//!             app.desktop.handle_event(&mut event);
//!
//!             if event.what == EventType::Command {
//!                 match event.command {
//!                     CM_QUIT => app.running = false,
//!                     CM_OK => {
//!                         // Handle button click
//!                         app.running = false;
//!                     }
//!                     _ => {}
//!                 }
//!             }
//!         }
//!     }
//!
//!     app.terminal.shutdown()?;
//!     Ok(())
//! }
//! ```
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
//!   - [`Editor`](views::editor::Editor) - Multi-line text editor
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
//! dialog.add(Box::new(text));
//!
//! // Add OK button
//! let ok_button = Button::new(
//!     Rect::new(10, 4, 18, 6),
//!     "OK",
//!     turbo_vision::core::command::CM_OK,
//!     true  // default button
//! );
//! dialog.add(Box::new(ok_button));
//!
//! // Add Cancel button
//! let cancel_button = Button::new(
//!     Rect::new(22, 4, 32, 6),
//!     "Cancel",
//!     turbo_vision::core::command::CM_CANCEL,
//!     false
//! );
//! dialog.add(Box::new(cancel_button));
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
//! - **TView** → [`View`](views::View) trait
//! - **TWindow** → [`Window`](views::window::Window)
//! - **TDialog** → [`Dialog`](views::dialog::Dialog)
//! - **TButton** → [`Button`](views::button::Button)
//! - **TInputLine** → [`InputLine`](views::input_line::InputLine)
//! - **TProgram** → [`Application`](app::Application)
//!
//! Event handling uses Rust's ownership system instead of raw pointers, with
//! events bubbling up through the call stack rather than using owner pointers.
//!
//! # See Also
//!
//! - [Examples](https://github.com/anthropics/turbo-vision/tree/main/examples) - Complete working examples
//! - [Borland TV Documentation](https://github.com/magiblot/tvision) - Original reference

// Core modules
pub mod core;
pub mod terminal;
pub mod views;
pub mod app;
pub mod helpers;

// Test utilities (only available with test-util feature)
#[cfg(feature = "test-util")]
pub mod test_util;

// Re-export commonly used types
pub mod prelude {
    pub use crate::core::geometry::{Point, Rect};
    pub use crate::core::event::{Event, EventType, KeyCode};

    // Explicit command re-exports (no glob imports)
    pub use crate::core::command::{
        CommandId,
        // Basic dialog commands
        CM_QUIT,
        CM_CLOSE,
        CM_OK,
        CM_CANCEL,
        CM_YES,
        CM_NO,
        CM_DEFAULT,
        // Internal view system commands
        CM_COMMAND_SET_CHANGED,
        CM_RECEIVED_FOCUS,
        CM_RELEASED_FOCUS,
        CM_GRAB_DEFAULT,
        CM_RELEASE_DEFAULT,
        CM_FILE_FOCUSED,
        CM_FILE_DOUBLE_CLICKED,
        // Application commands
        CM_ABOUT,
        CM_BIRTHDATE,
        CM_TEXT_VIEWER,
        CM_CONTROLS_DEMO,
        // File operations
        CM_NEW,
        CM_OPEN,
        CM_SAVE,
        CM_SAVE_AS,
        CM_SAVE_ALL,
        CM_CLOSE_FILE,
        // Edit operations
        CM_UNDO,
        CM_REDO,
        CM_CUT,
        CM_COPY,
        CM_PASTE,
        CM_SELECT_ALL,
        CM_FIND,
        CM_REPLACE,
        CM_SEARCH_AGAIN,
        CM_FIND_IN_FILES,
        CM_GOTO_LINE,
        // View commands
        CM_ZOOM_IN,
        CM_ZOOM_OUT,
        CM_TOGGLE_SIDEBAR,
        CM_TOGGLE_STATUSBAR,
        // Help commands
        CM_HELP_INDEX,
        CM_KEYBOARD_REF,
        // Demo commands
        CM_LISTBOX_DEMO,
        CM_LISTBOX_SELECT,
        CM_MEMO_DEMO,
    };

    pub use crate::views::View;
    pub use crate::app::Application;
}
