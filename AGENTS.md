# Turbo Vision for Rust: guide for AI assistants and new developers

This is the entry point for writing applications **with** this crate, and for
working **on** it. It explains the Turbo Vision way of building a text UI, the
idiomatic Rust shape of each task, and every component you can use. Each Rust
block below is compiled as a doctest (`cargo test --doc`), so it matches the
current API.

- Crate: `turbo-vision` (import as `turbo_vision`), Rust edition 2024.
- A Rust port of Borland Turbo Vision: windows, dialogs, menus and controls in
  a terminal, with keyboard and mouse.
- Read next: `cargo doc --open` (full API), [`examples/`](examples/) (runnable programs).
- Companion crate: [`tv-extensions`](https://github.com/aovestdipaperino/tv-extensions)
  holds further components, such as `AutoComplete`, `ScrollPane` and popup
  menus.

## Contents

1. [The Turbo Vision way, in one page](#1-the-turbo-vision-way-in-one-page)
2. [The application skeleton](#2-the-application-skeleton)
3. [Component index](#3-component-index)
4. [Recipes](#4-recipes)
5. [Rules and pitfalls](#5-rules-and-pitfalls)
6. [Working on the crate itself](#6-working-on-the-crate-itself)
7. [Where to read more](#7-where-to-read-more)

## 1. The Turbo Vision way, in one page

**Everything on screen is a view.** A view is a rectangle that draws itself and
handles events: it implements the `View` trait. Containers (`Group`, `Window`,
`Dialog`, `Desktop`) hold child views and implement `GroupLike`. A running
program is a tree:

```text
Application
├── MenuBar                      top row
├── Desktop                      the area in between
│   ├── Window "Notes"           movable, resizable, closable
│   │   └── TextViewer
│   └── Window "Customers"
│       └── Table
└── StatusLine                   bottom row: key hints like "Alt-X Exit"

Dialog (modal, run on top with dialog.execute(&mut app))
├── Label, InputLine, CheckBox, ...
└── Button "OK", Button "Cancel"
```

**Coordinates are relative to the owner.** A child's `Rect` is measured from
its owner's top-left corner. For a window or dialog, that is the corner of the
area *inside* the frame. Inside a dialog you place controls by hand, relative to the
area inside the frame.

**Commands connect everything.** A command is a `u16` (`CommandId`). Menu items,
status line items and buttons send commands: OK sends `CM_OK`, Alt-X sends
`CM_QUIT`. Your code reacts to commands; it does not wire callbacks into
controls. The standard commands (`CM_OK`, `CM_CANCEL`, `CM_QUIT`, `CM_OPEN`, …)
are constants in `core::command`. Number your own from `CM_USER` (200), since
100–199 are reserved for the crate.

**Commands can be switched off globally.** `app.disable_command(c)` greys out
every menu item and button that sends `c`, and `app.enable_command(c)` brings
them back. This is how you express "Save is not possible now".

**Events flow down the tree.** Keys go to the focused view, in three phases:
first views marked `PRE_PROCESS`, then the focused view, then views marked
`POST_PROCESS` (buttons and labels answering their `~` hot keys). Mouse events
go to the topmost view under the pointer. A *command* event goes to the focused
chain and then to your application. A *broadcast* goes to every view of a
group: controls use broadcasts to tell their siblings that something changed.

**The application owns the event loop.** You give `app.run_with(&mut handler)`
a value that implements `AppHandler`, and it calls you back:
`handle_command` for commands nobody else handled, `idle` when nothing happens,
`pre_event` before anything sees an event, `window_closed` when a window goes.
Do not write your own loop.

**Modal means a nested loop.** `dialog.execute(&mut app)` (or
`editor.edit(&mut app, record)`) runs until the dialog closes and returns the
command that closed it (`CM_OK`, `CM_CANCEL`, …). Esc and the close box give
`CM_CANCEL`.

**A container owns its children.** After `window.add(view)` the window owns the
view. To reach it again, keep the typed `Handle<T>` from `add_typed` and ask
the container: `window.get_mut(handle)`. Use `Shared<T>` only when your own
struct must hold a view that a container also owns.

**Colours come from palettes.** A view never chooses colours directly. It asks
for a colour by role (`self.map_color(STATIC_TEXT_NORMAL)`), and the owner
chain maps it. The same view therefore looks right in a gray dialog and in a
blue window.

**One thread.** The UI runs on one thread. The global command set and the timer
queue are thread-local.

## 2. The application skeleton

Every application has this shape: build the application, give it a menu bar
and a status line, put your state in a struct that implements `AppHandler`,
and call `run_with`.

```rust,no_run
use turbo_vision::app::{AppHandler, Application};
use turbo_vision::core::command::{CM_QUIT, CM_USER, CommandId};
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem, MenuItemBuilder};
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::msgbox::message_box_ok;
use turbo_vision::views::status_line::StatusLine;

// Your commands start at CM_USER; 100-199 belong to the crate.
const CM_ABOUT: CommandId = CM_USER;
const CM_GREET: CommandId = CM_USER + 1;

/// The application's own state; the handler of its commands.
struct MyApp {
    greetings: u32,
}

impl AppHandler for MyApp {
    fn handle_command(&mut self, app: &mut Application, command: CommandId, _: &Event) -> bool {
        match command {
            CM_ABOUT => {
                message_box_ok(app, "My App 1.0");
                true
            }
            CM_GREET => {
                self.greetings += 1;
                message_box_ok(app, &format!("Hello #{}", self.greetings));
                true
            }
            _ => false, // not ours
        }
    }
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width, 1));
    menu_bar.add_submenu(SubMenu::new(
        "~F~ile",
        Menu::from_items(vec![
            MenuItemBuilder::new()
                .text("~G~reet")
                .command(CM_GREET)
                .key("Ctrl+G") // works while the menu is closed too
                .build(),
            MenuItem::separator(),
            MenuItemBuilder::new()
                .text("E~x~it")
                .command(CM_QUIT)
                .key("Alt+X")
                .build(),
        ]),
    ));
    menu_bar.add_submenu(SubMenu::new(
        "~H~elp",
        Menu::from_items(vec![
            MenuItemBuilder::new().text("~A~bout").command(CM_ABOUT).build(),
        ]),
    ));
    app.set_menu_bar(menu_bar);

    app.set_status_line(StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItemBuilder::new()
                .text("~Alt-X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
            StatusItemBuilder::new()
                .text("~Ctrl-G~ Greet")
                .key("Ctrl+G")
                .command(CM_GREET)
                .build(),
        ],
    ));

    app.run_with(&mut MyApp { greetings: 0 });
    Ok(())
}
```

What the application does for you, with no code:

- Alt+X and `CM_QUIT` end `run_with`.
- F10 opens the menu bar, and Alt plus a menu's `~` letter opens that menu.
- F1 shows the help file set with `set_help_file`.
- F12 saves an ANSI screen dump, and Ctrl+F12 a PNG.
- Windows on the desktop can be moved, resized, zoomed and closed. F6
  switches between them, and Alt+1…9 picks one by number.
- `CM_TILE` and `CM_CASCADE` arrange the windows.

## 3. Component index

Each entry gives the module (under `turbo_vision::`), what the component is
for, how to create it, and how to use its value. Most views also have a
`…Builder`. `Rect` arguments are owner-relative.

### Application shell

| Component | Module | Use it for | Create / use |
|-----------|--------|------------|--------------|
| `Application` | `app` | The program: terminal, desktop, menu bar, status line, event loop. | `Application::new()?`, `set_menu_bar`, `set_status_line`, `run_with(&mut handler)` |
| `AppHandler` | `app` | Your reactions to commands, idle time, closed windows. | `impl AppHandler for MyApp { fn handle_command(..) -> bool }` |
| `MenuBar`, `SubMenu` | `views::menu_bar` | The top menu. | `MenuBar::new(rect)`, `add_submenu(SubMenu::new("~F~ile", menu))` |
| `Menu`, `MenuItem`, `MenuItemBuilder`, `MenuBuilder` | `core::menu_data` | Menu contents. | `MenuItemBuilder::new().text("~O~pen").command(CM_OPEN).key("Ctrl+O").build()`, `MenuItem::separator()` |
| `MenuBox` | `views::menu_box` | A pop-up menu at a point (context menus). | `MenuBox::new(point, menu).execute(&mut app.terminal)` returns the command |
| `MenuViewer` | `views::menu_viewer` | The shared base of menu views (internal). | used by `MenuBar` and `MenuBox` |
| `StatusLine`, `StatusItemBuilder` | `views::status_line`, `core::status_data` | The bottom row of key hints. | `StatusLine::new(rect, vec![StatusItemBuilder::new().text("~F1~ Help").key("F1").command(CM_HELP_INDEX).build()])` |
| `Desktop` | `views::desktop` | Holds the windows (`app.desktop`). | `app.desktop.add(window)`, `add_typed`, `get_mut(handle)`, `tile`/`cascade` |
| `Background` | `views::background` | The desktop's pattern fill. | made by the desktop |

### Containers and layout

| Component | Module | Use it for | Create / use |
|-----------|--------|------------|--------------|
| `Window` | `views::window` | A framed, movable window on the desktop. | `Window::new(rect, "Title")`, `window.add(view)` / `add_typed(view)` |
| `Dialog` | `views::dialog` | A modal window of controls. | `Dialog::new(rect, "Title")`, `dialog.execute(&mut app)` |
| `GroupBox` | `views::group_box` | A titled box around related controls. | `GroupBox::new(rect, "Title")` |
| `Group` | `views::group` | A plain container (a tab page, a panel). | `Group::new(rect)`, `add(view)` |
| `TabbedPane` | `views::tabbed_pane` | Tabs over pages. | `TabbedPane::new(rect)`, `add_page("~G~eneral", group)` |
| `SplitPane`, `Orientation` | `views::split_pane` | Two panes with a draggable divider. | `SplitPane::new(rect, Orientation::Horizontal, position)` |
| `Frame` | `views::frame` | A window's border, title, close and zoom icons. | made by `Window` |
| `Scroller` | `views::scroller` | Base for scrollable content views. | used by viewers |
| `ScrollBar` | `views::scrollbar` | A scroll bar. | `ScrollBar::new_vertical(rect)`; usually created by the view that scrolls |

### Text input and editing

| Component | Module | Use it for | Create / use |
|-----------|--------|------------|--------------|
| `InputLine` | `views::input_line` | One line of text. | `InputLine::new(rect, max_len)`; `text()`, `set_text(..)`; `with_validator(..)` |
| `Memo` | `views::memo` | Several lines of text in a form. | `Memo::new(rect)`; `get_text()`, `set_text(..)` |
| `Label` | `views::label` | A field's caption; Alt+letter focuses its field. | `Label::new(rect, "~N~ame")`, `set_link(field_id)` |
| `History`, `HistoryViewer`, `HistoryWindow` | `views::history`, `views::history_viewer`, `views::history_window` | A drop-down of earlier entries for an input line. | `History::new(point, history_id, input_handle)` |
| `EditorWindow` | `views::editor` | A text editor view: undo, search, selection, syntax colours. | `EditorWindow::new(rect)`; `get_text()`, `set_text(..)` |
| `EditWindow` | `views::edit_window` | A window holding an editor. | `EditWindow::new(rect, "Title")` |
| `FileEditorWindow` | `views::file_editor` | An editor window bound to a file (load, save, "save changes?"). | `FileEditorWindow::new(rect, "Title")` |
| `Editor`, `FileEditor` traits | `views::editor_traits` | What editor windows share (Borland `TEditor`). | implemented by the editor windows |
| `SyntaxHighlighter`, `RustHighlighter` | `views::syntax` | Colouring in the editor. | implement `SyntaxHighlighter` for a language |
| `Validator`, `FilterValidator`, `RangeValidator` | `views::validator` | Restrict what an input line accepts. | `Rc::new(RefCell::new(RangeValidator::new(1, 99)))` |
| `PictureValidator` | `views::picture_validator` | Input masks like `##/##/####`. | `PictureValidator::new("##/##/####")` |
| `LookupValidator` | `views::lookup_validator` | Only values from a list. | `LookupValidator::new(vec![..])` |

### Choices

| Component | Module | Use it for | Create / use |
|-----------|--------|------------|--------------|
| `Button` | `views::button` | Sends a command when pressed (mouse, Enter, Space, Alt+letter). | `Button::new(rect, "~O~K", CM_OK, true)` |
| `CheckBox` | `views::checkbox` | One on/off option. | `CheckBox::new(rect, "~V~IP")`; `is_checked()`, `set_checked(..)` |
| `CheckBoxes`, `RadioButtons` | `views::cluster_group` | Several options in one control; one of several. | `CheckBoxes::new(rect, labels)`, `is_checked(i)`; `RadioButtons::new(rect, labels)`, `selected()` |
| `RadioButton` | `views::radiobutton` | A single radio button in a group (prefer `RadioButtons`). | `RadioButton::new(rect, "~A~", group_id)` |
| `Cluster`, `ClusterState` | `views::cluster` | The shared base of check and radio controls. | used by the controls above |
| `ComboBox` | `views::combo_box` | One choice from a drop-down list. | `ComboBox::with_items(rect, unique_id, items)`; `selected()`, `set_selected(..)` |
| `Spinner` | `views::spinner` | A whole number with up/down steppers. | `Spinner::new(rect, min, max)`; `value()`, `set_value(..)` |
| `Slider` | `views::slider` | A whole number on a track. | `Slider::new(rect, min, max)`; `value()` |

### Lists, tables and trees

| Component | Module | Use it for | Create / use |
|-----------|--------|------------|--------------|
| `ListBox` | `views::listbox` | A scrolling list of strings; single or multiple selection. | `ListBox::new(rect, CM_ON_SELECT)`, `set_items(vec)`, `get_selected_item()`; large lists: `set_provider(..)` |
| `SortedListBox` | `views::sorted_listbox` | A sorted list with type-to-search. | `SortedListBox::new(rect, cmd)`, `set_items(vec)` |
| `ListViewer`, `ListViewerState` | `views::list_viewer` | The shared base of list views. | implement for a list of your own |
| `Table`, `Column`, `Align`, `RowProvider` | `views::table` | Rows and columns with a header; frozen rows and columns; rows on demand. | `Table::new(rect, CM_ON_SELECT)`, `set_columns(vec![Column::new("Name", 20)])`, `set_rows(..)` or `set_provider(..)`; `selected_row()` |
| `OutlineViewer`, `Node` | `views::outline` | A tree that expands and collapses. | `OutlineViewer::new(rect, format_fn)` with `Node::with_children(..)` |
| `DirListBox`, `FileList` | `views::dir_listbox`, `views::file_list` | Folder and file lists (used by the file dialogs). | `FileList::new(rect, path)` |

### Display and feedback

| Component | Module | Use it for | Create / use |
|-----------|--------|------------|--------------|
| `StaticText` | `views::static_text` | Fixed text, several lines. | `StaticText::new(rect, "Text\nmore")` |
| `ParamText` | `views::paramtext` | Text with placeholders filled in later. | `ParamText::new(rect, "%s files")` |
| `TextViewer` | `views::text_viewer` | Read-only scrolling text (logs, reports). | `TextViewer::new(rect).with_scrollbars(true)`, `set_text(..)` |
| `ProgressBar`, `ProgressMode`, `ProgressStyle` | `views::progress_bar` | Progress, known or unknown. | `ProgressBar::new(rect, max)`, `set_value(..)`; `set_mode(ProgressMode::Marquee)` |
| `Indicator` | `views::indicator` | An editor's line:column and modified mark. | `Indicator::new(rect)` |
| `Tooltip` | `views::tooltip` | Hover hints for a dialog's controls. | `Tooltip::new(rect)`, `add_hint(target_rect, "text")` |
| `msgbox` functions | `views::msgbox` | Messages, questions, one-line input. | `message_box_ok(app, ..)`, `confirmation_box_yes_no(app, ..)`, `input_box(app, title, label, initial, max)` |
| `ColorDialog`, `ColorSelector` | `views::color_dialog`, `views::color_selector` | Pick a colour. | `ColorDialog::new(rect, "Colors", attr).execute(&mut app)` |

### Ready-made dialogs and help

| Component | Module | Use it for | Create / use |
|-----------|--------|------------|--------------|
| `FileDialog` | `views::file_dialog` | Open / Save a file. | `FileDialog::new(rect, "Open", "*.txt", None).build().execute(&mut app)` returns `Option<PathBuf>` |
| `ChDirDialog` | `views::chdir_dialog` | Change folder. | `ChDirDialog::new(None).execute(&mut app)` |
| `HelpFile`, `HelpWindow`, `HelpViewer`, `HelpIndex`, `HelpToc`, `HelpContext` | `views::help_file`, `views::help_window`, `views::help_viewer`, `views::help_index`, `views::help_toc`, `views::help_context` | F1 help from a Markdown file, with links, index and contents. | `app.set_help_file("help.md")?`, `app.register_help_context(id, "topic")` |

### Building blocks

| Item | Module | Use it for |
|------|--------|------------|
| `View`, `ViewCore`, `ViewId` | `views::view` | Writing your own view (see the recipe). `dispatch_to_child` / `draw_child` when you route events yourself. |
| `GroupLike`, `WindowLike` | `views::group`, `views::window` | The container traits: `add`, `add_typed`, `get`, `get_mut`, `child_by_id_mut`. Bring `GroupLike` into scope. |
| `Handle<T>` | `views::handle` | A typed reference to a child, from `add_typed`. |
| `Shared<T>` | `views::shared` | A view owned by a container and also held by your struct. |
| `Rect`, `Point` | `core::geometry` | Positions and sizes. |
| `Event`, `EventType`, `KB_*` | `core::event` | Keys, mouse, commands, broadcasts. `Event::command(c)`, `Event::keyboard(KB_F1)`. |
| `command::CM_*`, `CM_USER` | `core::command` | Command numbers. |
| `command_set` | `core::command_set` | The global enabled-commands set (`app.enable_command` wraps it). |
| `timed_event` | `core::timed_event` | Post an event to arrive after a delay. |
| `palette`, `Attr`, `TvColor` | `core::palette` | Colours by role; colour constants. |
| `Grow`, `Options`, `State` | `core::state` | How a view follows its owner's resize; option and state flags. |
| `Terminal` | `terminal` | Drawing target and input. Apps use `app.terminal.size()`. |
| `test_util` | `test_util` (feature `test-util`) | A terminal without a TTY, for tests. |
| `tr`, `tr!`, `set_locale`, `add_translations` | `i18n` (feature `i18n`, on by default) | Text in the user's language, with Project Fluent. The crate's own strings come in `en`, `es` and `ca`. |
| `LazyMessage`, `tr_lazy!` | `i18n` | A message translated when it is shown, not when it is made: from a worker thread, before the locale is chosen, or kept across a change of language. |
| `width_for_title` | `views::button` | How wide a button must be for a translated title. |

## 4. Recipes

Every recipe is a function you can call from `handle_command`.

### Show a message, ask a question

```rust,no_run
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_YES;
use turbo_vision::views::msgbox::{confirmation_box_yes_no, input_box, message_box_ok};

fn ask(app: &mut Application) {
    message_box_ok(app, "Backup finished.");
    if confirmation_box_yes_no(app, "Delete the old backup?") == CM_YES {
        // delete it
    }
    if let Some(name) = input_box(app, "Rename", "~N~ew name", "backup", 40) {
        let _ = name;
    }
}
```

### A dialog of fields, values read back

Place the controls inside the frame, keep the handles, and read the values
after `execute`.

```rust,no_run
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_OK};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::GroupLike;
use turbo_vision::views::button::Button;
use turbo_vision::views::checkbox::CheckBox;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;

fn search_dialog(app: &mut Application) -> Option<(String, bool)> {
    let mut dialog = Dialog::new(Rect::new(0, 0, 46, 11), "Search");
    // Children are placed inside the frame: (0, 0) is the inner corner.
    dialog.add(Label::new(Rect::new(2, 1, 10, 2), "~T~ext"));
    let text = dialog.add_typed(InputLine::new(Rect::new(10, 1, 42, 2), 80));
    let case = dialog.add_typed(CheckBox::new(Rect::new(10, 3, 42, 4), "~M~atch case"));
    dialog.add(Button::new(Rect::new(8, 6, 20, 8), "~O~K", CM_OK, true));
    dialog.add(Button::new(Rect::new(24, 6, 36, 8), "Cancel", CM_CANCEL, false));
    if dialog.execute(app) != CM_OK {
        return None;
    }
    let text = dialog.get(text)?.text().to_string();
    let case = dialog.get(case)?.is_checked();
    Some((text, case))
}
```

### Open a file

```rust,no_run
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::file_dialog::FileDialog;

fn open(app: &mut Application) -> Option<std::path::PathBuf> {
    FileDialog::new(Rect::new(5, 2, 65, 20), "Open", "*.txt", None)
        .build()
        .execute(app)
}
```

### A window on the desktop, reached again later

```rust,no_run
use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::state::Grow;
use turbo_vision::views::{GroupLike, Handle, View};
use turbo_vision::views::text_viewer::TextViewer;
use turbo_vision::views::window::Window;

struct LogWindow {
    window: Handle<Window>,
    viewer: Handle<TextViewer>,
}

fn open_log(app: &mut Application) -> LogWindow {
    let mut window = Window::new(Rect::new(2, 1, 62, 16), "Log");
    // Children are placed inside the frame: (0, 0) is the inner corner.
    let mut viewer = TextViewer::new(Rect::new(0, 0, 58, 13)).with_scrollbars(true);
    viewer.set_grow_mode(Grow::HI_X | Grow::HI_Y); // stretch with the window
    let viewer = window.add_typed(viewer);
    let window = app.desktop.add_typed(window);
    LogWindow { window, viewer }
}

fn show(app: &mut Application, log: &LogWindow, text: &str) {
    if let Some(window) = app.desktop.get_mut(log.window) {
        if let Some(viewer) = window.get_mut(log.viewer) {
            viewer.set_text(text);
        }
    }
}
```

If the user closed the window, `get_mut` returns `None`. `AppHandler::window_closed`
tells you when that happens.

### Turn commands on and off

```rust,no_run
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_SAVE;

fn document_changed(app: &mut Application, unsaved: bool) {
    if unsaved {
        app.enable_command(CM_SAVE);
    } else {
        app.disable_command(CM_SAVE); // greys out every Save item and button
    }
}
```

### Do something periodically

`idle` runs whenever no event is waiting, many times a second. Keep it quick,
and rate-limit the work yourself:

```rust,no_run
use std::time::{Duration, Instant};
use turbo_vision::app::{AppHandler, Application};

struct Poller {
    last: Instant,
}

impl AppHandler for Poller {
    fn idle(&mut self, _app: &mut Application) {
        if self.last.elapsed() >= Duration::from_secs(5) {
            self.last = Instant::now();
            // check the queue, refresh a view, ...
        }
    }
}
```

### Follow a terminal resize

Give every view sized from the terminal its grow bits when you create it;
then rebuild, in `idle`, whatever depends on the exact size:

```rust,no_run
use turbo_vision::app::{AppHandler, Application};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::state::Grow;
use turbo_vision::views::text_viewer::TextViewer;
use turbo_vision::views::window::Window;
use turbo_vision::views::{GroupLike, View};

/// A side list that keeps its width; the rest is rebuilt for a new size.
struct Screen {
    laid_out: (i16, i16),
}

fn desktop_size(app: &Application) -> (i16, i16) {
    let b = app.desktop.get_bounds();
    (b.width(), b.height())
}

impl Screen {
    fn open(app: &mut Application) -> Self {
        let (_, height) = desktop_size(app);
        let mut side = Window::new(Rect::new(0, 0, 20, height), "Items");
        side.set_grow_mode(Grow::HI_Y); // same width, new height
        let mut list = TextViewer::new(Rect::new(0, 0, 18, height - 2));
        list.set_grow_mode(Grow::HI_Y); // and its content too
        side.add(list);
        app.desktop.add(side);
        Self { laid_out: desktop_size(app) }
    }
}

impl AppHandler for Screen {
    fn idle(&mut self, app: &mut Application) {
        let now = desktop_size(app);
        if now != self.laid_out {
            self.laid_out = now;
            // re-wrap text, recompute column widths, rebuild a panel, ...
        }
    }
}
```

### React to a control while a dialog is open

A control's change notification (`set_on_change(cmd)` on `Spinner`, `Slider`,
`ComboBox`, `RadioButtons`, `CheckBoxes`) is a **broadcast to the views of the
same window**. It does not reach your `AppHandler`. To react to it, write a
small view that listens for that broadcast (next recipe) and add it to the
same window. Or read the values when the dialog closes, or when a button sends
its command.

### Your own view

Implement `View`. Draw rows with `DrawBuffer` and `write_line_to_terminal`, and
take colours from a palette by role:

```rust
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{Event, EventType};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Palette, STATIC_TEXT_NORMAL, palettes};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::{View, ViewCore, write_line_to_terminal};

/// Shows how many times a broadcast command arrived.
pub struct Counter {
    core: ViewCore,
    command: u16,
    count: u32,
}

impl Counter {
    pub fn new(bounds: Rect, command: u16) -> Self {
        Self { core: ViewCore::new(bounds), command, count: 0 }
    }
}

impl View for Counter {
    fn core(&self) -> &ViewCore {
        &self.core
    }
    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }
    fn draw(&mut self, terminal: &mut Terminal) {
        let width = usize::try_from(self.bounds().width()).unwrap_or(0);
        let attr = self.map_color(STATIC_TEXT_NORMAL);
        let mut line = DrawBuffer::new(width);
        line.move_char(0, ' ', attr, width);
        line.move_str(0, &format!("Changed {} times", self.count), attr);
        write_line_to_terminal(terminal, 0, 0, &line); // (0, 0) is this view's corner
    }
    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::Broadcast && event.command == self.command {
            self.count += 1; // do not clear a broadcast: other views need it too
        }
    }
    fn get_palette(&self) -> Option<Palette> {
        Some(Palette::from_slice(palettes::CP_STATIC_TEXT))
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
```

### Translate the application

Write the texts in Fluent (`.ftl`) files, one per language, add them, choose
the locale, and only then build the menus and dialogs. Views keep the text
they were built with. Turbo Vision's own buttons and dialogs (OK, Cancel, the
file dialog, message boxes, help) follow the same locale.

```rust,no_run
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::menu_data::{Menu, MenuItemBuilder};
use turbo_vision::core::geometry::Rect;
use turbo_vision::tr;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::msgbox::message_box_ok;

const EN: &str = "
menu-file = ~F~ile
menu-exit = E~x~it
files-saved = { $count ->
    [one] One file saved.
   *[other] { $count } files saved.
}
";
const CA: &str = "
menu-file = ~F~itxer
menu-exit = ~S~urt
files-saved = { $count ->
    [one] S'ha desat un fitxer.
   *[other] S'han desat { $count } fitxers.
}
";

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    app.add_translations("en", EN).expect("valid Fluent");
    app.add_translations("ca", CA).expect("valid Fluent");
    app.set_locale_from_env(); // LC_ALL, LC_MESSAGES, LANG; English if unset

    let (width, _) = app.terminal.size();
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width, 1));
    menu_bar.add_submenu(SubMenu::new(
        &tr!("menu-file"),
        Menu::from_items(vec![
            MenuItemBuilder::new().text(tr!("menu-exit")).command(CM_QUIT).key("Alt+X").build(),
        ]),
    ));
    app.set_menu_bar(menu_bar);
    message_box_ok(&mut app, &tr!("files-saved", count = 3));
    Ok(())
}
```

- A missing message falls back along the chain: `ca-ES` → `ca` → `en`, and
  last to the id itself, so a half-translated file still works.
- Fluent's `{ $count -> [one] … *[other] … }` handles plurals. Do not build
  sentences from pieces with `format!`, because word order differs between
  languages.
- `app.set_help_file("help.md")` loads `help.ca.md` when the locale is
  Catalan and that file exists.
- **Text made away from the screen is a `LazyMessage`.** `tr!` translates
  now, with the calling thread's catalogue. A worker thread's catalogue has
  none of your messages (it returns the id) and is in English. So a worker,
  or code that runs before the locale is chosen, or a value kept and shown
  again after a language switch, makes `tr_lazy!("id", name = value)`: the
  same arguments as `tr!`, translated when the UI thread displays it
  (`message.to_string()`). Send `LazyMessage`s, not `String`s, from worker
  threads. An argument can be another `LazyMessage`; `&str` arguments must
  be `'static` (pass a `String` otherwise).

  ```rust
  use turbo_vision::i18n::LazyMessage;
  use turbo_vision::tr_lazy;

  // On the worker: no translation yet.
  let report: LazyMessage = std::thread::spawn(|| tr_lazy!("button-cancel"))
      .join()
      .unwrap();
  // On the UI thread, when shown: in the current locale.
  assert_eq!(report.to_string(), "~C~ancel");
  ```
- `add_translations` can also replace a built-in message, for example
  `button-ok = ~A~ccept`. The ids are in `locales/en/turbo-vision.ftl`.
- A language the crate does not ship (German, say) still works: your
  messages are in it and the built-in dialogs fall back to English. To have
  those in German too, translate a copy of `locales/en/turbo-vision.ftl` and
  add it with `app.add_translations("de", ..)`, next to your own messages.
- `cargo run --example i18n` shows the language switching while the program
  runs. The whole story, including adding a language to the crate, is in
  [Chapter 19](docs/user-guide/Chapter-19-Localization.md) of the user guide.

### Test a screen without a terminal

With the `test-util` feature (`turbo-vision = { version = "4", features =
["test-util"] }` under `[dev-dependencies]`), an application runs on a fake
terminal. You inject keys and read the screen back:

```rust,ignore
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_OK;
use turbo_vision::core::event::{Event, KB_ENTER};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::{Button, set_press_animation};
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::GroupLike;

#[test]
fn enter_presses_ok() {
    set_press_animation(std::time::Duration::ZERO); // no 100 ms key animation in tests
    let mut app = Application::with_terminal(turbo_vision::test_util::test_terminal(80, 25));
    let mut dialog = Dialog::new(Rect::new(10, 5, 40, 12), "Test");
    dialog.add(Button::new(Rect::new(10, 3, 20, 5), "~O~K", CM_OK, true));
    let keys = app.terminal.event_injector();
    keys.send(Event::keyboard(KB_ENTER)).unwrap();
    assert_eq!(dialog.execute(&mut app), CM_OK);
    // app.terminal.read_cell(x, y) returns what was drawn.
}
```

## 5. Rules and pitfalls

1. **Use `run_with`, never your own loop.** A hand-written loop has to
   translate every mouse position into each view's coordinates
   (`dispatch_to_child`), draw, run `idle` and remove closed windows. Forgetting
   one breaks clicks or animations silently.
2. **Your commands start at `CM_USER` (200).** Numbers 100–199 belong to the
   crate, and 0–99 are Borland's standard commands.
3. **One `~` letter per label, menu, status item and button, unique within its
   dialog or menu.** Two views with the same Alt+letter: the first one wins and
   the other can't be reached by keyboard. Alt+X, F1 and F10 are global. This
   holds in every language: a translator picks the letters again, and they
   must not clash in the translated dialog either.
4. **A button's plain letter is typed into a focused input line.** Alt+letter
   always reaches the button.
5. **A dialog's children are relative to the area inside the frame.** Link each
   `Label` to its field so its Alt+letter focuses it.
6. **Reach children with handles.** `add_typed` returns a `Handle<T>`; keep it,
   then use `get` / `get_mut`. Do not keep a `ViewId` and downcast unless the
   type is unknown.
7. **Never clear a broadcast you only observe** (`CM_IDLE_TICK`, change
   notifications): it stops travelling to the other views.
8. **Keyboard button presses are delayed 100 ms** so the user sees the button
   go down. The command arrives a moment later, as a queued event. In tests,
   `button::set_press_animation(Duration::ZERO)`.
9. **`ComboBox` ids must be unique among live combo boxes.**
10. **Esc, the close box and Cancel all return `CM_CANCEL`** from `execute`.
11. **Modal dialogs block.** Inside `execute`, your `idle` handler does not run.
    Use `dialog.set_auto_dismiss(timeout, command)` or the tick closure of
    `Application::execute_modal` for timed behaviour.
12. **Draw only in `draw`.** Views draw themselves when the application
    redraws; change their state and let them draw. `app.needs_redraw()`
    forces a full redraw.
13. **Grow modes decide resizing.** A child with no grow bits stays put when its
    owner resizes. `Grow::HI_X | Grow::HI_Y` fills (it stretches with the owner),
    `Grow::HI_X` stretches to the right. `Grow::ALL` moves every edge, so the
    view keeps its size and travels with the bottom-right corner.
14. **The terminal can be resized at any moment, and your layout must follow.**
    This is the rule most often forgotten. The application moves the menu bar,
    status line and desktop for you; every other view follows only through its
    grow bits. So whatever you size from `app.terminal.size()` or the
    desktop's bounds (a window that fills the desktop, a side list, the views
    inside them) needs grow bits: `Grow::HI_X | Grow::HI_Y` to stretch with
    the desktop, `Grow::HI_Y` to keep its width and take the new height.
    Windows default to `HI_X | HI_Y`; most views default to none. A layout
    that grow bits cannot express (text wrapped to a width, columns computed
    from it) is rebuilt: compare the desktop's size in `idle` with the size
    you laid out for (recipe "Follow a terminal resize"). Test it: a backend
    whose size you change, then `app.step(&mut handler, None)`.
15. **Translated text is longer.** German or Catalan text can be half as long
    again as the English. Size buttons and labels from their text
    (`button::width_for_title`), not from the English width, and choose the
    locale before you build the views.
16. **Translate on the UI thread, when the text is shown.** The catalogue
    is per thread: `tr!` on a worker thread gives English and ids. Text made
    on a worker, before the locale is chosen, or kept across a change of
    language travels as a `LazyMessage` (`tr_lazy!`) and becomes a `String`
    on the UI thread. A view keeps the `String` it was given, so after a
    language switch set its text again from the `LazyMessage`.

## 6. Working on the crate itself

- **Build and test:** `cargo build`, `cargo test` (unit, integration and
  doctests, including this file), and
  `cargo clippy --workspace --all-targets -- -D warnings` (pedantic lints are
  on; the tree is clean, so any warning fails).
- **Format with `cargo fmt`:** the tree is fully formatted, and
  `cargo fmt --check` must pass.
- **Faithful to Borland:** behaviour follows Borland Turbo Vision (and
  magiblot's tvision where it improves on it). Comments cite the original
  (`Matches Borland: TButton::handleEvent`).
- **New views** follow the established shape: a struct holding `ViewCore`, an
  `impl View`, a `…Builder` with `build()`, tests in the same file, a
  `pub mod` line and a listing in `src/views/mod.rs`. Then add the view to
  this file's component index and to `docs/RUST-API-CATALOG.md`:
  `tests/docs_index.rs` fails until you do.
- **Tests that drive the UI** use `test_util::test_terminal` and
  `Application::with_terminal`, inject keys with
  `app.terminal.event_injector()`, and read cells with
  `terminal.read_cell(x, y)`.
- **Text the user sees goes through `i18n::tr`.** Give it an id in
  `locales/en/turbo-vision.ftl`, add the same id to every other locale file
  (`es`, `ca`), and add it to the hot-key groups in `src/i18n/mod.rs` when it
  has a `~` letter in a dialog. The tests there check that every locale has
  exactly the English ids and that no dialog's hot keys clash. Log messages,
  panics and `Debug` output stay in English. Text the crate makes before it
  is shown, or on another thread, is a `LazyMessage`, resolved when drawn
  or shown.
- **A new built-in language** is a translated copy of
  `locales/en/turbo-vision.ftl` in `locales/<tag>/` and one line in
  `BUILT_IN` (`src/i18n/mod.rs`). The tests find it there and check its ids,
  hot keys, fixed areas, and that no dialog's controls overlap in it. The
  steps are in Chapter 19, "Adding a new language".
- **Record every change** in `CHANGELOG.md` under `[Unreleased]`, and in its
  copy `website/docs/reference/changelog.md`. `python3 website/sync_docs.py`
  copies `docs/` into the website.

## 7. Where to read more

| Document | What it covers |
|----------|----------------|
| [`docs/RUST-API-CATALOG.md`](docs/RUST-API-CATALOG.md) | Every public type and method, by module. |
| [`docs/user-guide/`](docs/user-guide/) | The 19-chapter user guide (Borland's, ported, plus localization): application, events, views, windows, dialogs, controls, validation, editors, help, palettes, translations. |
| [`docs/user-guide/Chapter-19-Localization.md`](docs/user-guide/Chapter-19-Localization.md) | Translating an application with Fluent, changing the language at run time, hot keys and room in other languages, adding a language to the crate. |
| [`docs/OWNER-COORDINATES.md`](docs/OWNER-COORDINATES.md) | How owner-relative coordinates work. |
| [`docs/PALETTE-SYSTEM.md`](docs/PALETTE-SYSTEM.md) | How colours are looked up by role. |
| [`docs/MORE-CONTROLS.md`](docs/MORE-CONTROLS.md) | What was added beyond Borland, and the roadmap. |
| [`examples/gallery/`](examples/gallery/) | The component gallery (`cargo run --example gallery`): each component live, grouped by kind, with how it works, its parameters, links to related components, and the code that built it. Each demo in `examples/gallery/demos/` is a short, idiomatic file to copy from. |
| [`examples/`](examples/) | `table_frozen` (window + table + `AppHandler`), `showcase`, `new_controls`, `file_dialog`, `help`, ... |
| [`docs/DESIGN-SYSTEM-PLAN.md`](docs/DESIGN-SYSTEM-PLAN.md) | The design system plan and its decision log. |
