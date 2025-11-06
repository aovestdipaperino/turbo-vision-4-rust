// (C) 2025 - Enzo Lombardi

//! Views module containing all UI components and widgets.
//!
//! This module provides the building blocks for creating text-based user interfaces,
//! including windows, dialogs, buttons, input fields, and more. All widgets implement
//! the [`View`] trait which provides a consistent interface for drawing, event handling,
//! and focus management.
//!
//! # Widget Categories
//!
//! ## Core Components
//! - [`View`] - Base trait for all UI components
//! - [`Group`](group::Group) - Container for organizing child views
//! - [`Window`](window::Window) - Movable, resizable window with frame
//! - [`Dialog`](dialog::Dialog) - Modal dialog with standard button handling
//! - [`Desktop`](desktop::Desktop) - Root container managing all windows
//!
//! ## Input Widgets
//! - [`InputLine`](input_line::InputLine) - Single-line text input with validation
//! - [`Editor`](editor::Editor) - Multi-line text editor with syntax highlighting
//! - [`Button`](button::Button) - Clickable button that emits commands
//! - [`CheckBox`](checkbox::CheckBox) - Binary on/off checkbox
//! - [`RadioButton`](radiobutton::RadioButton) - Mutually exclusive radio buttons
//!
//! ## Display Widgets
//! - [`StaticText`](static_text::StaticText) - Non-interactive text label
//! - [`TextViewer`](text_viewer::TextViewer) - Scrollable read-only text viewer
//! - [`ListBox`](listbox::ListBox) - Scrollable list of selectable items
//! - [`Memo`](memo::Memo) - Multi-line read-only text display
//!
//! ## Menus and Status
//! - [`MenuBar`](menu_bar::MenuBar) - Top menu bar with pull-down menus
//! - [`StatusLine`](status_line::StatusLine) - Bottom status line with key hints
//!
//! ## Dialogs and Utilities
//! - [`FileDialog`](file_dialog::FileDialog) - File selection dialog
//! - [`msgbox`] - Message boxes and confirmation dialogs
//! - [`HelpWindow`](help_window::HelpWindow) - Context-sensitive help system
//!
//! # Examples
//!
//! Creating a simple window with a button:
//!
//! ```rust,no_run
//! use turbo_vision::views::{Window, Button};
//! use turbo_vision::core::geometry::Rect;
//! use turbo_vision::core::command::CM_OK;
//!
//! let mut window = Window::new(Rect::new(10, 5, 50, 15), "My Window");
//! let button = Button::new(Rect::new(15, 5, 25, 7), "OK", CM_OK);
//! window.add(Box::new(button));
//! ```

pub mod view;
pub mod group;
pub mod window;
pub mod frame;
pub mod dialog;
pub mod desktop;
pub mod status_line;
pub mod menu_bar;
pub mod menu_viewer;
pub mod menu_box;
pub mod button;
pub mod static_text;
pub mod input_line;
pub mod label;
pub mod scrollbar;
pub mod scroller;
pub mod indicator;
pub mod text_viewer;
pub mod cluster;
pub mod checkbox;
pub mod radiobutton;
pub mod listbox;
pub mod sorted_listbox;
pub mod list_viewer;
pub mod history_viewer;
pub mod history_window;
pub mod history;
pub mod paramtext;
pub mod background;
pub mod memo;
pub mod editor;
pub mod edit_window;
pub mod file_editor;
pub mod file_dialog;
pub mod file_list;
pub mod dir_listbox;
pub mod msgbox;
pub mod validator;
pub mod lookup_validator;
pub mod picture_validator;
pub mod syntax;
pub mod help_file;
pub mod help_viewer;
pub mod help_window;
pub mod help_context;
pub mod outline;
pub mod terminal_widget;
pub mod chdir_dialog;

#[doc(inline)]
pub use view::View;
#[doc(inline)]
pub use list_viewer::{ListViewer, ListViewerState};
#[doc(inline)]
pub use menu_viewer::{MenuViewer, MenuViewerState};
#[doc(inline)]
pub use menu_box::MenuBox;
#[doc(inline)]
pub use cluster::{Cluster, ClusterState};
#[doc(inline)]
pub use label::Label;
