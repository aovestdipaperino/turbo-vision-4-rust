// (C) 2025 - Enzo Lombardi

//! Application module providing the main application structure and event loop.
//!
//! This module contains the [`Application`] type which serves as the central
//! coordinator for Turbo Vision applications. It manages:
//! - The terminal instance
//! - The desktop (root container for all windows)
//! - Optional menu bar and status line
//! - The main event loop
//! - Modal dialog execution
//!
//! # Architecture
//!
//! A Turbo Vision application follows this structure:
//!
//! ```text
//! Application
//! ├── Terminal (rendering backend)
//! ├── Desktop (window manager)
//! │   ├── Background
//! │   └── Windows/Dialogs
//! ├── MenuBar (optional)
//! └── StatusLine (optional)
//! ```
//!
//! # Examples
//!
//! Basic application with event loop:
//!
//! ```rust,no_run
//! use turbo_vision::app::Application;
//! use turbo_vision::views::View;
//! use turbo_vision::core::error::Result;
//! use turbo_vision::core::event::EventType;
//! use turbo_vision::core::command::CM_QUIT;
//!
//! fn main() -> Result<()> {
//!     let mut app = Application::new()?;
//!
//!     app.running = true;
//!     while app.running {
//!         // Draw
//!         app.desktop.draw(&mut app.terminal);
//!         app.terminal.flush()?;
//!
//!         // Handle events
//!         if let Ok(Some(mut event)) = app.terminal.poll_event(
//!             std::time::Duration::from_millis(50)
//!         ) {
//!             app.desktop.handle_event(&mut event);
//!
//!             if event.what == EventType::Command && event.command == CM_QUIT {
//!                 app.running = false;
//!             }
//!         }
//!     }
//!
//!     app.terminal.shutdown()?;
//!     Ok(())
//! }
//! ```

pub mod application;

pub use application::Application;
