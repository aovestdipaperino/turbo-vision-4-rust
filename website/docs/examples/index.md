# Examples

The repository ships 39 runnable examples under `examples/`. They are not decoration: building them
all is how the crate checks that its public API still works from outside, so any change that breaks
a downstream program breaks the example build first.

```bash
cargo run --example showcase     # run one
cargo build --examples           # build all 39
```

!!! warning "Quit with Alt+X"
    These are full-screen terminal applications. Leave through the application, not by closing the
    terminal window, or you will be left in raw mode.

## Start here

| Example | Lines | What it shows |
|---|---:|---|
| `quick_start_00` .. `quick_start_05` | 10-130 | The application shell, assembled one piece at a time. Walked through in [the quick start tutorial](../tutorials/quick-start.md). |
| `minimal_app` | 80 | A status line and a window, no menu bar. The equivalent of deriving from `TProgram` instead of `TApplication`. |
| `dialogs` | 64 | The three standard library dialogs. |
| `showcase` | 1785 | The broad demo: calculator, calendar, ASCII table and puzzle windows, overlapping with shadows and z-order. |

![Three windows open at once in the showcase demo, overlapping with shadows](../assets/captures/showcase.png)

## By subject

### Application shell

- **`menu_status`** &mdash; a menu bar with submenus, a right-click context menu, and a status line with hot spots and hints.
- **`command_set`** &mdash; buttons that enable and disable themselves as application state changes, driven by the global command set.
- **`function_keys`** &mdash; F1 through F10, and what actually arrives from the terminal.
- **`dynamic_title`** &mdash; changing a window title while the application runs.

![The File menu open over the desktop, its items showing their key chords](../assets/captures/menu_status.png)

### Windows and layout

- **`window_resize`** &mdash; dragging, resizing and the growth modes that decide how children follow.
- **`test_window_overlap`** &mdash; modal against non-modal, and what a modal window blocks.
- **`broadcast`** &mdash; a standalone group distributing a broadcast to its children.
- **`wrong_owner`** &mdash; a deliberate mistake: a button that reports the wrong owner type and picks up the wrong palette. Useful when your colours look off.

### Controls

- **`new_controls`** &mdash; the widgets added beyond the Borland set, on two tabbed pages.
- **`table_frozen`** &mdash; a `Table` with frozen panes: a Region column and a totals row stay put while thirty regions by twelve months scroll both ways. The table grows with its window, and Enter on a cell shows its details through the table's `on_select` command and an `AppHandler`.
- **`cluster_tooltip`** &mdash; multi-item check box and radio clusters, tooltips, and the frame zoom triangle.
- **`list_components`** &mdash; the list viewer family end to end.
- **`sorted_listbox`** &mdash; a sorted list with binary search and type-ahead.
- **`tree_view`** &mdash; a hierarchical outline view.
- **`progress_bar`** &mdash; determinate and marquee bars, animated as overlay widgets.
- **`label_link`** &mdash; labels that focus the input field they are linked to.
- **`validator`** &mdash; every validator type in one dialog: filter, range and picture.

![Check box and radio clusters side by side, over a window with a zoom icon](../assets/captures/cluster_tooltip.png)

![A list box of items beside the instructions pane of the list components demo](../assets/captures/list_components.png)

### Files and the editor

- **`file_dialog`** &mdash; the open and save dialogs.
- **`file_browser`** &mdash; a directory tree beside a file list.

![The Open File dialog listing a directory, with Open and Cancel buttons](../assets/captures/file_dialog.png)
- **`pascal_ide`** &mdash; the editor from the Bruto-Pascal project, with a breakpoint gutter and Pascal syntax highlighting. The most complex example in the tree.

![The Pascal IDE example with a highlighted source file and a breakpoint gutter](../assets/captures/pascal_ide.png)

### Colour and text

- **`palette_themes`** &mdash; replacing the application palette at runtime.
- **`text_styling`** &mdash; a table of every text style the terminal layer can emit.

Real images in the terminal through the Kitty graphics protocol, and ANSI-art backgrounds, moved to
[tv-extensions](https://github.com/aovestdipaperino/tv-extensions)' `examples/` (feature `graphics`).

![The palette themes demo after switching to the Solarized theme, every control recoloured](../assets/captures/palette_themes.png)

### Terminal and plumbing

- **`suspend_resume`** &mdash; handing the terminal back to the shell and taking it again.
- **`screenshot`** &mdash; the built-in screen capture shortcuts.
- **`beep`** &mdash; audio feedback.
- **`test_events`**, **`test_mouse`** &mdash; diagnostic tools that print every event, for when a key or a click is not arriving.

A scrolling output viewer fed by a simulated build log, `tracing` output routed into a scrollable
window, and serving the application over SSH all moved to
[tv-extensions](https://github.com/aovestdipaperino/tv-extensions)' `examples/` (features `log`
and `ssh`).

### Whole applications

- **`biorhythm`** &mdash; the calculator built in [the tutorial](../tutorials/biorhythm.md): a validated date form and a custom chart view.
- **`ui_features`** &mdash; beeps, dynamic titles and message boxes together.

![The biorhythm calculator showing three coloured sine curves and a marker for today](../assets/captures/biorhythm.png)

## Reading an example

Four of them are picked apart in [the walkthroughs](walkthroughs.md).
