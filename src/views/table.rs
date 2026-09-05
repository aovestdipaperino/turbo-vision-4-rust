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

use super::list_viewer::{ListViewer, ListViewerState};
use super::view::{View, write_line_to_terminal};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_DOWN, KB_END, KB_ENTER, KB_HOME, KB_LEFT, KB_PGDN, KB_PGUP, KB_RIGHT,
    KB_UP, MB_LEFT_BUTTON,
};
use crate::core::geometry::{Point, Rect};
use crate::core::palette::{LISTBOX_DIVIDER, LISTBOX_FOCUSED, LISTBOX_NORMAL, LISTBOX_SELECTED};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;

/// Blank cells between two columns.
const COLUMN_GAP: usize = 1;

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

/// A scrollable grid of rows and sized columns.
pub struct Table {
    bounds: Rect,
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    /// Row focus and vertical scrolling, shared with the other list views.
    list_state: ListViewerState,
    /// Index of the focused column.
    focused_col: usize,
    /// Leftmost visible column, for grids wider than the view.
    first_col: usize,
    /// Whether the header row is drawn.
    show_header: bool,
    /// Command emitted by Enter or a double-click.
    on_select: CommandId,
    view_state: StateFlags,
    palette_chain: Option<crate::core::palette_chain::PaletteChainNode>,
}

impl Table {
    /// Create an empty table that emits `on_select` when a cell is chosen.
    pub fn new(bounds: Rect, on_select: CommandId) -> Self {
        Self {
            bounds,
            columns: Vec::new(),
            rows: Vec::new(),
            list_state: ListViewerState::new(),
            focused_col: 0,
            first_col: 0,
            show_header: true,
            on_select,
            view_state: 0,
            palette_chain: None,
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
        self.rows = rows;
        self.list_state.set_range(self.rows.len());
        self.scroll_row_into_view();
    }

    /// Append one row.
    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
        self.list_state.set_range(self.rows.len());
    }

    /// Drop every row, keeping the columns.
    pub fn clear_rows(&mut self) {
        self.rows.clear();
        self.list_state.set_range(0);
        self.first_col = 0;
    }

    /// Number of rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
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
    pub fn selected_cell(&self) -> Option<&str> {
        let row = self.rows.get(self.list_state.focused?)?;
        row.get(self.focused_col).map(|s| &**s)
    }

    /// Focus a row, clamped to the rows that exist.
    pub fn set_selected_row(&mut self, row: usize) {
        if self.rows.is_empty() {
            return;
        }
        let row = row.min(self.rows.len() - 1);
        let visible = self.visible_rows();
        self.list_state.focus_item(row, visible);
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

    /// Command emitted by Enter or a double-click.
    pub fn set_on_select(&mut self, command: CommandId) {
        self.on_select = command;
    }

    /// Rows of grid visible at once, header excluded.
    fn visible_rows(&self) -> usize {
        let height = self.bounds.height_clamped().max(0) as usize;
        height.saturating_sub(self.header_rows())
    }

    /// 1 when the header is drawn, 0 otherwise.
    fn header_rows(&self) -> usize {
        usize::from(self.show_header)
    }

    /// Keep the column indices inside the current column list.
    fn clamp_columns(&mut self) {
        if self.columns.is_empty() {
            self.focused_col = 0;
            self.first_col = 0;
            return;
        }
        let last = self.columns.len() - 1;
        self.focused_col = self.focused_col.min(last);
        self.first_col = self.first_col.min(last);
        self.scroll_col_into_view();
    }

    /// Scroll vertically so the focused row is on screen.
    fn scroll_row_into_view(&mut self) {
        if let Some(row) = self.list_state.focused {
            let visible = self.visible_rows();
            self.list_state.focus_item(row, visible);
        }
    }

    /// Scroll horizontally so the focused column is fully visible.
    ///
    /// Columns are whole units: the leftmost visible column advances until the
    /// focused one fits, rather than clipping a column in half.
    fn scroll_col_into_view(&mut self) {
        if self.focused_col < self.first_col {
            self.first_col = self.focused_col;
            return;
        }
        let width = self.bounds.width_clamped().max(0) as usize;
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

    /// Starting cell of a column within the drawn row, or `None` when it is
    /// scrolled off to the left.
    fn column_offset(&self, index: usize) -> Option<usize> {
        if index < self.first_col {
            return None;
        }
        Some((self.first_col..index).map(|i| self.column_span(i)).sum())
    }

    /// Move the focused row by `delta`, clamping at both ends.
    fn move_row(&mut self, delta: i32) {
        if self.rows.is_empty() {
            return;
        }
        let last = self.rows.len() as i32 - 1;
        let current = self.list_state.focused.unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, last) as usize;
        let visible = self.visible_rows();
        self.list_state.focus_item(next, visible);
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
        if !self.bounds.contains(pos) {
            return None;
        }
        let local_y = (pos.y - self.bounds.a.y) as usize;
        // The header is not a cell.
        let row_index = local_y.checked_sub(self.header_rows())?;
        let row = self.list_state.top_item + row_index;
        if row >= self.rows.len() {
            return None;
        }

        let local_x = (pos.x - self.bounds.a.x) as usize;
        let mut offset = 0;
        for index in self.first_col..self.columns.len() {
            let width = self.columns[index].width as usize;
            if local_x < offset + width {
                return Some((row, index));
            }
            offset += width + COLUMN_GAP;
        }
        None
    }

    /// Lay one row of text into a buffer, one column at a time.
    ///
    /// `cell` yields the text for a column index; `attr_for` its attribute, so
    /// the header and the body share this code.
    fn write_row(
        &self,
        buf: &mut DrawBuffer,
        width: usize,
        cell: impl Fn(usize) -> String,
        attr_for: impl Fn(usize) -> crate::core::palette::Attr,
    ) {
        for index in self.first_col..self.columns.len() {
            let Some(offset) = self.column_offset(index) else {
                continue;
            };
            if offset >= width {
                break;
            }
            let column = &self.columns[index];
            let col_width = (column.width as usize).min(width - offset);
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
        let Some(row) = self.rows.get(item) else {
            return String::new();
        };
        let joined = row.join(" ");
        joined.chars().take(max_len).collect()
    }
}

impl View for Table {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
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
        let width = self.bounds.width_clamped().max(0) as usize;
        let height = self.bounds.height_clamped().max(0) as usize;
        if width == 0 || height == 0 {
            return;
        }

        let normal = if self.is_focused() {
            self.map_color(LISTBOX_FOCUSED)
        } else {
            self.map_color(LISTBOX_NORMAL)
        };
        let selected = self.map_color(LISTBOX_SELECTED);
        let header = self.map_color(LISTBOX_DIVIDER);
        // The four-entry list palette has no dedicated cursor colour; the
        // divider entry is the one that reads as distinct from both.
        let cursor = header;

        let mut y = self.bounds.a.y;

        if self.show_header {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', header, width);
            self.write_row(
                &mut buf,
                width,
                |i| self.columns[i].title.clone(),
                |_| header,
            );
            write_line_to_terminal(terminal, self.bounds.a.x, y, &buf);
            y += 1;
        }

        for screen_row in 0..self.visible_rows() {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', normal, width);

            let row_index = self.list_state.top_item + screen_row;
            if let Some(row) = self.rows.get(row_index) {
                let row_focused = Some(row_index) == self.list_state.focused;
                self.write_row(
                    &mut buf,
                    width,
                    |i| row.get(i).cloned().unwrap_or_default(),
                    |i| {
                        // The selected row lifts as a whole. Within it, the
                        // focused cell is marked separately while the table has
                        // focus, so Left and Right are visible.
                        match (row_focused, self.is_focused() && i == self.focused_col) {
                            (true, true) => cursor,
                            (true, false) => selected,
                            (false, _) => normal,
                        }
                    },
                );
            }
            write_line_to_terminal(terminal, self.bounds.a.x, y + screen_row as i16, &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::MouseDown && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
            if let Some((row, col)) = self.cell_at(event.mouse.pos) {
                let visible = self.visible_rows();
                self.list_state.focus_item(row, visible);
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

        if event.what == EventType::MouseWheelUp && self.bounds.contains(event.mouse.pos) {
            self.move_row(-1);
            event.clear();
            return;
        }
        if event.what == EventType::MouseWheelDown && self.bounds.contains(event.mouse.pos) {
            self.move_row(1);
            event.clear();
            return;
        }

        if !self.is_focused() || event.what != EventType::Keyboard {
            return;
        }

        let ctrl = event
            .key_modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL);
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

    fn set_palette_chain(&mut self, node: Option<crate::core::palette_chain::PaletteChainNode>) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_LISTBOX))
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
    on_select: CommandId,
}

impl TableBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            columns: Vec::new(),
            rows: Vec::new(),
            show_header: true,
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
    pub fn on_select(mut self, command: CommandId) -> Self {
        self.on_select = command;
        self
    }

    pub fn build(self) -> Table {
        let bounds = self.bounds.expect("Table bounds must be set");
        let mut table = Table::new(bounds, self.on_select);
        table.set_show_header(self.show_header);
        table.set_columns(self.columns);
        table.set_rows(self.rows);
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
    use crate::core::state::SF_FOCUSED;

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
        t.set_state(SF_FOCUSED);
        t
    }

    fn press(t: &mut Table, code: u16) -> Event {
        let mut e = Event::keyboard(code);
        t.handle_event(&mut e);
        e
    }

    fn press_ctrl(t: &mut Table, code: u16) {
        let mut e = Event::keyboard(code);
        e.key_modifiers = crossterm::event::KeyModifiers::CONTROL;
        t.handle_event(&mut e);
    }

    #[test]
    fn first_row_and_column_start_focused() {
        let t = table(3);
        assert_eq!(t.selected_row(), Some(0));
        assert_eq!(t.selected_col(), 0);
        assert_eq!(t.selected_cell(), Some("file0"));
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
        assert_eq!(t.selected_cell(), Some("10"));

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
        t.set_state(0);
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
}
