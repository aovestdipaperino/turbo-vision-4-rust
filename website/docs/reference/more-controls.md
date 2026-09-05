# More Controls — gap analysis and roadmap

Survey of the widget set in `src/views/` against both the Borland Turbo Vision
reference and the expectations of a modern text UI toolkit. Work top to bottom
within each section.

## What already exists

Containers: `Group`, `Window`, `Dialog`, `Desktop`, `Frame`, `Background`,
`AnsiBackground`.

Input: `InputLine` (with `Validator`, `PictureValidator`, `LookupValidator`),
`Button`, `CheckBox`, `RadioButton`, `Cluster`, `Editor` / `EditWindow` /
`FileEditor`.

Display: `StaticText`, `Label`, `ParamText`, `Memo`, `TextViewer`, `ListBox`,
`SortedListBox`, `DirListBox`, `FileList`, `Outline` (tree), `Indicator`,
`Scroller`, `ScrollBar`, `KittyImage`, `TerminalWidget`, `LogWindow`.

Chrome and canned dialogs: `MenuBar`, `MenuBox`, `MenuViewer`, `StatusLine`,
`msgbox`, `FileDialog`, `ChDirDialog`, `ColorDialog`, `ColorSelector`,
`HistoryWindow`, the `help_*` family.

## New controls

Ordered by benefit-to-effort. Each one composes existing views wherever it can.

- [x] **ProgressBar / gauge** — done, `src/views/progress_bar.rs`. Determinate
      and marquee modes; `Smooth` / `Blocks` / `Ascii` glyph styles; centred
      percentage overlay that can be toggled off or replaced with fixed text;
      self-animating marquee via `IdleView`; `CP_PROGRESS_BAR` palette reusing
      the scrollbar gauge colours. Demo in `examples/progress_bar.rs`.
      Still open: a compact status-line variant.
- [x] **ComboBox / dropdown list** — done, `src/views/combo_box.rs`. Read-only
      flavour: a field with a drop arrow, F4 or a click drops the list, arrows
      cycle without opening. The popup runs modally through the same two-step
      command the history button uses, since a control cannot reach the terminal
      from `handle_event`. Still open: the editable flavour, where the field is
      a real `InputLine`.
- [x] **Spinner / numeric up-down** — done, `src/views/spinner.rs`. Holds one
      integer inside a range, so typed input is clamped rather than rejected.
      Arrows step, PgUp and PgDn step ten times as far, Home and End jump to the
      ends, digits edit the number, and the steppers are clickable. Optional
      wrap-around and a unit suffix.
- [x] **TabbedPane / notebook** — done, `src/views/tabbed_pane.rs`. Each page is
      a `Group`, so it holds ordinary controls and runs its own focus traversal.
      Drawn as enclosed tab boxes over a framed page, the active tab's floor
      open. F6 and Shift+F6 switch, as do Ctrl+PgUp/PgDn where the terminal
      sends them, tilde hotkeys, and clicking a tab. Tab cycles within the
      active page rather than escaping it.
- [x] **Table / grid view** — done, `src/views/table.rs`. Header row, per-column
      widths and alignment, and a focused cell rather than a focused row: Up and
      Down move rows, Left and Right move columns, and the grid scrolls in both
      directions by whole columns. Ragged rows draw blank instead of panicking.
- [x] **Splitter** — done as `SplitPane`, `src/views/split_pane.rs`. A bare
      divider cannot resize siblings it does not own, so the control owns both
      halves, each a `Group`, the way `TabbedPane` owns its pages. Vertical or
      horizontal, draggable, with a minimum size per half; F8 moves focus between
      them. Keyboard divider movement is left to the host through `grow_first`
      and `shrink_first`, rather than stealing a key from the panes.
- [x] **Tooltip / hint popup** — done, `src/views/tooltip.rs`. One tooltip serves
      a whole dialog: register a rect and a line of text per control, and the
      pointer resting on one raises the hint beside it. Add it last, since it
      draws over its neighbours. The hover delay runs off the new `CM_IDLE_TICK`
      broadcast.

## Completing existing controls

- [x] **`CheckBoxes` / `RadioButtons` as true clusters** — done,
      `src/views/cluster_group.rs`. Each holds its items in one focusable control
      with a single bitmask value, as Borland does. Arrows move within the
      cluster, Space toggles or selects, Tab leaves it, and a tilde-marked letter
      is an item's Alt hotkey; items can be disabled individually. The existing
      one-label `CheckBox` and `RadioButton` are untouched and still supported.
- [x] **Multi-select in `ListBox`** — done. `set_multi_select` adds marks that
      are independent of the focus: Space marks the focused item, Shift+click
      marks a run from the anchor, and `marked_items` / `marked_text` report them
      in list order. Marked rows carry a check glyph in a two-cell column, and
      `is_selected` follows the marks in that mode. Off by default, so existing
      single-selection lists are untouched.
- [x] **ScrollBar mouse auto-repeat** — done. `Application` tracks whether a
      button is held and, only then, broadcasts `CM_MOUSE_AUTO_REPEAT` from its
      idle pass; the scrollbar repeats the press it is holding after a 400 ms
      delay, every 80 ms, and stops at the end of the range. Nothing is sent
      while no button is down, so an idle app stays idle.
- [x] **Frame zoom-icon rendering** — done. A resizable window's title bar now
      carries `[\u{25B2}]` beside the close box, turning into `[\u{25BC}]` once
      zoomed. It tracks press and release like the close box, so a press that
      slides off cancels. Dialogs show none, since Borland pairs wfZoom with
      wfGrow and a dialog has neither.

## Demo

`examples/new_controls.rs` runs all five new controls in one dialog. A tabbed
pane holds two pages: the first wires three combo boxes and a spinner to a
progress bar, the second is a table.

## What is left

Nothing on this list. Two things worth knowing about what shipped:

- `CM_IDLE_TICK`, added for the tooltip's hover delay, is a general timer any
  view can use. `Application::idle` broadcasts it whenever the event poll times
  out. Views must not consume it, since a broadcast stops travelling once it is.
- The clusters take their colours from `CP_CLUSTER`, so they look exactly like
  the existing one-label `CheckBox` and `RadioButton`.

## Notes

New views follow the established shape: a struct holding `bounds` and a
`palette_chain`, an `impl View`, a `*Builder` with a fluent API and
`build()` / `build_boxed()`, unit tests in the same file, and a `pub mod` entry
plus doc-comment listing in `src/views/mod.rs`.
