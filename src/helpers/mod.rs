// (C) 2025 - Enzo Lombardi

//! Helper functions and utilities for common UI patterns.

pub mod msgbox;

// Re-export commonly used functions and constants
pub use msgbox::{
    MF_ABOUT, MF_CANCEL_BUTTON, MF_CONFIRMATION, MF_ERROR, MF_INFORMATION, MF_NO_BUTTON,
    MF_OK_BUTTON, MF_OK_CANCEL, MF_WARNING, MF_YES_BUTTON, MF_YES_NO_CANCEL, input_box,
    input_box_rect, message_box, message_box_rect,
};
