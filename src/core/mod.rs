// (C) 2025 - Enzo Lombardi

//! Core module containing fundamental TUI framework types.
//!
//! This module provides the essential building blocks for the Turbo Vision
//! framework including:
//! - **Geometry primitives** ([`geometry`]): [`Point`](geometry::Point), [`Rect`](geometry::Rect) for layout
//! - **Event handling** ([`event`]): [`Event`](event::Event), [`KeyCode`](event::KeyCode), mouse events
//! - **Drawing utilities** ([`draw`]): [`Cell`](draw::Cell), [`Buffer`](draw::Buffer), [`Attr`](draw::Attr) for terminal rendering
//! - **Command system** ([`command`], [`command_set`]): Action management and command routing
//! - **Color management** ([`palette`]): Terminal color schemes and attributes
//! - **Error handling** ([`error`]): [`Result`](error::Result), [`TurboVisionError`](error::TurboVisionError)
//! - **State management** ([`state`]): View state flags and constants
//! - **Clipboard** ([`clipboard`]): Copy/paste support
//! - **History** ([`history`]): Input history management
//!
//! # Examples
//!
//! Creating and working with geometric primitives:
//!
//! ```rust
//! use turbo_vision::core::geometry::{Point, Rect};
//!
//! let origin = Point::new(0, 0);
//! let size = Point::new(80, 25);
//! let screen_bounds = Rect::from_points(origin, size);
//!
//! assert!(screen_bounds.contains(Point::new(40, 12)));
//! ```
//!
//! Handling events:
//!
//! ```rust
//! use turbo_vision::core::event::{Event, EventType};
//!
//! # let event = Event::nothing();
//! match event.what {
//!     EventType::Keyboard => {
//!         // Handle keyboard event
//!     }
//!     EventType::MouseDown => {
//!         // Handle mouse click at event.mouse.pos
//!     }
//!     _ => {}
//! }
//! ```

pub mod geometry;
pub mod draw;
pub mod event;
pub mod command;
pub mod command_set;
pub mod palette;
pub mod clipboard;
pub mod state;
pub mod ansi_dump;
pub mod ansi;
pub mod menu_data;
pub mod status_data;
pub mod history;
pub mod error;
