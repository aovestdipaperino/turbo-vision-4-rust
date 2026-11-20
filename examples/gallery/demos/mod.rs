// (C) 2025 - Enzo Lombardi
//! One file per component. Each file starts with a `//!` header, shown in
//! the gallery as "How it works"; the rest of the file is shown as the code.
//!
//! The header has three parts, a blank `//!` line between them: what the
//! component does and its keys; "Parameters:", one "- " item per
//! constructor argument or setter, saying what it changes; and a "See
//! also:" line naming up to three related demos, which the gallery shows
//! as buttons. The tests check all three.

// The headers are shown as plain text in the gallery, where backticks would
// appear as they are; key names like PgUp are not code.
#![allow(
    clippy::doc_markdown,
    reason = "demo headers are displayed as plain text in the gallery"
)]

pub mod button;
pub mod check_boxes;
pub mod color_dialog;
pub mod combo_box;
pub mod editor;
pub mod file_dialogs;
pub mod group_box;
pub mod help;
pub mod history;
pub mod input_line;
pub mod languages;
pub mod list_box;
pub mod memo;
pub mod menu_bar;
pub mod menu_box;
pub mod message_boxes;
pub mod outline;
pub mod progress_bar;
pub mod radio_button;
pub mod radio_buttons;
pub mod scroll_bar;
pub mod slider;
pub mod sorted_list_box;
pub mod spinner;
pub mod split_pane;
pub mod static_text;
pub mod status_line;
pub mod tabbed_pane;
pub mod table;
pub mod text_viewer;
pub mod tooltip;
pub mod window;
