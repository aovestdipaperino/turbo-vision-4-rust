// (C) 2025 - Enzo Lombardi
//! The gallery's list of demos.
//!
//! Each entry ties a demo file to what the gallery needs to show it. The
//! file's own text is the code shown (`include_str!`), so adding a demo is:
//! write `demos/<name>.rs`, add a `pub mod` line in `demos/mod.rs`, and an
//! entry here.
//!
//! The list groups the demos by the kind of component, in the order of
//! [`Group`], and orders them by name within a group.

use super::demos;
use super::panel::Panel;
use std::ops::RangeInclusive;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_USER, CommandId};

/// The commands demos may use (D8). Demos share them, one at a time, so
/// the gallery enables them all again before it shows a demo: a demo that
/// disables one (the Button demo does) must not grey out the next demo's.
pub const DEMO_COMMANDS: RangeInclusive<CommandId> = CM_USER + 100..=CM_USER + 199;

/// The columns a demo may use: the panel's width on an 80-column terminal,
/// inside its "Try it" box.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "the tests check every demo against it")
)]
pub const PANEL_WIDTH: i16 = 48;

/// The kinds of component the list groups the demos under, in list order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    /// Controls that pick or set a value: buttons, boxes, numbers.
    Controls,
    /// Views the user types text into.
    TextEntry,
    /// Views that browse rows of items.
    Lists,
    /// Views that show text or progress and take no input of their own.
    Display,
    /// Views that hold, frame or lay out other views.
    Containers,
    /// Ready-made modal dialogs.
    Dialogs,
    /// The application's menus and status line.
    Menus,
}

impl Group {
    /// The heading shown above the group in the list.
    pub fn title(self) -> &'static str {
        match self {
            Group::Controls => "Controls",
            Group::TextEntry => "Text entry",
            Group::Lists => "Lists and tables",
            Group::Display => "Display",
            Group::Containers => "Containers",
            Group::Dialogs => "Dialogs",
            Group::Menus => "Menus and status",
        }
    }
}

/// One component's demo.
pub struct Demo {
    /// The name in the list and the panel's title.
    pub name: &'static str,
    /// The kind of component, which the list groups it under.
    pub group: Group,
    /// The module, under `turbo_vision::`.
    pub module: &'static str,
    /// The demo file, as written: a `//!` header, then the code.
    pub source: &'static str,
    /// Rows the live demo takes.
    pub height: i16,
    /// Put the demo's views in the panel, from `(0, 0)` and at most
    /// [`PANEL_WIDTH`] wide.
    pub build: fn(&mut Panel),
    /// React to the demo's commands; `true` when it handled one.
    pub handle: Option<fn(&mut Application, CommandId) -> bool>,
}

/// Every demo, in the order of the list: by group, then by name.
pub const DEMOS: &[Demo] = &[
    Demo {
        name: "Button",
        group: Group::Controls,
        module: "views::button",
        source: include_str!("demos/button.rs"),
        height: 2,
        build: demos::button::build,
        handle: Some(demos::button::handle),
    },
    Demo {
        name: "CheckBoxes",
        group: Group::Controls,
        module: "views::cluster_group",
        source: include_str!("demos/check_boxes.rs"),
        height: 4,
        build: demos::check_boxes::build,
        handle: None,
    },
    Demo {
        name: "ComboBox",
        group: Group::Controls,
        module: "views::combo_box",
        source: include_str!("demos/combo_box.rs"),
        height: 3,
        build: demos::combo_box::build,
        handle: None,
    },
    Demo {
        name: "RadioButton",
        group: Group::Controls,
        module: "views::radiobutton",
        source: include_str!("demos/radio_button.rs"),
        height: 3,
        build: demos::radio_button::build,
        handle: None,
    },
    Demo {
        name: "RadioButtons",
        group: Group::Controls,
        module: "views::cluster_group",
        source: include_str!("demos/radio_buttons.rs"),
        height: 3,
        build: demos::radio_buttons::build,
        handle: None,
    },
    Demo {
        name: "ScrollBar",
        group: Group::Controls,
        module: "views::scrollbar",
        source: include_str!("demos/scroll_bar.rs"),
        height: 6,
        build: demos::scroll_bar::build,
        handle: None,
    },
    Demo {
        name: "Slider",
        group: Group::Controls,
        module: "views::slider",
        source: include_str!("demos/slider.rs"),
        height: 3,
        build: demos::slider::build,
        handle: None,
    },
    Demo {
        name: "Spinner",
        group: Group::Controls,
        module: "views::spinner",
        source: include_str!("demos/spinner.rs"),
        height: 5,
        build: demos::spinner::build,
        handle: None,
    },
    Demo {
        name: "Editor",
        group: Group::TextEntry,
        module: "views::editor",
        source: include_str!("demos/editor.rs"),
        height: 7,
        build: demos::editor::build,
        handle: Some(demos::editor::handle),
    },
    Demo {
        name: "History",
        group: Group::TextEntry,
        module: "views::history",
        source: include_str!("demos/history.rs"),
        height: 1,
        build: demos::history::build,
        handle: None,
    },
    Demo {
        name: "InputLine",
        group: Group::TextEntry,
        module: "views::input_line",
        source: include_str!("demos/input_line.rs"),
        height: 5,
        build: demos::input_line::build,
        handle: None,
    },
    Demo {
        name: "Memo",
        group: Group::TextEntry,
        module: "views::memo",
        source: include_str!("demos/memo.rs"),
        height: 6,
        build: demos::memo::build,
        handle: None,
    },
    Demo {
        name: "ListBox",
        group: Group::Lists,
        module: "views::listbox",
        source: include_str!("demos/list_box.rs"),
        height: 5,
        build: demos::list_box::build,
        handle: Some(demos::list_box::handle),
    },
    Demo {
        name: "Outline",
        group: Group::Lists,
        module: "views::outline",
        source: include_str!("demos/outline.rs"),
        height: 7,
        build: demos::outline::build,
        handle: None,
    },
    Demo {
        name: "SortedListBox",
        group: Group::Lists,
        module: "views::sorted_listbox",
        source: include_str!("demos/sorted_list_box.rs"),
        height: 6,
        build: demos::sorted_list_box::build,
        handle: None,
    },
    Demo {
        name: "Table",
        group: Group::Lists,
        module: "views::table",
        source: include_str!("demos/table.rs"),
        height: 6,
        build: demos::table::build,
        handle: None,
    },
    Demo {
        name: "ProgressBar",
        group: Group::Display,
        module: "views::progress_bar",
        source: include_str!("demos/progress_bar.rs"),
        height: 6,
        build: demos::progress_bar::build,
        handle: Some(demos::progress_bar::handle),
    },
    Demo {
        name: "StaticText, Label",
        group: Group::Display,
        module: "views::static_text",
        source: include_str!("demos/static_text.rs"),
        height: 6,
        build: demos::static_text::build,
        handle: None,
    },
    Demo {
        name: "TextViewer",
        group: Group::Display,
        module: "views::text_viewer",
        source: include_str!("demos/text_viewer.rs"),
        height: 6,
        build: demos::text_viewer::build,
        handle: None,
    },
    Demo {
        name: "Tooltip",
        group: Group::Display,
        module: "views::tooltip",
        source: include_str!("demos/tooltip.rs"),
        height: 3,
        build: demos::tooltip::build,
        handle: Some(demos::tooltip::handle),
    },
    Demo {
        name: "GroupBox",
        group: Group::Containers,
        module: "views::group_box",
        source: include_str!("demos/group_box.rs"),
        height: 5,
        build: demos::group_box::build,
        handle: None,
    },
    Demo {
        name: "SplitPane",
        group: Group::Containers,
        module: "views::split_pane",
        source: include_str!("demos/split_pane.rs"),
        height: 7,
        build: demos::split_pane::build,
        handle: None,
    },
    Demo {
        name: "TabbedPane",
        group: Group::Containers,
        module: "views::tabbed_pane",
        source: include_str!("demos/tabbed_pane.rs"),
        height: 7,
        build: demos::tabbed_pane::build,
        handle: None,
    },
    Demo {
        name: "Window",
        group: Group::Containers,
        module: "views::window",
        source: include_str!("demos/window.rs"),
        height: 2,
        build: demos::window::build,
        handle: Some(demos::window::handle),
    },
    Demo {
        name: "ColorDialog",
        group: Group::Dialogs,
        module: "views::color_dialog",
        source: include_str!("demos/color_dialog.rs"),
        height: 2,
        build: demos::color_dialog::build,
        handle: Some(demos::color_dialog::handle),
    },
    Demo {
        name: "File dialogs",
        group: Group::Dialogs,
        module: "views::file_dialog",
        source: include_str!("demos/file_dialogs.rs"),
        height: 2,
        build: demos::file_dialogs::build,
        handle: Some(demos::file_dialogs::handle),
    },
    Demo {
        name: "Help",
        group: Group::Dialogs,
        module: "views::help_window",
        source: include_str!("demos/help.rs"),
        height: 2,
        build: demos::help::build,
        handle: Some(demos::help::handle),
    },
    Demo {
        name: "Languages",
        group: Group::Dialogs,
        module: "i18n",
        source: include_str!("demos/languages.rs"),
        height: 5,
        build: demos::languages::build,
        handle: Some(demos::languages::handle),
    },
    Demo {
        name: "Message boxes",
        group: Group::Dialogs,
        module: "views::msgbox",
        source: include_str!("demos/message_boxes.rs"),
        height: 2,
        build: demos::message_boxes::build,
        handle: Some(demos::message_boxes::handle),
    },
    Demo {
        name: "MenuBar",
        group: Group::Menus,
        module: "views::menu_bar",
        source: include_str!("demos/menu_bar.rs"),
        height: 2,
        build: demos::menu_bar::build,
        handle: Some(demos::menu_bar::handle),
    },
    Demo {
        name: "MenuBox",
        group: Group::Menus,
        module: "views::menu_box",
        source: include_str!("demos/menu_box.rs"),
        height: 2,
        build: demos::menu_box::build,
        handle: Some(demos::menu_box::handle),
    },
    Demo {
        name: "StatusLine",
        group: Group::Menus,
        module: "views::status_line",
        source: include_str!("demos/status_line.rs"),
        height: 4,
        build: demos::status_line::build,
        handle: Some(demos::status_line::handle),
    },
];

/// The line of a header that names related components.
const SEE_ALSO: &str = "See also:";

/// Split a demo file into its `//!` header, as plain text, and the code
/// after it.
pub fn split_source(source: &str) -> (String, String) {
    let mut how = Vec::new();
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next_if(|l| l.starts_with("//!")) {
        how.push(line.trim_start_matches("//!").trim_start().to_string());
    }
    // The blank line between the header and the code.
    lines.next_if(|l| l.trim().is_empty());
    let code: Vec<&str> = lines.collect();
    (how.join("\n"), code.join("\n"))
}

/// Take the "See also:" line out of a header: the header without it, and
/// the names it lists.
pub fn split_see_also(how: &str) -> (String, Vec<String>) {
    let mut related = Vec::new();
    let mut text = Vec::new();
    for line in how.lines() {
        match line.strip_prefix(SEE_ALSO) {
            Some(names) => related.extend(
                names
                    .split(',')
                    .map(|name| name.trim().trim_end_matches('.').to_string())
                    .filter(|name| !name.is_empty()),
            ),
            None => text.push(line),
        }
    }
    (text.join("\n").trim_end().to_string(), related)
}

/// The demo a "See also" name refers to: the one with that name, or whose
/// name lists it (`Label` is the `StaticText, Label` demo).
pub fn find(name: &str) -> Option<usize> {
    DEMOS
        .iter()
        .position(|d| d.name == name || d.name.split(", ").any(|part| part == name))
}
