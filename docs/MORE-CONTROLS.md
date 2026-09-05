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
- [ ] **Splitter** — draggable divider resizing two sibling views. The geometry
      maths already exists in `Desktop`'s tiling code.
- [ ] **Tooltip / hint popup** — transient hover text. Borland pushed this to the
      status line; a real popup suits mouse-driven use better.

## Completing existing controls

- [ ] **`CheckBoxes` / `RadioButtons` as true clusters** — Borland's versions hold
      a list of items in one focusable control with a bitmask value. Here
      `CheckBox` holds a single label and grouping is emulated with a
      `CM_RADIO_SELECTED` broadcast on a group id. Works, but diverges from the
      reference and is verbose for multi-item groups.
- [ ] **Multi-select in `ListBox`** — `ListViewer::is_selected` exists but
      `ListBox` exposes only a single `get_selection`. Wants Space to mark,
      Shift-click to extend, and a marked-item accessor.
- [ ] **ScrollBar mouse auto-repeat** — held arrow clicks should repeat. Already
      recorded as an omission in `TO-DO.md`.
- [ ] **Frame zoom-icon rendering** — the zoom command dispatches but the icon is
      never drawn. Also recorded in `TO-DO.md`.

## Demo

`examples/new_controls.rs` runs all five new controls in one dialog. A tabbed
pane holds two pages: the first wires three combo boxes and a spinner to a
progress bar, the second is a table.

## Notes

New views follow the established shape: a struct holding `bounds` and a
`palette_chain`, an `impl View`, a `*Builder` with a fluent API and
`build()` / `build_boxed()`, unit tests in the same file, and a `pub mod` entry
plus doc-comment listing in `src/views/mod.rs`.
