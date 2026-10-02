# What's new

The full history is in the [changelog](reference/changelog.md). This page tracks the recent work
in prose, newest first. If you are moving a project onto 4.0.0, work through
[the 4.0 upgrade guide](reference/upgrading-4.0.md) instead: it is the same ground as an ordered
checklist. Moving from 3.0.0 instead uses [the 3.0 upgrade guide](reference/upgrading.md).

## Unreleased

Not in a release yet &mdash; coming in the next release. The crate on crates.io is still 4.0.2.

`Table` can freeze panes, like a spreadsheet. `set_frozen_cols(n)` keeps the first `n` columns
at the left edge while the others scroll sideways, with a `║` marking the edge in every line;
`set_frozen_rows(n)` keeps the first `n` rows under the header while the others scroll, the
last of them underlined. Frozen columns read as row labels, in the header's colour; frozen rows
stay ordinary rows you can focus and select, and they work with a lazy `RowProvider`. Both
default to zero, so existing tables are unchanged. The `table_frozen` example freezes a region
column and a totals row over thirty regions by twelve months.

## 4.0.2 &mdash; October 2026

The text cursor no longer shows through menus, combo-box drop-downs and history lists: a popup hides
it while it owns the input and puts it back when it closes. The cursor is now sent once per frame,
after the changed cells, so it never flickers at its new position over the old screen, and raw
output (`Terminal::write_raw`, used for Kitty images) leaves it where it was. `show_cursor` and
`hide_cursor` take effect on the next `flush`: a loop that drives the terminal itself sets the
cursor before flushing, and a host-driven embedder flushes after its final draw or reads
`Terminal::cursor`.

## 4.0.1 &mdash; October 2026

The text cursor now shows in a focused InputLine, Memo or Editor, inside windows, dialogs, tab pages
and split panes, and stays put on a real terminal and over SSH (it is put back after each redraw).
Also fixed: help links are clickable after scrolling sideways, every link on a help line reaches
"See also", and the byte-stream input parser used by SSH and WASM hosts keeps Shift/Alt/Ctrl on
F-keys, arrows and editing keys. PNG screenshots now draw the whole box-drawing block (heavy, dashed,
rounded, diagonal and half lines).

## 4.0.0 &mdash; October 2026

SSH, remote input, Kitty/ANSI graphics, the log window and terminal widget, and the
`turbo-vision-extras` crate all leave core for a new crate,
[tv-extensions](https://github.com/aovestdipaperino/tv-extensions), which versions on its own pace
so core can stay small and stable. Apart from the removed methods listed below, nothing about
`View`, `Application` or `Terminal` itself changed shape — every item either moved verbatim or was
folded into a core control that already existed (`ComboBox`, `Spinner`, `TabbedPane`, `ProgressBar`,
`Slider`; `GridView` becomes `Table` + `RowProvider`, `VirtualListBox` becomes `ListBox` +
`ListProvider`). [The 4.0 upgrade guide](reference/upgrading-4.0.md) is the ordered list of what to
change, with the exact `use` line for every moved item.

## 3.1.0 &mdash; October 2026

A new `Slider` view: a horizontal track with a thumb that picks one integer in a range, moved
with Left/Right/Home/End or by clicking and dragging (the drag keeps tracking when the pointer
leaves the track).

`Table` can draw a `│` separator between columns (`set_separators`, off by default), and both
`Table` and `ListBox` can now read their rows or items from a lazy source instead of holding
everything in memory: a `table::RowProvider` or `listbox::ListProvider`, installed with
`set_provider` and refreshed with `refresh_rows` / `refresh_items` when the underlying data's
length changes.

The terminal layer gained extension hooks: `event_injector` for synthetic events, a capture hook
(`set_capture_hook`, `clear_capture_hook`, `run_capture_hook`, `CaptureKind`) that Ctrl+F12, F12 and
`CM_SCREENSHOT` now call instead of the built-in capture when one is set, and `write_raw` for bytes
the terminal layer does not otherwise emit. `terminal::InputParser` no longer needs the `ssh`
feature.

Screen capture stays in core as a debug facility. F12 (ANSI dump) is always built; Ctrl+F12 PNG
screenshots now sit behind a `screenshot` cargo feature, on by default. A crate that depends with
`default-features = false` adds `features = ["screenshot"]` to keep them. The PNG renderer also
draws far more characters instead of `?`: the framework's own marks (check marks, arrows,
message-box icons, partial progress blocks), the whole CP437 set, and Latin-1 accented letters.

Fixes: the ComboBox drop-down opens under its field again (#112); "See also" links in help land
under their text and show topic titles (#110); and Borland's default window keys work (#111):
Alt+F3 closes the selected window, F5 zooms, Ctrl+F5 resizes, F6 / Shift+F6 cycle windows. The
selected window now enables its own window commands, so menu and status-line items for Close,
Zoom and Next follow it.

**Upgrade note:** because a row or item can now come from a provider, `Table::selected_cell` and
`ListBox::get_selected_item` return owned strings (`Option<String>`) and `ListBox::marked_text`
returns `Vec<String>`, instead of borrowing. Add `.as_deref()` where a `&str` is still needed.

## 3.0.0 &mdash; September 2026

A major release. The `View` trait changed, so every other breaking change rode along with it.
Downstream crates that implement `View` for their own types need the migration described below.
The two documents behind it are [the inheritance analysis](reference/inheritance.md) and
[the coordinate model](reference/owner-coordinates.md).

### Owner-relative coordinates

The headline change. A view's bounds are now relative to its owner, the way Borland's
`TView::origin` always was, and they stay put when the owner moves. A view draws in its own space
from `(0, 0)` to `extent()`, and mouse positions arrive already translated into that space. The
terminal keeps an origin stack beside the clip stack, so every write, read, cursor and clip call
is translated for you.

```rust
// 2.x: bounds were screen coordinates once the view was added
let x = self.bounds().a.x;
write_line_to_terminal(terminal, x, self.bounds().a.y + row, &buf);
if self.bounds().contains(event.mouse.pos) { /* ... */ }

// 3.0.0: draw at the origin, test against your own extent
write_line_to_terminal(terminal, 0, row, &buf);
if self.extent().contains(event.mouse.pos) { /* ... */ }
```

A view that holds children by value uses `views::view::draw_child` and
`views::view::dispatch_to_child`, which push and pop the child's origin around the call. Code
outside the tree that draws a top-level view calls `terminal.draw_view(&mut view)`.

### One base, one hook layer

`ViewCore` now holds the fields every view has: bounds, state, options, grow mode and the palette
chain. A view implements `core()` and `core_mut()`, and the ten field accessors become trait
defaults reading that core. A view can no longer forget to report its own state.

Above `View` sit two behaviour traits. `GroupLike` carries `TGroup`'s behaviour as `group_*`
default methods, including the modal loop and typed child access. `WindowLike` does the same for
`TWindow` with `window_*` bodies. A window-shaped type implements the two accessor pairs and gets
its `View` implementation from `impl_view_for_window!`, writing only its overrides inline. The
overrides are late-bound, so a base `window_draw` paints with the derived type's palette, which is
what the C++ virtual call did.

Six hooks left the `View` trait entirely: the two button hooks, the two list selection hooks and
the two end-state hooks. Downcast to `Button` or `ListBox`, or reach the modal end state through
`View::as_group`, the analogue of `dynamic_cast<TGroup*>`.

### Typed flags instead of bare integers

`State`, `Options`, `Grow`, `MsgBox` and `ValidatorOptions` are newtypes with associated constants,
so a state can no longer be passed where options were expected. Write `state.contains(State::MODAL)`
rather than a bitwise test against zero. The old `SF_*`, `OF_*`, `GF_*` and `MF_*` names survive as
deprecated aliases for one release.

### Command numbers have owners

Commands from 0 to 99 are Borland's standard set, 100 to 199 belong to this crate, and applications
start at `CM_USER`, which is 200. The demo-application commands left the library. Alongside this,
the old rule that any command below 1000 closed a dialog is gone, replaced by an explicit `CloseOn`
policy: a modal dialog ends on the standard four commands plus its own buttons by default, and any
other child's command passes through to the caller whatever its number.

### Loops you no longer have to write

`Application::run_with` takes an `AppHandler` with `pre_event`, `handle_command`, `idle` and
`window_closed` hooks, replacing the hand-copied event loops programs used to carry.
`Application::execute_modal` is now the single modal loop behind every dialog, and it dispatches to
the outer type's `handle_event`, so a wrapper reacts to its own children without duplicating the
loop.

### Smaller changes worth knowing

An input line owns its own text, so the `Rc<RefCell<String>>` parameter is gone and you read the
value back through a typed `Handle<InputLine>` after the dialog closes. Menus and status items are
built with key chord strings such as `"Ctrl+O"`, and an unknown chord panics on first run rather
than silently binding nothing. A generic `Shared<T>` replaced five per-type forwarding newtypes.
`add` accepts any view, boxed or not.

### Fixed

A window as large as its desktop was pushed to a negative origin, hiding its top row and left
column. Drag limits now clamp the far edges before the near ones, as `TView::moveGrow` does, so the
top-left corner stays where it belongs.

## 2.4.0 &mdash; September 2026

Ten new controls and three completions: a split pane, multi-select list boxes, scroll bar
auto-repeat, multi-item clusters, tooltips and the frame zoom triangle. See
[More controls](reference/more-controls.md).

## Earlier releases

Version 2.0.0 declared the port production ready at full API parity with the C++ original. The
releases before it built up the widget set, the editor, the palette system and the persistence
layer. All of them are in the [changelog](reference/changelog.md).
