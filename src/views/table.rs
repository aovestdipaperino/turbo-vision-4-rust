// (C) 2025 - Enzo Lombardi

//! Table view - a scrollable grid with a header row and sized columns.
//!
//! Not part of the Borland Turbo Vision widget set. `ListViewer` carries a
//! `num_cols` field, but that lays one list out in newspaper columns; it has no
//! header, no per-column widths and no column-aware navigation. This is the
//! separate control.
//!
//! Focus is a cell, not a row: Up and Down move between rows, Left and Right
//! between columns, and the grid scrolls in both directions to keep the focused
//! cell on screen.
//!
//! The leading columns can be frozen with [`Table::set_frozen_cols`]: they stay
//! at the left edge while the others scroll past them, like a spreadsheet's
//! frozen panes, and a [`FROZEN_SEPARATOR`] marks where the frozen part ends.
//! Likewise [`Table::set_frozen_rows`] keeps the leading rows under the header
//! while the others scroll up and down; the last frozen row is underlined.
//!
//! Rows come from `set_rows`, or lazily from a [`RowProvider`] given to
//! [`Table::set_provider`], which asks only for the rows on screen.
//!
//! # Keys
//!
//! | Key | Action |
//! |-----|--------|
//! | Up, Down | Previous or next row |
//! | Left, Right | Previous or next column |
//! | PgUp, PgDn | Previous or next screenful of rows |
//! | Home, End | First or last row |
//! | Ctrl+Left, Ctrl+Right | First or last column |
//! | Enter | Emit the selection command |
//!
//! Clicking a cell focuses it; double-clicking emits the selection command too.
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::views::table::{Column, Table};
//! use turbo_vision::core::geometry::Rect;
//!
//! let mut table = Table::new(Rect::new(2, 2, 40, 10), 0);
//! table.set_columns(vec![Column::new("Name", 12), Column::right("Size", 8)]);
//! table.set_rows(vec![vec!["main.rs".into(), "1024".into()]]);
//! assert_eq!(table.selected_row(), Some(0));
//! ```
//!
//! ## Frozen panes
//!
//! A region column and a totals row that stay put while the rest scrolls:
//!
//! ```rust
//! use turbo_vision::views::table::{Column, TableBuilder};
//! use turbo_vision::core::geometry::Rect;
//!
//! let table = TableBuilder::new()
//!     .bounds(Rect::new(1, 2, 69, 17))
//!     .columns(vec![
//!         Column::new("Region", 14),
//!         Column::right("Jan", 6),
//!         Column::right("Feb", 6),
//!     ])
//!     .rows(vec![
//!         vec!["All regions".into(), "870".into(), "940".into()],
//!         vec!["Baleares".into(), "310".into(), "420".into()],
//!     ])
//!     .frozen_cols(1)
//!     .frozen_rows(1)
//!     .build();
//! assert_eq!((table.frozen_cols(), table.frozen_rows()), (1, 1));
//! ```
//!
//! `examples/table_frozen.rs` shows it running, over thirty regions by twelve
//! months.

use super::list_viewer::{ListViewer, ListViewerState};
use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_DOWN, KB_END, KB_ENTER, KB_HOME, KB_LEFT, KB_PGDN, KB_PGUP, KB_RIGHT,
    KB_UP, MB_LEFT_BUTTON,
};
use crate::core::geometry::{Point, Rect};
use crate::core::palette::{
    LISTBOX_DIVIDER, LISTBOX_FOCUSED, LISTBOX_NORMAL, LISTBOX_SELECTED, Style,
};
use crate::core::state::{State, StateFlags};
use crate::terminal::Terminal;

/// Blank cells between two columns.
const COLUMN_GAP: usize = 1;

/// Drawn in each gap between two visible columns when separators are on.
pub const SEPARATOR: char = '│';

/// Drawn in the gap after the last frozen column, whether separators are on
/// or not, so the edge of the frozen part always shows.
pub const FROZEN_SEPARATOR: char = '║';

/// How a cell's text sits inside its column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Against the left edge. The default, and right for text.
    Left,
    /// Against the right edge. Right for numbers.
    Right,
}

/// One column of a [`Table`].
#[derive(Debug, Clone)]
pub struct Column {
    /// Text shown in the header row.
    pub title: String,
    /// Width in cells, not counting the gap to the next column.
    pub width: u16,
    /// How this column's cells are aligned.
    pub align: Align,
}

impl Column {
    /// A left-aligned column.
    pub fn new(title: impl Into<String>, width: u16) -> Self {
        Self {
            title: title.into(),
            width: width.max(1),
            align: Align::Left,
        }
    }

    /// A right-aligned column, for numbers.
    pub fn right(title: impl Into<String>, width: u16) -> Self {
        Self {
            align: Align::Right,
            ..Self::new(title, width)
        }
    }
}

/// A source of rows a [`Table`] reads only as it draws them, so a table can
/// browse more rows than would fit in memory as strings.
///
/// `cell` is only asked for rows below `rows()` and columns the table has.
/// A provider whose length changes must be followed by
/// [`Table::refresh_rows`].
pub trait RowProvider {
    /// How many rows there are.
    fn rows(&self) -> usize;
    /// The text of one cell.
    fn cell(&self, row: usize, col: usize) -> String;
}

/// Where a table's rows come from.
enum Rows {
    /// Rows held by the table, as `set_rows` and `add_row` give them.
    Owned(Vec<Vec<String>>),
    /// Rows read on demand.
    Provided(Box<dyn RowProvider>),
}

impl Rows {
    fn len(&self) -> usize {
        match self {
            Rows::Owned(rows) => rows.len(),
            Rows::Provided(p) => p.rows(),
        }
    }

    /// The cell, or `None` past the end of the row (an owned ragged row) or
    /// of the table.
    fn get(&self, row: usize, col: usize) -> Option<String> {
        match self {
            Rows::Owned(rows) => rows.get(row)?.get(col).cloned(),
            Rows::Provided(p) => (row < p.rows()).then(|| p.cell(row, col)),
        }
    }
}

/// A scrollable grid of rows and sized columns.
pub struct Table {
    core: ViewCore,
    columns: Vec<Column>,
    rows: Rows,
    /// Row focus and vertical scrolling, shared with the other list views.
    list_state: ListViewerState,
    /// Index of the focused column.
    focused_col: usize,
    /// Leftmost visible scrolling column, for grids wider than the view.
    /// Never one of the frozen columns.
    first_col: usize,
    /// Leading columns that stay put while the rest scroll sideways.
    frozen_cols: usize,
    /// Leading rows that stay under the header while the rest scroll.
    frozen_rows: usize,
    /// Whether the header row is drawn.
    show_header: bool,
    /// Whether a `SEPARATOR` is drawn between columns.
    separators: bool,
    /// Command emitted by Enter or a double-click.
    on_select: CommandId,
    view_state: StateFlags,
}

impl Table {
    /// Create an empty table that emits `on_select` when a cell is chosen.
    pub fn new(bounds: Rect, on_select: CommandId) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            columns: Vec::new(),
            rows: Rows::Owned(Vec::new()),
            list_state: ListViewerState::new(),
            focused_col: 0,
            first_col: 0,
            frozen_cols: 0,
            frozen_rows: 0,
            show_header: true,
            separators: false,
            on_select,
            view_state: State::empty(),
        }
    }

    /// Replace the columns. Clamps the focused and leftmost column to the new
    /// set, so a narrower table never points past its own edge.
    pub fn set_columns(&mut self, columns: Vec<Column>) {
        self.columns = columns;
        self.clamp_columns();
    }

    /// The columns, in display order.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Replace the rows. Each row is read positionally against the columns;
    /// missing cells draw blank and extra cells are ignored, so a ragged row
    /// never panics.
    pub fn set_rows(&mut self, rows: Vec<Vec<String>>) {
        self.rows = Rows::Owned(rows);
        self.list_state.set_range(self.rows.len());
        self.scroll_row_into_view();
    }

    /// Append one row. On a provider-backed table this starts a new
    /// in-memory list holding just this row.
    pub fn add_row(&mut self, row: Vec<String>) {
        match &mut self.rows {
            Rows::Owned(rows) => rows.push(row),
            Rows::Provided(_) => self.rows = Rows::Owned(vec![row]),
        }
        self.list_state.set_range(self.rows.len());
    }

    /// Drop every row, keeping the columns.
    pub fn clear_rows(&mut self) {
        self.rows = Rows::Owned(Vec::new());
        self.list_state.set_range(0);
        self.first_col = self.frozen();
    }

    /// Read rows from `provider` instead of an in-memory list. The focus is
    /// kept where it was, clamped to the new length.
    pub fn set_provider(&mut self, provider: Box<dyn RowProvider>) {
        self.rows = Rows::Provided(provider);
        self.refresh_rows();
    }

    /// Re-read the row count, after a provider's source has grown or shrunk.
    pub fn refresh_rows(&mut self) {
        self.list_state.set_range(self.rows.len());
        self.scroll_row_into_view();
    }

    /// Number of rows. Cached from the row source at the last `set_rows`,
    /// `add_row`, `set_provider` or `refresh_rows`.
    pub fn row_count(&self) -> usize {
        self.list_state.range
    }

    /// Index of the focused row.
    pub fn selected_row(&self) -> Option<usize> {
        self.list_state.focused
    }

    /// Index of the focused column.
    pub fn selected_col(&self) -> usize {
        self.focused_col
    }

    /// Text of the focused cell, if there is one.
    pub fn selected_cell(&self) -> Option<String> {
        self.rows.get(self.list_state.focused?, self.focused_col)
    }

    /// Focus a row, clamped to [`row_count`](Self::row_count) (the count as
    /// of the last refresh, not the source's possibly-changed live length).
    pub fn set_selected_row(&mut self, row: usize) {
        if self.row_count() == 0 {
            return;
        }
        let row = row.min(self.row_count() - 1);
        self.focus_row(row);
    }

    /// Focus a column, clamped to the columns that exist.
    pub fn set_selected_col(&mut self, col: usize) {
        if self.columns.is_empty() {
            return;
        }
        self.focused_col = col.min(self.columns.len() - 1);
        self.scroll_col_into_view();
    }

    /// Whether the header row is drawn. On by default.
    pub fn set_show_header(&mut self, show: bool) {
        self.show_header = show;
        self.scroll_row_into_view();
    }

    /// Draw a [`SEPARATOR`] in the one-cell gap between each pair of visible
    /// columns. Off by default. Columns stay where they are, so clicks land
    /// on the same cells either way.
    pub fn set_separators(&mut self, on: bool) {
        self.separators = on;
    }

    /// Whether separators are drawn.
    pub fn separators(&self) -> bool {
        self.separators
    }

    /// Freeze the first `count` columns: they stay at the left edge while
    /// the others scroll sideways, and a [`FROZEN_SEPARATOR`] follows them.
    /// Zero, the default, freezes none. A count past the last column freezes
    /// them all.
    pub fn set_frozen_cols(&mut self, count: usize) {
        self.frozen_cols = count;
        self.clamp_columns();
    }

    /// How many leading columns are frozen, as set (it may exceed the
    /// columns there are).
    pub fn frozen_cols(&self) -> usize {
        self.frozen_cols
    }

    /// Freeze the first `count` rows: they stay under the header while the
    /// others scroll up and down, and the last of them is underlined. Zero,
    /// the default, freezes none. They are ordinary rows otherwise: they can
    /// be focused and selected, and count in [`row_count`](Self::row_count).
    /// A count past the last row freezes them all.
    pub fn set_frozen_rows(&mut self, count: usize) {
        self.frozen_rows = count;
        self.scroll_row_into_view();
    }

    /// How many leading rows are frozen, as set (it may exceed the rows
    /// there are).
    pub fn frozen_rows(&self) -> usize {
        self.frozen_rows
    }

    /// Command emitted by Enter or a double-click.
    pub fn set_on_select(&mut self, command: CommandId) {
        self.on_select = command;
    }

    /// Scrolling rows visible at once: the header and the frozen rows
    /// excluded.
    fn visible_rows(&self) -> usize {
        self.body_height().saturating_sub(self.frozen_row_lines())
    }

    /// Lines below the header.
    fn body_height(&self) -> usize {
        let height = self.core.bounds.height_clamped().max(0) as usize;
        height.saturating_sub(self.header_rows())
    }

    /// Frozen rows that exist: the setting, capped at the row count.
    fn frozen_row_count(&self) -> usize {
        self.frozen_rows.min(self.row_count())
    }

    /// Lines the frozen rows take, clipped to the space under the header.
    fn frozen_row_lines(&self) -> usize {
        self.frozen_row_count().min(self.body_height())
    }

    /// First scrolling row on screen. Scrolling never starts inside the
    /// frozen rows, which are drawn above it anyway.
    fn top_row(&self) -> usize {
        self.list_state.top_item.max(self.frozen_row_count())
    }

    /// Focus a row and scroll the scrolling rows so it shows. A frozen row
    /// is always on screen, so focusing one scrolls nothing.
    fn focus_row(&mut self, row: usize) {
        if row >= self.row_count() {
            return;
        }
        self.list_state.top_item = self.top_row();
        if row < self.frozen_row_count() {
            self.list_state.focused = Some(row);
            return;
        }
        let visible = self.visible_rows();
        self.list_state.focus_item(row, visible);
    }

    /// 1 when the header is drawn, 0 otherwise.
    fn header_rows(&self) -> usize {
        usize::from(self.show_header)
    }

    /// Frozen columns that exist: the setting, capped at the column count.
    fn frozen(&self) -> usize {
        self.frozen_cols.min(self.columns.len())
    }

    /// Cells the frozen columns take, the gap after each included.
    fn frozen_span(&self) -> usize {
        (0..self.frozen()).map(|i| self.column_span(i)).sum()
    }

    /// Keep the column indices inside the current column list, and the
    /// leftmost scrolling column out of the frozen ones.
    fn clamp_columns(&mut self) {
        let frozen = self.frozen();
        if self.columns.is_empty() {
            self.focused_col = 0;
            self.first_col = 0;
            return;
        }
        let last = self.columns.len() - 1;
        self.focused_col = self.focused_col.min(last);
        // With every column frozen there is no scrolling column to start at.
        self.first_col = self.first_col.clamp(frozen, last.max(frozen));
        self.scroll_col_into_view();
    }

    /// Scroll vertically so the focused row is on screen.
    fn scroll_row_into_view(&mut self) {
        if let Some(row) = self.list_state.focused {
            self.focus_row(row);
        }
    }

    /// Scroll horizontally so the focused column is fully visible.
    ///
    /// Columns are whole units: the leftmost visible column advances until the
    /// focused one fits, rather than clipping a column in half. A frozen
    /// column is always visible, so focusing one scrolls nothing; the others
    /// scroll in the width the frozen ones leave.
    fn scroll_col_into_view(&mut self) {
        let frozen = self.frozen();
        self.first_col = self.first_col.max(frozen);
        if self.focused_col < frozen {
            return;
        }
        if self.focused_col < self.first_col {
            self.first_col = self.focused_col;
            return;
        }
        let width =
            (self.core.bounds.width_clamped().max(0) as usize).saturating_sub(self.frozen_span());
        while self.first_col < self.focused_col {
            let span: usize = (self.first_col..=self.focused_col)
                .map(|i| self.column_span(i))
                .sum();
            // The last column on screen needs no trailing gap.
            if span.saturating_sub(COLUMN_GAP) <= width {
                break;
            }
            self.first_col += 1;
        }
    }

    /// Cells one column occupies, its trailing gap included.
    fn column_span(&self, index: usize) -> usize {
        self.columns
            .get(index)
            .map_or(0, |c| c.width as usize + COLUMN_GAP)
    }

    /// The columns as drawn, in order: each one's index, x offset from the
    /// table's left edge, and the cells it is drawn in. The frozen columns
    /// come first, then the scrolling ones from `first_col`. A column that
    /// starts past the right edge is left out and the last one shown is
    /// clipped to the table's width. Drawing, hit-testing and
    /// [`column_offsets`](Self::column_offsets) all read this, so they agree.
    fn layout(&self) -> Vec<(usize, usize, usize)> {
        let width = usize::try_from(self.core.bounds.width_clamped()).unwrap_or(0);
        let frozen = self.frozen();
        let mut columns = Vec::new();
        let mut x = 0;
        for index in (0..frozen).chain(self.first_col.max(frozen)..self.columns.len()) {
            if x >= width {
                break;
            }
            let column_width = self.columns[index].width as usize;
            columns.push((index, x, column_width.min(width - x)));
            x += column_width + COLUMN_GAP;
        }
        columns
    }

    /// Where each visible column lands, in draw order: its x offset from the
    /// table's left edge and the cells it is drawn in.
    ///
    /// Frozen columns come first. Scrolling columns scrolled off to the left
    /// are left out, and so is any that starts past the right edge; the last
    /// one shown is clipped to the table's width, as it is drawn. The
    /// one-cell gap after each column is not part of its width, so a widget
    /// that draws between columns (a separator, a resize handle) finds the
    /// gap at `x + width`.
    #[must_use]
    pub fn column_offsets(&self) -> Vec<(usize, u16)> {
        self.layout()
            .into_iter()
            .map(|(_, x, width)| (x, u16::try_from(width).unwrap_or(u16::MAX)))
            .collect()
    }

    /// Move the focused row by `delta`, clamping at both ends. Clamps to
    /// [`row_count`](Self::row_count), the count as of the last refresh.
    fn move_row(&mut self, delta: i32) {
        if self.row_count() == 0 {
            return;
        }
        let last = self.row_count() as i32 - 1;
        let current = self.list_state.focused.unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, last) as usize;
        self.focus_row(next);
    }

    /// Move the focused column by `delta`, clamping at both ends.
    fn move_col(&mut self, delta: i32) {
        if self.columns.is_empty() {
            return;
        }
        let last = self.columns.len() as i32 - 1;
        let next = (self.focused_col as i32 + delta).clamp(0, last) as usize;
        self.focused_col = next;
        self.scroll_col_into_view();
    }

    /// Row and column under a screen point, if it lands on a cell.
    fn cell_at(&self, pos: Point) -> Option<(usize, usize)> {
        if !self.extent().contains(pos) {
            return None;
        }
        let local_y = (pos.y) as usize;
        // The header is not a cell.
        let line = local_y.checked_sub(self.header_rows())?;
        let frozen_lines = self.frozen_row_lines();
        let row = if line < frozen_lines {
            line
        } else {
            self.top_row() + (line - frozen_lines)
        };
        if row >= self.rows.len() {
            return None;
        }

        // A click in the gap after a column lands on the next one.
        let local_x = (pos.x) as usize;
        self.layout()
            .into_iter()
            .find(|&(_, x, width)| local_x < x + width)
            .map(|(index, _, _)| (row, index))
    }

    /// Lay one row of text into a buffer, one column at a time.
    ///
    /// `cell` yields the text for a column index; `attr_for` its attribute, so
    /// the header and the body share this code. The gaps between columns are
    /// left as the caller filled them.
    fn write_row(
        &self,
        buf: &mut DrawBuffer,
        cell: impl Fn(usize) -> String,
        attr_for: impl Fn(usize) -> crate::core::palette::Attr,
    ) {
        for (index, offset, col_width) in self.layout() {
            let column = &self.columns[index];
            let attr = attr_for(index);
            buf.move_char(offset, ' ', attr, col_width);

            let text = cell(index);
            let chars: Vec<char> = text.chars().collect();
            let shown: String = chars.iter().take(col_width).collect();
            let shown_len = shown.chars().count();
            let pad = match column.align {
                Align::Left => 0,
                Align::Right => col_width - shown_len,
            };
            buf.move_str(offset + pad, &shown, attr);
        }
    }

    /// Put a separator into each gap between two visible columns of one
    /// drawn line, in the colour the line already has there. The gap after
    /// the last visible column is not between two columns and stays blank.
    ///
    /// The gap after the last frozen column takes a [`FROZEN_SEPARATOR`]
    /// whether separators are on or not, as long as a scrolling column
    /// follows.
    fn write_separators(&self, buf: &mut DrawBuffer, width: usize) {
        let mut put = |gap: usize, ch: char| {
            if gap < width {
                let attr = buf.data[gap].attr;
                buf.put_char(gap, ch, attr);
            }
        };
        if self.separators {
            let offsets = self.column_offsets();
            for &(x, w) in offsets.iter().take(offsets.len().saturating_sub(1)) {
                put(x + usize::from(w), SEPARATOR);
            }
        }
        let frozen = self.frozen();
        if frozen > 0 && frozen < self.columns.len() {
            put(self.frozen_span() - COLUMN_GAP, FROZEN_SEPARATOR);
        }
    }

    /// Which palette the table resolves through: `CP_LISTBOX` when its
    /// dialog-relative indices fit the owner palette (a dialog), the
    /// window's scroller entries otherwise (a window, whose palette is too
    /// short for them).
    fn palette_slice(&self) -> &'static [u8] {
        use crate::core::palette::palettes::{CP_LISTBOX, CP_TABLE_WINDOW};
        let needed = CP_LISTBOX.iter().copied().max().unwrap_or(0) as usize;
        match self
            .core
            .palette_chain
            .as_ref()
            .and_then(crate::core::palette_chain::PaletteChainNode::nearest_palette_len)
        {
            Some(len) if len < needed => CP_TABLE_WINDOW,
            _ => CP_LISTBOX,
        }
    }
}

impl ListViewer for Table {
    fn list_state(&self) -> &ListViewerState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListViewerState {
        &mut self.list_state
    }

    fn visible_rows(&self) -> usize {
        Table::visible_rows(self)
    }

    /// One row rendered as a single string, for the shared list machinery.
    ///
    /// The table draws itself column by column and never calls this; it exists
    /// so a `Table` satisfies the same contract as the other list views.
    fn get_text(&self, item: usize, max_len: usize) -> String {
        if item >= self.rows.len() {
            return String::new();
        }
        let joined = (0..self.columns.len())
            .filter_map(|col| self.rows.get(item, col))
            .collect::<Vec<_>>()
            .join(" ");
        joined.chars().take(max_len).collect()
    }
}

impl View for Table {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.core.bounds = bounds;
        self.scroll_row_into_view();
        self.scroll_col_into_view();
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.view_state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.view_state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped().max(0) as usize;
        let height = self.core.bounds.height_clamped().max(0) as usize;
        if width == 0 || height == 0 {
            return;
        }

        // The same state-to-index mapping as ListBox: the list colour depends
        // on whether the table has focus, the focused row takes the selected
        // colour. The palette those indices go through depends on the owner
        // (see `palette_slice`).
        let focused = self.is_focused();
        let normal = if focused {
            self.map_color(LISTBOX_FOCUSED)
        } else {
            self.map_color(LISTBOX_NORMAL)
        };
        let selected = self.map_color(LISTBOX_SELECTED);
        let header = self.map_color(LISTBOX_DIVIDER);
        // The focused cell is the selected colour reversed: always distinct
        // from the row it sits in, whatever the palette, and it follows the
        // theme instead of hard-coding an attribute.
        let cursor = selected.swap();

        let mut y = 0;

        if self.show_header {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', header, width);
            self.write_row(&mut buf, |i| self.columns[i].title.clone(), |_| header);
            self.write_separators(&mut buf, width);
            write_line_to_terminal(terminal, 0, y, &buf);
            y += 1;
        }

        let frozen = self.frozen();
        let frozen_span = self.frozen_span().min(width);
        // One data row as a drawn line.
        let line = |row_index: usize| {
            let mut buf = DrawBuffer::new(width);
            let row_selected = Some(row_index) == self.list_state.focused;
            let row_attr = if row_selected { selected } else { normal };
            // Fill the whole line, gaps included, so the selected row reads
            // as one bar, as ListBox draws its selected item.
            buf.move_char(0, ' ', row_attr, width);
            // Frozen columns read as row labels, in the header's colour,
            // except where the selected row's bar runs through them.
            if !row_selected {
                buf.move_char(0, ' ', header, frozen_span);
            }

            if row_index < self.rows.len() {
                self.write_row(
                    &mut buf,
                    |i| self.rows.get(row_index, i).unwrap_or_default(),
                    |i| {
                        // Within the selected row the focused cell is marked
                        // while the table has focus, so Left and Right show.
                        if row_selected && focused && i == self.focused_col {
                            cursor
                        } else if i < frozen && !row_selected {
                            header
                        } else {
                            row_attr
                        }
                    },
                );
            }
            self.write_separators(&mut buf, width);
            buf
        };

        // The frozen rows, under the header. The last one is underlined to
        // mark the edge, as long as scrolling rows follow it.
        let frozen_lines = self.frozen_row_lines();
        let marks_edge = frozen_lines > 0 && self.frozen_row_count() < self.row_count();
        for row_index in 0..frozen_lines {
            let mut buf = line(row_index);
            if marks_edge && row_index + 1 == frozen_lines {
                for cell in &mut buf.data {
                    cell.attr.style |= Style::UNDERLINE;
                }
            }
            write_line_to_terminal(terminal, 0, y, &buf);
            y += 1;
        }

        let top = self.top_row();
        for screen_row in 0..self.visible_rows() {
            write_line_to_terminal(terminal, 0, y + screen_row as i16, &line(top + screen_row));
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::MouseDown && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
            if let Some((row, col)) = self.cell_at(event.mouse.pos) {
                self.focus_row(row);
                self.focused_col = col;
                self.scroll_col_into_view();
                if event.mouse.double_click && self.on_select != 0 {
                    *event = Event::command(self.on_select);
                } else {
                    event.clear();
                }
            }
            return;
        }

        if event.what == EventType::MouseWheelUp && self.extent().contains(event.mouse.pos) {
            self.move_row(-1);
            event.clear();
            return;
        }
        if event.what == EventType::MouseWheelDown && self.extent().contains(event.mouse.pos) {
            self.move_row(1);
            event.clear();
            return;
        }

        if !self.is_focused() || event.what != EventType::Keyboard {
            return;
        }

        let ctrl = event
            .key_modifiers
            .contains(crate::core::keys::KeyModifiers::CONTROL);
        let page = self.visible_rows().max(1) as i32;

        match event.key_code {
            KB_UP => self.move_row(-1),
            KB_DOWN => self.move_row(1),
            KB_PGUP => self.move_row(-page),
            KB_PGDN => self.move_row(page),
            KB_HOME => self.move_row(i32::MIN / 2),
            KB_END => self.move_row(i32::MAX / 2),
            KB_LEFT if ctrl => self.move_col(i32::MIN / 2),
            KB_RIGHT if ctrl => self.move_col(i32::MAX / 2),
            KB_LEFT => self.move_col(-1),
            KB_RIGHT => self.move_col(1),
            KB_ENTER if self.on_select != 0 => {
                *event = Event::command(self.on_select);
                return;
            }
            // Not ours: Tab, Enter without a command, and hotkeys must reach
            // the dialog.
            _ => return,
        }
        event.clear();
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        Some(crate::core::palette::Palette::from_slice(
            self.palette_slice(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating tables with a fluent API.
pub struct TableBuilder {
    bounds: Option<Rect>,
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    show_header: bool,
    separators: bool,
    frozen_cols: usize,
    frozen_rows: usize,
    on_select: CommandId,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            columns: Vec::new(),
            rows: Vec::new(),
            show_header: true,
            separators: false,
            frozen_cols: 0,
            frozen_rows: 0,
            on_select: 0,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self
    }

    #[must_use]
    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    #[must_use]
    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    #[must_use]
    pub fn separators(mut self, on: bool) -> Self {
        self.separators = on;
        self
    }

    /// See [`Table::set_frozen_cols`].
    #[must_use]
    pub fn frozen_cols(mut self, count: usize) -> Self {
        self.frozen_cols = count;
        self
    }

    /// See [`Table::set_frozen_rows`].
    #[must_use]
    pub fn frozen_rows(mut self, count: usize) -> Self {
        self.frozen_rows = count;
        self
    }

    #[must_use]
    pub fn on_select(mut self, command: CommandId) -> Self {
        self.on_select = command;
        self
    }

    pub fn build(self) -> Table {
        let bounds = self.bounds.expect("Table bounds must be set");
        let mut table = Table::new(bounds, self.on_select);
        table.set_show_header(self.show_header);
        table.set_separators(self.separators);
        table.set_columns(self.columns);
        table.set_frozen_cols(self.frozen_cols);
        table.set_rows(self.rows);
        table.set_frozen_rows(self.frozen_rows);
        table
    }

    pub fn build_boxed(self) -> Box<Table> {
        Box::new(self.build())
    }
}

impl Default for TableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(rows: usize) -> Table {
        // 30 wide, 6 tall: header plus five rows.
        let mut t = Table::new(Rect::new(0, 0, 30, 6), 42);
        t.set_columns(vec![
            Column::new("Name", 10),
            Column::right("Size", 6),
            Column::new("Kind", 8),
        ]);
        t.set_rows(
            (0..rows)
                .map(|i| vec![format!("file{i}"), format!("{}", i * 10), "text".into()])
                .collect(),
        );
        t.set_state(State::FOCUSED);
        t
    }

    fn press(t: &mut Table, code: u16) -> Event {
        let mut e = Event::keyboard(code);
        t.handle_event(&mut e);
        e
    }

    fn press_ctrl(t: &mut Table, code: u16) {
        let mut e = Event::keyboard(code);
        e.key_modifiers = crate::core::keys::KeyModifiers::CONTROL;
        t.handle_event(&mut e);
    }

    #[test]
    fn first_row_and_column_start_focused() {
        let t = table(3);
        assert_eq!(t.selected_row(), Some(0));
        assert_eq!(t.selected_col(), 0);
        assert_eq!(t.selected_cell().as_deref(), Some("file0"));
    }

    #[test]
    fn an_empty_table_has_no_selection() {
        let t = Table::new(Rect::new(0, 0, 20, 5), 0);
        assert_eq!(t.selected_row(), None);
        assert_eq!(t.selected_cell(), None);
    }

    #[test]
    fn header_costs_one_row_of_grid() {
        let mut t = table(20);
        assert_eq!(t.visible_rows(), 5, "six rows tall, one is the header");
        t.set_show_header(false);
        assert_eq!(t.visible_rows(), 6);
    }

    #[test]
    fn arrows_move_the_focused_cell_and_clamp() {
        let mut t = table(3);
        press(&mut t, KB_DOWN);
        assert_eq!(t.selected_row(), Some(1));
        press(&mut t, KB_RIGHT);
        assert_eq!(t.selected_col(), 1);
        assert_eq!(t.selected_cell().as_deref(), Some("10"));

        for _ in 0..10 {
            press(&mut t, KB_RIGHT);
        }
        assert_eq!(t.selected_col(), 2, "clamped at the last column");
        for _ in 0..10 {
            press(&mut t, KB_DOWN);
        }
        assert_eq!(t.selected_row(), Some(2), "clamped at the last row");
    }

    #[test]
    fn ctrl_arrows_jump_to_the_end_columns() {
        let mut t = table(3);
        press_ctrl(&mut t, KB_RIGHT);
        assert_eq!(t.selected_col(), 2);
        press_ctrl(&mut t, KB_LEFT);
        assert_eq!(t.selected_col(), 0);
    }

    #[test]
    fn home_and_end_jump_to_the_end_rows() {
        let mut t = table(40);
        press(&mut t, KB_END);
        assert_eq!(t.selected_row(), Some(39));
        press(&mut t, KB_HOME);
        assert_eq!(t.selected_row(), Some(0));
    }

    #[test]
    fn paging_moves_a_screenful() {
        let mut t = table(40);
        press(&mut t, KB_PGDN);
        assert_eq!(t.selected_row(), Some(5), "five grid rows per screen");
    }

    #[test]
    fn the_grid_scrolls_to_keep_the_focused_row_visible() {
        let mut t = table(40);
        press(&mut t, KB_END);
        let top = t.list_state.top_item;
        assert!(top > 0, "scrolled down");
        assert!(t.selected_row().unwrap() >= top);
        assert!(t.selected_row().unwrap() < top + t.visible_rows());
    }

    #[test]
    fn wide_grids_scroll_sideways_by_whole_columns() {
        // 10 + 6 + 8 plus two gaps is 26, so all three fit in 30.
        let mut t = table(3);
        press_ctrl(&mut t, KB_RIGHT);
        assert_eq!(t.first_col, 0, "no scrolling needed when everything fits");

        // Narrow the view so only the first column fits.
        t.set_bounds(Rect::new(0, 0, 12, 6));
        press_ctrl(&mut t, KB_RIGHT);
        assert!(t.first_col > 0, "scrolled right to reach the last column");
        assert_eq!(t.selected_col(), 2);
    }

    #[test]
    fn scrolling_back_left_restores_the_first_column() {
        let mut t = table(3);
        t.set_bounds(Rect::new(0, 0, 12, 6));
        press_ctrl(&mut t, KB_RIGHT);
        press_ctrl(&mut t, KB_LEFT);
        assert_eq!(t.first_col, 0);
    }

    #[test]
    fn enter_emits_the_selection_command() {
        let mut t = table(3);
        let e = press(&mut t, KB_ENTER);
        assert_eq!(e.what, EventType::Command);
        assert_eq!(e.command, 42);
    }

    #[test]
    fn enter_is_left_alone_without_a_command() {
        let mut t = table(3);
        t.set_on_select(0);
        let e = press(&mut t, KB_ENTER);
        assert_eq!(
            e.what,
            EventType::Keyboard,
            "Enter must still reach the default button"
        );
    }

    #[test]
    fn keys_do_nothing_when_not_focused() {
        let mut t = table(3);
        t.set_state(State::empty());
        press(&mut t, KB_DOWN);
        assert_eq!(t.selected_row(), Some(0));
    }

    #[test]
    fn clicking_a_cell_focuses_it() {
        let mut t = table(5);
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        // Row 2 of the grid sits three lines down: header, row 0, row 1.
        e.mouse.pos = Point::new(12, 3);
        t.handle_event(&mut e);
        assert_eq!(t.selected_row(), Some(2));
        assert_eq!(t.selected_col(), 1, "x 12 lands in the second column");
    }

    #[test]
    fn clicking_the_header_selects_nothing() {
        let mut t = table(5);
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.pos = Point::new(2, 0);
        t.handle_event(&mut e);
        assert_eq!(t.selected_row(), Some(0), "unchanged");
    }

    #[test]
    fn clicking_past_the_last_row_selects_nothing() {
        let mut t = table(2);
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.pos = Point::new(2, 5);
        t.handle_event(&mut e);
        assert_eq!(t.selected_row(), Some(0));
    }

    #[test]
    fn double_clicking_emits_the_selection_command() {
        let mut t = table(5);
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.double_click = true;
        e.mouse.pos = Point::new(2, 2);
        t.handle_event(&mut e);
        assert_eq!(e.command, 42);
        assert_eq!(t.selected_row(), Some(1));
    }

    #[test]
    fn ragged_rows_do_not_panic() {
        let mut t = table(0);
        t.set_rows(vec![vec!["only one cell".into()], vec![]]);
        t.set_selected_row(1);
        assert_eq!(t.selected_cell(), None, "missing cells read as absent");
    }

    #[test]
    fn shrinking_the_column_list_clamps_the_focus() {
        let mut t = table(3);
        press_ctrl(&mut t, KB_RIGHT);
        assert_eq!(t.selected_col(), 2);
        t.set_columns(vec![Column::new("Only", 6)]);
        assert_eq!(t.selected_col(), 0);
        assert_eq!(t.first_col, 0);
    }

    #[test]
    fn clearing_rows_drops_the_selection() {
        let mut t = table(5);
        t.clear_rows();
        assert_eq!(t.row_count(), 0);
        assert_eq!(t.selected_cell(), None);
    }

    #[test]
    fn builder_configures_the_table() {
        let t = TableBuilder::new()
            .bounds(Rect::new(0, 0, 20, 4))
            .columns(vec![Column::new("A", 4)])
            .rows(vec![vec!["x".into()]])
            .show_header(false)
            .on_select(9)
            .build();
        assert_eq!(t.row_count(), 1);
        assert_eq!(t.visible_rows(), 4, "no header row");
    }

    #[test]
    fn column_offsets_give_each_visible_column_its_x_and_width() {
        // Name x 0..10, gap, Size x 11..17, gap, Kind x 18..26 in a 30-wide table.
        let mut t = table(3);
        assert_eq!(t.column_offsets(), vec![(0, 10), (11, 6), (18, 8)]);

        // Scrolled one column right: Size now starts the row.
        t.first_col = 1;
        assert_eq!(t.column_offsets(), vec![(0, 6), (7, 8)]);
    }

    #[test]
    fn column_offsets_clip_the_last_column_and_skip_the_off_screen_ones() {
        // 12 wide: Name 0..10 fits, Size starts at 11 with one cell left, Kind is past the edge.
        let mut t = table(1);
        t.set_bounds(Rect::new(0, 0, 12, 6));
        t.first_col = 0;
        assert_eq!(t.column_offsets(), vec![(0, 10), (11, 1)]);
    }

    #[test]
    fn column_offsets_agree_with_the_hit_test() {
        let t = table(3);
        for (index, (x, width)) in t.column_offsets().into_iter().enumerate() {
            let first = Point::new(x as i16, 1);
            let last = Point::new((x + width as usize - 1) as i16, 1);
            assert_eq!(t.cell_at(first), Some((0, index)));
            assert_eq!(t.cell_at(last), Some((0, index)));
        }
    }

    fn draw(t: &mut Table, w: u16, h: u16) -> crate::terminal::Terminal {
        let mut term = crate::test_util::test_terminal(w, h);
        t.draw(&mut term);
        term
    }

    fn ch(term: &crate::terminal::Terminal, x: i16, y: i16) -> char {
        term.read_cell(x, y).unwrap().ch
    }

    #[test]
    fn separators_are_off_by_default() {
        let mut t = table(3);
        assert!(!t.separators());
        let term = draw(&mut t, 30, 6);
        assert_eq!(ch(&term, 10, 1), ' ');
    }

    #[test]
    fn separators_fill_the_gaps_between_visible_columns() {
        // Columns: Name 0..10, gap 10, Size 11..17, gap 17, Kind 18..26.
        let mut t = table(3);
        t.set_separators(true);
        let term = draw(&mut t, 30, 6);
        for y in 0..6 {
            assert_eq!(ch(&term, 10, y), SEPARATOR, "first gap, line {y}");
            assert_eq!(ch(&term, 17, y), SEPARATOR, "second gap, line {y}");
            assert_ne!(ch(&term, 26, y), SEPARATOR, "after the last column, line {y}");
        }
        // Text is untouched.
        assert_eq!(ch(&term, 0, 1), 'f');
    }

    #[test]
    fn a_separator_takes_the_colour_of_its_line() {
        let mut t = table(3);
        t.set_separators(true);
        let term = draw(&mut t, 30, 6);
        let attr = |x, y| term.read_cell(x, y).unwrap().attr;
        assert_eq!(attr(10, 0), attr(0, 0), "header");
        assert_eq!(attr(10, 1), attr(11, 1), "selected row bar");
        assert_eq!(attr(10, 2), attr(11, 2), "normal row");
    }

    #[test]
    fn separators_follow_horizontal_scrolling() {
        // 20 wide: focusing Kind scrolls Name off, leaving Size at 0..6 and
        // Kind at 7..15. The only gap between visible columns is x = 6.
        let mut t = table(3);
        t.set_bounds(Rect::new(0, 0, 20, 6));
        t.set_separators(true);
        t.set_selected_col(2);
        let term = draw(&mut t, 20, 6);
        assert_eq!(ch(&term, 6, 1), SEPARATOR);
        assert_ne!(ch(&term, 15, 1), SEPARATOR);
        assert_ne!(ch(&term, 19, 1), SEPARATOR);
    }

    #[test]
    fn the_builder_sets_separators() {
        let t = TableBuilder::new()
            .bounds(Rect::new(0, 0, 10, 3))
            .separators(true)
            .build();
        assert!(t.separators());
    }

    use std::cell::Cell as Counter;
    use std::rc::Rc;

    /// Row `n` is `[n, n²]`; counts how many cells were asked for.
    struct Squares {
        rows: usize,
        calls: Rc<Counter<usize>>,
    }

    impl RowProvider for Squares {
        fn rows(&self) -> usize {
            self.rows
        }
        fn cell(&self, row: usize, col: usize) -> String {
            self.calls.set(self.calls.get() + 1);
            match col {
                0 => row.to_string(),
                1 => (row * row).to_string(),
                _ => String::new(),
            }
        }
    }

    fn squares(rows: usize) -> (Table, Rc<Counter<usize>>) {
        let calls = Rc::new(Counter::new(0));
        let mut t = Table::new(Rect::new(0, 0, 30, 6), 0);
        t.set_columns(vec![Column::new("N", 10), Column::right("N2", 12)]);
        t.set_provider(Box::new(Squares { rows, calls: Rc::clone(&calls) }));
        t.set_state(State::FOCUSED);
        (t, calls)
    }

    #[test]
    fn a_provider_supplies_rows_lazily() {
        let (mut t, calls) = squares(1_000_000);
        assert_eq!(t.row_count(), 1_000_000);
        t.set_selected_row(999_999);
        assert_eq!(t.selected_cell().as_deref(), Some("999999"));

        calls.set(0);
        let term = draw(&mut t, 30, 6);
        // Five visible rows times two columns, never the whole source.
        assert!(calls.get() <= 10, "asked for {} cells", calls.get());
        let shown = (1..6).any(|y| {
            let line: String = (0..10).map(|x| ch(&term, x, y)).collect();
            line.trim_end() == "999999"
        });
        assert!(shown, "the focused last row is on screen");
    }

    /// A provider with zero rows must behave safely: no focused row or cell,
    /// a zero count, and neither drawing nor key handling panics.
    #[test]
    fn empty_provider_is_safe() {
        let (mut t, _) = squares(0);
        assert_eq!(t.row_count(), 0);
        assert_eq!(t.selected_row(), None);
        assert_eq!(t.selected_cell(), None);

        // Drawing must not panic.
        let _ = draw(&mut t, 30, 6);

        // Key presses must not panic either.
        press(&mut t, KB_DOWN);
        press(&mut t, KB_UP);
        press(&mut t, KB_END);
        press(&mut t, KB_HOME);
        assert_eq!(t.selected_row(), None);
    }

    #[test]
    fn a_shorter_provider_clamps_the_focus() {
        let (mut t, _) = squares(100);
        t.set_selected_row(99);
        t.set_provider(Box::new(Squares { rows: 3, calls: Rc::new(Counter::new(0)) }));
        assert_eq!(t.selected_row(), Some(2));
        // Drawing after the swap must not index past the new end.
        let term = draw(&mut t, 30, 6);
        assert!(ch(&term, 0, 1).is_ascii_digit());
    }

    #[test]
    fn refresh_rows_picks_up_a_grown_source() {
        struct Growing(Rc<Counter<usize>>);
        impl RowProvider for Growing {
            fn rows(&self) -> usize {
                self.0.get()
            }
            fn cell(&self, row: usize, _col: usize) -> String {
                row.to_string()
            }
        }
        let len = Rc::new(Counter::new(2));
        let mut t = Table::new(Rect::new(0, 0, 30, 6), 0);
        t.set_columns(vec![Column::new("N", 10)]);
        t.set_provider(Box::new(Growing(Rc::clone(&len))));
        len.set(5);
        assert_eq!(t.row_count(), 2, "cached until refreshed");
        t.refresh_rows();
        assert_eq!(t.row_count(), 5);
    }

    /// `set_selected_row`/`move_row` clamp to the *cached* `row_count`, not
    /// the provider's possibly-grown live length, so a provider that grows
    /// behind the table's back without a `refresh_rows` call still clamps as
    /// `row_count`'s doc promises.
    #[test]
    fn set_selected_row_clamps_to_the_cached_count_not_a_grown_provider() {
        struct Growing(Rc<Counter<usize>>);
        impl RowProvider for Growing {
            fn rows(&self) -> usize {
                self.0.get()
            }
            fn cell(&self, row: usize, _col: usize) -> String {
                row.to_string()
            }
        }
        let len = Rc::new(Counter::new(2));
        let mut t = Table::new(Rect::new(0, 0, 30, 6), 0);
        t.set_columns(vec![Column::new("N", 10)]);
        t.set_provider(Box::new(Growing(Rc::clone(&len))));

        // The provider grows without a refresh_rows call.
        len.set(100);
        assert_eq!(t.row_count(), 2, "still cached at the old length");

        t.set_selected_row(50);
        assert_eq!(
            t.selected_row(),
            Some(1),
            "clamped to the cached row_count (2), not the live length (100)"
        );

        t.move_row(1000);
        assert_eq!(
            t.selected_row(),
            Some(1),
            "move_row also clamps to the cached count"
        );
    }

    #[test]
    fn set_rows_and_add_row_replace_a_provider() {
        let (mut t, _) = squares(10);
        t.add_row(vec!["only".into(), "row".into()]);
        assert_eq!(t.row_count(), 1);
        assert_eq!(t.selected_cell().as_deref(), Some("only"));

        let (mut t, _) = squares(10);
        t.set_rows(vec![vec!["a".into()], vec!["b".into()]]);
        assert_eq!(t.row_count(), 2);
    }

    #[test]
    fn a_ragged_owned_row_still_reports_no_cell() {
        let mut t = table(1);
        t.set_rows(vec![vec!["short".into()]]);
        t.set_selected_col(2);
        assert_eq!(t.selected_cell(), None);
    }

    // ---- frozen columns ----

    /// The three-column test table, 20 wide (Name 10 + gap fills half), with
    /// Name frozen.
    fn frozen_table(rows: usize) -> Table {
        let mut t = table(rows);
        t.set_bounds(Rect::new(0, 0, 20, 6));
        t.set_frozen_cols(1);
        t
    }

    #[test]
    fn nothing_is_frozen_by_default() {
        let t = table(3);
        assert_eq!(t.frozen_cols(), 0);
        assert_eq!(t.column_offsets(), vec![(0, 10), (11, 6), (18, 8)]);
    }

    #[test]
    fn a_frozen_column_stays_at_the_left_while_the_rest_scroll() {
        let mut t = frozen_table(3);
        // Name 0..10 frozen, gap 10, Size 11..17, Kind from 18, clipped to 2.
        assert_eq!(t.column_offsets(), vec![(0, 10), (11, 6), (18, 2)]);
        // Focusing Kind scrolls Size away; Name stays.
        t.set_selected_col(2);
        assert_eq!(t.column_offsets(), vec![(0, 10), (11, 8)]);
        assert_eq!(t.first_col, 2);
    }

    #[test]
    fn focusing_a_frozen_column_scrolls_nothing() {
        let mut t = frozen_table(3);
        t.set_selected_col(2);
        t.set_selected_col(0);
        assert_eq!(t.first_col, 2, "the scrolled part stays where it was");
        assert_eq!(t.selected_cell().as_deref(), Some("file0"));
    }

    #[test]
    fn left_walks_back_into_the_frozen_column() {
        let mut t = frozen_table(3);
        press_ctrl(&mut t, KB_RIGHT);
        assert_eq!(t.selected_col(), 2);
        press(&mut t, KB_LEFT);
        assert_eq!(t.selected_col(), 1);
        assert_eq!(t.first_col, 1, "Size scrolls back in");
        press(&mut t, KB_LEFT);
        assert_eq!(t.selected_col(), 0);
        press_ctrl(&mut t, KB_RIGHT);
        press_ctrl(&mut t, KB_LEFT);
        assert_eq!(t.selected_col(), 0, "Ctrl+Left still reaches column 0");
    }

    #[test]
    fn clicks_land_on_frozen_and_scrolled_columns_alike() {
        let mut t = frozen_table(3);
        t.set_selected_col(2); // Size scrolled off
        for (index, (x, width)) in t.column_offsets().into_iter().enumerate() {
            let expected = [0, 2][index];
            let first = Point::new(i16::try_from(x).unwrap(), 1);
            let last = Point::new(i16::try_from(x + usize::from(width) - 1).unwrap(), 1);
            assert_eq!(t.cell_at(first), Some((0, expected)));
            assert_eq!(t.cell_at(last), Some((0, expected)));
        }
    }

    #[test]
    fn the_freeze_line_marks_the_edge_even_without_separators() {
        let mut t = frozen_table(3);
        assert!(!t.separators());
        let term = draw(&mut t, 20, 6);
        for y in 0..6 {
            assert_eq!(ch(&term, 10, y), FROZEN_SEPARATOR, "line {y}");
        }
        assert_eq!(ch(&term, 17, 1), ' ', "no plain separators");
    }

    #[test]
    fn with_separators_the_freeze_line_keeps_its_own_glyph() {
        let mut t = frozen_table(3);
        t.set_separators(true);
        let term = draw(&mut t, 20, 6);
        assert_eq!(ch(&term, 10, 1), FROZEN_SEPARATOR);
        assert_eq!(ch(&term, 17, 1), SEPARATOR);
    }

    #[test]
    fn no_freeze_line_when_nothing_or_everything_is_frozen() {
        let mut t = table(3);
        let term = draw(&mut t, 30, 6);
        assert_ne!(ch(&term, 10, 1), FROZEN_SEPARATOR);
        t.set_frozen_cols(3);
        let term = draw(&mut t, 30, 6);
        assert_ne!(
            ch(&term, 26, 1),
            FROZEN_SEPARATOR,
            "nothing scrolls past it"
        );
    }

    #[test]
    fn frozen_cells_take_the_header_colour_outside_the_selected_row() {
        let mut t = frozen_table(3);
        let term = draw(&mut t, 20, 6);
        let attr = |x, y| term.read_cell(x, y).unwrap().attr;
        let header = attr(0, 0);
        // Row 1 (screen line 2) is not selected: its label is header-coloured,
        // its scrolling cells are not.
        assert_eq!(attr(0, 2), header);
        assert_eq!(attr(9, 2), header);
        assert_ne!(attr(11, 2), header);
        // The selected row's bar runs through the frozen part; the focused
        // cell (Name, frozen) still shows as the cursor.
        assert_ne!(attr(0, 1), header);
        t.set_selected_col(1);
        let term = draw(&mut t, 20, 6);
        let attr = |x, y| term.read_cell(x, y).unwrap().attr;
        assert_eq!(attr(0, 1), attr(18, 1), "one bar across the selected row");
    }

    #[test]
    fn freezing_more_columns_than_exist_freezes_them_all() {
        let mut t = table(3);
        t.set_frozen_cols(10);
        assert_eq!(t.frozen_cols(), 10);
        assert_eq!(t.column_offsets(), vec![(0, 10), (11, 6), (18, 8)]);
        press_ctrl(&mut t, KB_RIGHT);
        assert_eq!(t.selected_col(), 2);
        let _ = draw(&mut t, 30, 6);
    }

    #[test]
    fn frozen_columns_wider_than_the_table_are_clipped() {
        let mut t = table(3);
        t.set_bounds(Rect::new(0, 0, 8, 6));
        t.set_frozen_cols(2);
        assert_eq!(t.column_offsets(), vec![(0, 8)]);
        press_ctrl(&mut t, KB_RIGHT);
        assert_eq!(t.selected_col(), 2);
        let _ = draw(&mut t, 8, 6);
    }

    #[test]
    fn fewer_columns_keep_the_scroll_out_of_the_frozen_part() {
        let mut t = frozen_table(3);
        t.set_selected_col(2);
        t.set_columns(vec![Column::new("Name", 10), Column::new("Kind", 8)]);
        assert!(t.first_col >= 1);
        assert_eq!(t.column_offsets(), vec![(0, 10), (11, 8)]);
        t.clear_rows();
        assert_eq!(t.first_col, 1);
    }

    #[test]
    fn the_builder_freezes_columns() {
        let t = TableBuilder::new()
            .bounds(Rect::new(0, 0, 20, 6))
            .columns(vec![Column::new("A", 4), Column::new("B", 4)])
            .frozen_cols(1)
            .build();
        assert_eq!(t.frozen_cols(), 1);
    }

    // ---- frozen rows ----

    /// The text on screen line `y`, from x 0 to 6.
    fn line_start(term: &crate::terminal::Terminal, y: i16) -> String {
        (0..6)
            .map(|x| ch(term, x, y))
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    fn underlined(term: &crate::terminal::Terminal, x: i16, y: i16) -> bool {
        term.read_cell(x, y)
            .unwrap()
            .attr
            .style
            .contains(Style::UNDERLINE)
    }

    #[test]
    fn a_frozen_row_stays_under_the_header_while_the_rest_scroll() {
        let mut t = table(40);
        t.set_frozen_rows(1);
        assert_eq!(
            t.visible_rows(),
            4,
            "header and one frozen row take two of six"
        );
        press(&mut t, KB_END);
        let term = draw(&mut t, 30, 6);
        assert_eq!(line_start(&term, 0), "Name");
        assert_eq!(line_start(&term, 1), "file0");
        assert_eq!(line_start(&term, 2), "file36");
        assert_eq!(line_start(&term, 5), "file39");
    }

    #[test]
    fn the_last_frozen_row_is_underlined() {
        let mut t = table(40);
        t.set_frozen_rows(2);
        let term = draw(&mut t, 30, 6);
        for x in [0, 15, 29] {
            assert!(!underlined(&term, x, 1), "first frozen row, x {x}");
            assert!(underlined(&term, x, 2), "last frozen row, x {x}");
            assert!(!underlined(&term, x, 3), "first scrolling row, x {x}");
        }
    }

    #[test]
    fn no_underline_when_nothing_or_everything_is_frozen() {
        let mut t = table(3);
        let term = draw(&mut t, 30, 6);
        assert!(!(0..6).any(|y| underlined(&term, 0, y)));
        t.set_frozen_rows(3);
        let term = draw(&mut t, 30, 6);
        assert!(
            !(0..6).any(|y| underlined(&term, 0, y)),
            "nothing scrolls under it"
        );
    }

    #[test]
    fn focusing_a_frozen_row_scrolls_nothing() {
        let mut t = table(40);
        t.set_frozen_rows(1);
        press(&mut t, KB_END);
        let top = t.top_row();
        press(&mut t, KB_HOME);
        assert_eq!(t.selected_row(), Some(0));
        assert_eq!(t.top_row(), top, "the scrolled rows stay where they were");
    }

    #[test]
    fn up_walks_from_the_scrolling_rows_into_the_frozen_one() {
        let mut t = table(40);
        t.set_frozen_rows(1);
        press(&mut t, KB_DOWN); // row 1, the first scrolling row
        assert_eq!(t.top_row(), 1);
        press(&mut t, KB_UP);
        assert_eq!(t.selected_row(), Some(0));
        assert_eq!(
            t.top_row(),
            1,
            "scrolling never starts inside the frozen rows"
        );
    }

    #[test]
    fn paging_moves_a_screenful_of_scrolling_rows() {
        let mut t = table(40);
        t.set_frozen_rows(1);
        press(&mut t, KB_PGDN);
        assert_eq!(t.selected_row(), Some(4), "four scrolling rows per screen");
    }

    #[test]
    fn clicks_land_on_frozen_and_scrolled_rows_alike() {
        let mut t = table(40);
        t.set_frozen_rows(1);
        press(&mut t, KB_END);
        assert_eq!(t.cell_at(Point::new(0, 0)), None, "the header");
        assert_eq!(t.cell_at(Point::new(0, 1)), Some((0, 0)));
        assert_eq!(t.cell_at(Point::new(0, 2)), Some((36, 0)));
        let mut e = Event::mouse(
            EventType::MouseDown,
            Point::new(0, 1),
            MB_LEFT_BUTTON,
            false,
        );
        t.handle_event(&mut e);
        assert_eq!(t.selected_row(), Some(0));
        assert_eq!(t.top_row(), 36);
    }

    #[test]
    fn frozen_rows_taller_than_the_table_are_clipped() {
        let mut t = table(40);
        t.set_frozen_rows(10);
        assert_eq!(t.visible_rows(), 0);
        press(&mut t, KB_END);
        assert_eq!(t.selected_row(), Some(39));
        let term = draw(&mut t, 30, 6);
        assert_eq!(line_start(&term, 5), "file4");
    }

    #[test]
    fn freezing_more_rows_than_exist_freezes_them_all() {
        let mut t = table(3);
        t.set_frozen_rows(10);
        press(&mut t, KB_END);
        assert_eq!(t.selected_row(), Some(2));
        let term = draw(&mut t, 30, 6);
        assert_eq!(line_start(&term, 3), "file2");
        assert_eq!(line_start(&term, 4), "", "no rows past the end");
    }

    #[test]
    fn frozen_rows_come_from_a_provider_too() {
        let (mut t, _) = squares(1000);
        t.set_frozen_rows(1);
        t.set_selected_row(500);
        let term = draw(&mut t, 30, 6);
        assert_eq!(line_start(&term, 1), "0");
        assert_eq!(
            line_start(&term, 5),
            "500",
            "the focused row, last on screen"
        );
    }

    #[test]
    fn frozen_rows_and_columns_together_keep_the_corner() {
        let mut t = table(40);
        t.set_bounds(Rect::new(0, 0, 20, 6));
        t.set_frozen_cols(1);
        t.set_frozen_rows(1);
        press(&mut t, KB_END);
        press_ctrl(&mut t, KB_RIGHT);
        let term = draw(&mut t, 20, 6);
        assert_eq!(line_start(&term, 1), "file0", "the corner cell stays");
        assert_eq!(ch(&term, 10, 1), FROZEN_SEPARATOR);
        assert!(underlined(&term, 10, 1), "the two freeze lines cross");
        assert_eq!(t.selected_cell().as_deref(), Some("text"));
        assert_eq!(t.cell_at(Point::new(11, 1)), Some((0, 2)));
    }

    #[test]
    fn the_builder_freezes_rows() {
        let t = TableBuilder::new()
            .bounds(Rect::new(0, 0, 20, 6))
            .columns(vec![Column::new("A", 4)])
            .rows(vec![vec!["x".into()], vec!["y".into()]])
            .frozen_rows(1)
            .build();
        assert_eq!(t.frozen_rows(), 1);
    }
}
