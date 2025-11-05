// (C) 2025 - Enzo Lombardi
// Turbo Vision - Rust TUI Library
// Core modules
pub mod core;
pub mod terminal;
pub mod views;
pub mod app;

// Re-export commonly used types
pub mod prelude {
    pub use crate::core::geometry::{Point, Rect};
    pub use crate::core::event::{Event, EventType, KeyCode};
    pub use crate::core::command::*;
    pub use crate::views::View;
    pub use crate::app::Application;
}
