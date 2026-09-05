// (C) 2026 - Enzo Lombardi

//! GridView - multi-column data browser (tvDMX style).

use turbo_vision::core::command::CommandId;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{
    Event, EventType, KB_DOWN, KB_END, KB_ENTER, KB_HOME, KB_LEFT, KB_PGDN, KB_PGUP, KB_RIGHT,
    KB_UP, MB_LEFT_BUTTON,
};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::core::state::State;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{View, ViewCore};

/// Column definition for [`GridView`].
#[derive(Clone, Debug)]
pub struct GridColumn {
    /// Header caption.
    pub title: String,
    /// Column width in cells (content is truncated to fit).
    pub width: usize,
}

impl GridColumn {
    /// Create a column.
    pub fn new(title: impl Into<String>, width: usize) -> Self {
        Self {
            title: title.into(),
            width: width.max(1),
        }
    }
}

/// Lazy cell source for [`GridView`].
///
/// Cells are fetched only for the visible viewport, so providers can sit
/// directly on top of files, databases, or computed data (the tvDMX
/// pattern).
pub trait RowProvider {
    /// Total number of rows.
    fn rows(&self) -> usize;

    /// Cell text for (`row`, `col`); both are always in range.
    fn cell(&self, row: usize, col: usize) -> String;
}

/// Simple in-memory provider over `Vec<Vec<String>>`.
#[derive(Debug, Default)]
pub struct VecRowProvider {
    /// Row-major cell data.
    pub data: Vec<Vec<String>>,
}

impl RowProvider for VecRowProvider {
    fn rows(&self) -> usize {
        self.data.len()
    }

    fn cell(&self, row: usize, col: usize) -> String {
        self.data
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
            .unwrap_or_default()
    }
}

/// Scrollable multi-column data browser with a header row.
///
/// Arrows move the row/column cursor, PgUp/PgDn page, Home/End jump to the
/// first/last row, clicking focuses a cell, and Enter (or double-click)
/// converts the event into the constructor's command with the focused row
/// in `event.info` (clamped to `u16`).
///
/// # Examples
///
/// ```
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision_extras::{GridColumn, GridView, VecRowProvider};
///
/// let provider = VecRowProvider {
///     data: vec![vec!["1".into(), "Ada".into()], vec!["2".into(), "Grace".into()]],
/// };
/// let columns = vec![GridColumn::new("Id", 4), GridColumn::new("Name", 12)];
/// let grid = GridView::new(Rect::new(0, 0, 30, 10), columns, Box::new(provider), 900);
/// assert_eq!(grid.focused_row(), 0);
/// ```
pub struct GridView {
    core: ViewCore,
    columns: Vec<GridColumn>,
    provider: Box<dyn RowProvider>,
    top_row: usize,
    focused_row: usize,
    focused_col: usize,
    on_select: CommandId,
    palette_chain: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
}

impl std::fmt::Debug for GridView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GridView")
            .field("bounds", &self.core.bounds)
            .field("columns", &self.columns.len())
            .field("rows", &self.provider.rows())
            .finish()
    }
}

impl GridView {
    /// Create a grid; Enter/double-click emits `on_select` with the row in
    /// `event.info`.
    pub fn new(
        bounds: Rect,
        columns: Vec<GridColumn>,
        provider: Box<dyn RowProvider>,
        on_select: CommandId,
    ) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: State::empty(),
                ..ViewCore::default()
            },
            columns,
            provider,
            top_row: 0,
            focused_row: 0,
            focused_col: 0,
            on_select,
            palette_chain: None,
        }
    }

    /// Replace the data source, resetting the cursor.
    pub fn set_provider(&mut self, provider: Box<dyn RowProvider>) {
        self.provider = provider;
        self.top_row = 0;
        self.focused_row = 0;
        self.focused_col = 0;
    }

    /// Focused (cursor) row.
    pub fn focused_row(&self) -> usize {
        self.focused_row
    }

    /// Focused (cursor) column.
    pub fn focused_col(&self) -> usize {
        self.focused_col
    }

    /// Rows visible below the header.
    fn page_rows(&self) -> usize {
        (self.core.bounds.height_clamped() as usize)
            .saturating_sub(1)
            .max(1)
    }

    fn clamp_and_scroll(&mut self) {
        let rows = self.provider.rows();
        if rows == 0 {
            self.focused_row = 0;
            self.top_row = 0;
            return;
        }
        self.focused_row = self.focused_row.min(rows - 1);
        self.focused_col = self.focused_col.min(self.columns.len().saturating_sub(1));

        let page = self.page_rows();
        if self.focused_row < self.top_row {
            self.top_row = self.focused_row;
        } else if self.focused_row >= self.top_row + page {
            self.top_row = self.focused_row + 1 - page;
        }
    }

    /// Move the row cursor by `delta` (negative = up).
    pub fn move_cursor(&mut self, delta: i64) {
        let rows = self.provider.rows() as i64;
        if rows == 0 {
            return;
        }
        let new_row = (self.focused_row as i64 + delta).clamp(0, rows - 1);
        self.focused_row = new_row as usize;
        self.clamp_and_scroll();
    }

    /// Starting cell column (relative) of grid column `col`, including the
    /// separator after each preceding column.
    fn col_start(&self, col: usize) -> usize {
        self.columns[..col].iter().map(|c| c.width + 1).sum()
    }

    fn col_at(&self, x: usize) -> Option<usize> {
        let mut start = 0;
        for (i, c) in self.columns.iter().enumerate() {
            if x < start + c.width {
                return Some(i);
            }
            start += c.width + 1;
        }
        None
    }

    fn select(&self, event: &mut Event) {
        *event = Event::broadcast_with_info(
            self.on_select,
            self.focused_row.min(u16::MAX as usize) as u16,
        );
    }
}

impl View for GridView {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        let height = self.core.bounds.height_clamped() as usize;
        if width == 0 || height == 0 {
            return;
        }

        let header_attr = Attr::new(TvColor::White, TvColor::Green);
        let normal = Attr::new(TvColor::Black, TvColor::Cyan);
        let row_hl = Attr::new(TvColor::White, TvColor::Blue);
        let cell_hl = Attr::new(TvColor::Yellow, TvColor::Blue);
        let sep_char = '│';

        // Header
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', header_attr, width);
        for (i, col) in self.columns.iter().enumerate() {
            let start = self.col_start(i);
            if start >= width {
                break;
            }
            let text: String = col
                .title
                .chars()
                .take(col.width.min(width - start))
                .collect();
            buf.move_str(start, &text, header_attr);
        }
        write_line_to_terminal(terminal, 0, 0, &buf);

        // Rows
        let rows = self.provider.rows();
        for screen_row in 0..height - 1 {
            let row = self.top_row + screen_row;
            let row_focused = row == self.focused_row && self.is_focused();
            let base = if row_focused { row_hl } else { normal };

            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', base, width);
            if row < rows {
                for (i, col) in self.columns.iter().enumerate() {
                    let start = self.col_start(i);
                    if start >= width {
                        break;
                    }
                    let attr = if row_focused && i == self.focused_col {
                        cell_hl
                    } else {
                        base
                    };
                    let avail = col.width.min(width - start);
                    let text: String = self.provider.cell(row, i).chars().take(avail).collect();
                    buf.move_str(start, &text, attr);
                    let sep = start + col.width;
                    if sep < width {
                        buf.put_char(sep, sep_char, base);
                    }
                }
            }
            write_line_to_terminal(
                terminal,
                0,
                1 + screen_row as i16,
                &buf,
            );
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard if self.is_focused() => {
                let page = self.page_rows() as i64;
                match event.key_code {
                    KB_UP => self.move_cursor(-1),
                    KB_DOWN => self.move_cursor(1),
                    KB_PGUP => self.move_cursor(-page),
                    KB_PGDN => self.move_cursor(page),
                    KB_HOME => {
                        self.focused_row = 0;
                        self.clamp_and_scroll();
                    }
                    KB_END => {
                        self.focused_row = self.provider.rows().saturating_sub(1);
                        self.clamp_and_scroll();
                    }
                    KB_LEFT => {
                        self.focused_col = self.focused_col.saturating_sub(1);
                    }
                    KB_RIGHT => {
                        self.focused_col =
                            (self.focused_col + 1).min(self.columns.len().saturating_sub(1));
                    }
                    KB_ENTER => {
                        if self.provider.rows() > 0 {
                            self.select(event);
                        }
                        return;
                    }
                    _ => return,
                }
                event.clear();
            }
            EventType::MouseDown => {
                let pos = event.mouse.pos;
                if event.mouse.buttons & MB_LEFT_BUTTON == 0 || !self.extent().contains(pos) {
                    return;
                }
                let rel_y = (pos.y) as usize;
                if rel_y >= 1 {
                    let row = self.top_row + rel_y - 1;
                    if row < self.provider.rows() {
                        let was_focused = row == self.focused_row;
                        self.focused_row = row;
                        if let Some(col) = self.col_at((pos.x) as usize) {
                            self.focused_col = col;
                        }
                        self.clamp_and_scroll();
                        if event.mouse.double_click && was_focused {
                            self.select(event);
                            return;
                        }
                    }
                }
                event.clear();
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }

    fn set_palette_chain(
        &mut self,
        node: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
    ) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&turbo_vision::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::geometry::Point;

    struct Big;
    impl RowProvider for Big {
        fn rows(&self) -> usize {
            100_000
        }
        fn cell(&self, row: usize, col: usize) -> String {
            format!("{row}:{col}")
        }
    }

    fn make() -> GridView {
        let columns = vec![GridColumn::new("A", 6), GridColumn::new("B", 8)];
        let mut grid = GridView::new(Rect::new(0, 0, 30, 6), columns, Box::new(Big), 900);
        grid.set_focus(true);
        grid
    }

    #[test]
    fn navigation_scrolls_and_clamps() {
        let mut grid = make();
        // 6 rows tall = 5 data rows per page
        let mut ev = Event::keyboard(KB_PGDN);
        grid.handle_event(&mut ev);
        assert_eq!(grid.focused_row(), 5);

        let mut ev = Event::keyboard(KB_END);
        grid.handle_event(&mut ev);
        assert_eq!(grid.focused_row(), 99_999);

        let mut ev = Event::keyboard(KB_RIGHT);
        grid.handle_event(&mut ev);
        assert_eq!(grid.focused_col(), 1);
        let mut ev = Event::keyboard(KB_RIGHT);
        grid.handle_event(&mut ev);
        assert_eq!(grid.focused_col(), 1); // clamped to last column
    }

    #[test]
    fn enter_broadcasts_selected_row() {
        let mut grid = make();
        let mut ev = Event::keyboard(KB_DOWN);
        grid.handle_event(&mut ev);
        let mut ev = Event::keyboard(KB_ENTER);
        grid.handle_event(&mut ev);
        assert_eq!(ev.what, EventType::Broadcast);
        assert_eq!(ev.command, 900);
        assert_eq!(ev.info, 1);
    }

    #[test]
    fn click_focuses_cell() {
        let mut grid = make();
        // Row 2 on screen = data row 1; x=8 falls in column B (starts at 7)
        let mut ev = Event::mouse(
            EventType::MouseDown,
            Point::new(8, 2),
            MB_LEFT_BUTTON,
            false,
        );
        grid.handle_event(&mut ev);
        assert_eq!(grid.focused_row(), 1);
        assert_eq!(grid.focused_col(), 1);
    }

    #[test]
    fn empty_provider_is_safe() {
        let provider = VecRowProvider { data: Vec::new() };
        let mut grid = GridView::new(
            Rect::new(0, 0, 20, 5),
            vec![GridColumn::new("A", 5)],
            Box::new(provider),
            900,
        );
        grid.set_focus(true);
        let mut ev = Event::keyboard(KB_ENTER);
        grid.handle_event(&mut ev);
        assert_ne!(ev.what, EventType::Broadcast);
    }
}
