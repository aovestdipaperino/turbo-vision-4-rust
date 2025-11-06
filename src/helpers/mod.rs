// (C) 2025 - Enzo Lombardi

//! Helper functions and utilities for common UI patterns.

pub mod msgbox;

// Re-export commonly used functions and constants
pub use msgbox::{
    message_box, message_box_rect, input_box, input_box_rect,
    MF_WARNING, MF_ERROR, MF_INFORMATION, MF_CONFIRMATION,
    MF_YES_BUTTON, MF_NO_BUTTON, MF_OK_BUTTON, MF_CANCEL_BUTTON,
    MF_YES_NO_CANCEL, MF_OK_CANCEL,
};
