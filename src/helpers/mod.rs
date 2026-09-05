// (C) 2025 - Enzo Lombardi

//! Deprecated shims. The message-box helpers live in [`crate::views::msgbox`];
//! this module re-exports them under their 2.x names for one release.

#![allow(deprecated)]

use crate::app::Application;
use crate::core::command::{CM_CANCEL, CM_OK, CommandId};
use crate::core::geometry::Rect;

/// The message-box module, moved to `views::msgbox`.
pub mod msgbox {
    //! Deprecated: use [`crate::views::msgbox`].
    pub use super::{input_box, input_box_rect};
    pub use crate::views::msgbox::*;
}

pub use crate::views::msgbox::{
    MF_ABOUT, MF_CANCEL_BUTTON, MF_CONFIRMATION, MF_ERROR, MF_INFORMATION, MF_NO_BUTTON,
    MF_OK_BUTTON, MF_OK_CANCEL, MF_WARNING, MF_YES_BUTTON, MF_YES_NO_CANCEL, message_box,
    message_box_rect,
};

/// Display an input box; returns the button command and the text.
#[deprecated(
    since = "3.0.0",
    note = "use `views::msgbox::input_box`, which returns `Option<String>`"
)]
pub fn input_box(
    app: &mut Application,
    title: &str,
    label: &str,
    default: &str,
    limit: usize,
) -> (CommandId, String) {
    match crate::views::msgbox::input_box(app, title, label, default, limit) {
        Some(text) => (CM_OK, text),
        None => (CM_CANCEL, default.to_string()),
    }
}

/// Display an input box in the given rectangle; returns the button command
/// and the text.
#[deprecated(
    since = "3.0.0",
    note = "use `views::msgbox::input_box_rect`, which returns `Option<String>`"
)]
pub fn input_box_rect(
    app: &mut Application,
    bounds: Rect,
    title: &str,
    label: &str,
    default: &str,
    limit: usize,
) -> (CommandId, String) {
    match crate::views::msgbox::input_box_rect(app, bounds, title, label, default, limit) {
        Some(text) => (CM_OK, text),
        None => (CM_CANCEL, default.to_string()),
    }
}
