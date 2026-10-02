# Turbo Vision Examples

39 examples, one file each. Run any of them with `cargo run --example <name>`;
`cargo build --examples` builds them all. Two need the `screenshot` feature,
which is on by default, so the plain command works unless you build with
`--no-default-features`. Each description below comes from the example's
own header comment.

## Getting started

| Example | Shows | Run |
|---|---|---|
| `quick_start.rs` | The dialog-and-button program from the top-level README | `cargo run --example quick_start` |
| `quick_start_00.rs` | The bare minimum app; Alt+X exits | `cargo run --example quick_start_00` |
| `quick_start_01.rs` | A status line added to the desktop | `cargo run --example quick_start_01` |
| `quick_start_02.rs` | The status line code moved into a function | `cargo run --example quick_start_02` |
| `quick_start_03.rs` | A menu bar, with no actions behind it yet | `cargo run --example quick_start_03` |
| `quick_start_04.rs` | Actions behind the menu bar items | `cargo run --example quick_start_04` |
| `quick_start_05.rs` | Global shortcuts (press F1 to see one work) | `cargo run --example quick_start_05` |
| `minimal_app.rs` | A stripped-down application, like deriving from TProgram instead of TApplication in Borland Turbo Vision | `cargo run --example minimal_app` |

## Controls

| Example | Shows | Run |
|---|---|---|
| `cluster_tooltip.rs` | CheckBoxes and RadioButtons clusters, a per-dialog Tooltip, and the frame zoom icon | `cargo run --example cluster_tooltip` |
| `new_controls.rs` | The widgets added beyond the Borland set: TabbedPane, ComboBox, Spinner, ProgressBar and Table | `cargo run --example new_controls` |
| `table_frozen.rs` | Table with frozen panes: a frozen Region column and a frozen totals row while the rest scrolls both ways; Enter or a double-click shows a cell's region, month and sales via `on_select` and an `AppHandler` | `cargo run --example table_frozen` |
| `progress_bar.rs` | ProgressBar: determinate, marquee, and the percentage toggle | `cargo run --example progress_bar` |
| `list_components.rs` | ListBox with the ListViewer trait, MenuBar with MenuViewer, a MenuBox popup, MenuBuilder | `cargo run --example list_components` |
| `sorted_listbox.rs` | SortedListBox: automatic sorting, binary and prefix search, case-sensitive or not | `cargo run --example sorted_listbox` |
| `tree_view.rs` | A hierarchical tree view (outline) | `cargo run --example tree_view` |
| `label_link.rs` | Clicking a label to focus its linked input field | `cargo run --example label_link` |
| `validator.rs` | Every validator: FilterValidator, RangeValidator, PictureValidator | `cargo run --example validator` |
| `text_styling.rs` | Bold, italic, underline, reverse, dim and strikethrough, printed through the framework's SGR path | `cargo run --example text_styling` |

## Dialogs, menus and windows

| Example | Shows | Run |
|---|---|---|
| `dialogs.rs` | The standard dialogs: MessageBox, ConfirmationBox, InputBox | `cargo run --example dialogs` |
| `file_dialog.rs` | FileDialog: mouse and keyboard selection, folder navigation, wildcards, custom button labels | `cargo run --example file_dialog` |
| `file_browser.rs` | FileList and DirListBox side by side | `cargo run --example file_browser` |
| `dynamic_title.rs` | Changing a dialog's title at runtime | `cargo run --example dynamic_title` |
| `ui_features.rs` | Beep, dynamic titles and message boxes | `cargo run --example ui_features` |
| `beep.rs` | Terminal beep as audio feedback | `cargo run --example beep` |
| `menu_status.rs` | Menu bar with submenus, a right-click popup menu, global shortcuts, event handling patterns | `cargo run --example menu_status` |
| `command_set.rs` | Buttons enabled and disabled from the command set, as in Borland Turbo Vision | `cargo run --example command_set` |
| `broadcast.rs` | `Group::broadcast()` skipping its owner: every sibling counts the broadcast but the clicked button | `cargo run --example broadcast` |
| `window_resize.rs` | Dragging and resizing several windows, and right-aligned menu shortcuts | `cargo run --example window_resize` |
| `test_window_overlap.rs` | Non-modal windows coming to the front, and modal dialogs blocking them | `cargo run --example test_window_overlap` |
| `palette_themes.rs` | App-level palette themes: Default, Dark, High-Contrast, Solarized | `cargo run --example palette_themes` |
| `wrong_owner.rs` | Wrong palette: a Button with an incorrect `owner_type` in a Window | `cargo run --example wrong_owner` |
| `suspend_resume.rs` | Suspending and resuming the terminal | `cargo run --example suspend_resume` |

## Editor and help

| Example | Shows | Run |
|---|---|---|
| `pascal_ide.rs` | The Bruto-Pascal IDE editor: Pascal syntax highlighting, menu bar and status line, a sample program, and F1 context-sensitive help from a `HelpFile` | `cargo run --example pascal_ide` |

## Capture and debugging

| Example | Shows | Run |
|---|---|---|
| `screenshot.rs` | F12 writes an ANSI dump of the screen, Ctrl+F12 a PNG screenshot (features `native`, `screenshot`) | `cargo run --example screenshot --features native,screenshot` |
| `glyph_sample.rs` | Every non-ASCII glyph the PNG renderer draws, written to `target/glyph-sample.png` (feature `screenshot`) | `cargo run --example glyph_sample --features screenshot` |
| `function_keys.rs` | F1-F10 key detection with a live display | `cargo run --example function_keys` |
| `test_events.rs` | Prints and logs every event, to diagnose mouse and keyboard issues | `cargo run --example test_events` |
| `test_mouse.rs` | Prints mouse events, to check mouse capture works | `cargo run --example test_mouse` |

## Demos

| Example | Shows | Run |
|---|---|---|
| `showcase.rs` | The full feature demo, a port of the classic Borland TV demo application | `cargo run --example showcase` |
| `biorhythm.rs` | A biorhythm calculator with semi-graphical charts | `cargo run --example biorhythm` |

## Moved to tv-extensions

The SSH server, Kitty image, ANSI-art background, log window and remote-input
examples moved with their features to the
[tv-extensions](https://github.com/aovestdipaperino/tv-extensions) crate in 4.0.
See [UPGRADING-TO-4.0.md](../UPGRADING-TO-4.0.md).
