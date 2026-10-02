// (C) 2026 - Antoni Aloy
// Frozen Table Demo - a Table with a frozen column and a frozen row.
//
// Monthly sales for thirty regions: too wide and too tall for the window, so
// the grid scrolls both ways. Like a spreadsheet's frozen panes,
//
//   the Region column (set_frozen_cols(1)) stays at the left while the months
//   scroll sideways; the double line after it marks the edge, and
//
//   the "All regions" totals row (set_frozen_rows(1)) stays under the header
//   while the regions scroll up and down; it is underlined.
//
// Arrows move the focused cell, PgUp/PgDn page, Home/End and Ctrl+Left/Right
// jump to the ends. Resize or zoom the window and the table follows it.
// Enter (or a double-click) on a cell shows its region, month and sales in a
// message box: the table sends its `on_select` command, which the
// application's `AppHandler` answers. Alt-X quits.

use turbo_vision::app::{AppHandler, Application};
use turbo_vision::core::command::{CM_QUIT, CommandId};
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::state::Grow;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::msgbox::message_box_ok;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::StatusLine;
use turbo_vision::views::table::{Column, Table, TableBuilder};
use turbo_vision::views::window::{Window, WindowBuilder};
use turbo_vision::views::{GroupLike, View, ViewId};

/// Sent by the table when a cell is chosen with Enter or a double-click.
const CM_SHOW_CELL: CommandId = 1000;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const REGIONS: [&str; 30] = [
    "Andalucía",
    "Aragón",
    "Asturias",
    "Baleares",
    "Canarias",
    "Cantabria",
    "Castilla-LM",
    "Castilla-León",
    "Catalunya",
    "Ceuta",
    "Extremadura",
    "Galicia",
    "La Rioja",
    "Madrid",
    "Melilla",
    "Murcia",
    "Navarra",
    "País Vasco",
    "Valencia",
    "Algarve",
    "Alentejo",
    "Lisboa",
    "Norte",
    "Centro",
    "Açores",
    "Madeira",
    "Occitanie",
    "Provence",
    "Bretagne",
    "Corse",
];

/// A made-up but steady figure for one region and month.
fn sales(region: usize, month: usize) -> u32 {
    let seed = u32::try_from((region * 37 + month * 11) % 90).unwrap_or(0);
    (seed + 10) * 10
}

/// Answers the table's `CM_SHOW_CELL` with a message box about the focused
/// cell. It keeps the rows it gave the table, to name the region of any cell.
struct SalesApp {
    rows: Vec<Vec<String>>,
    window: ViewId,
    table: ViewId,
}

impl SalesApp {
    /// The focused row and column, read from the table in its window.
    fn focus(&self, app: &mut Application) -> Option<(usize, usize)> {
        let table = app
            .desktop
            .child_by_id_mut(self.window)?
            .as_any_mut()
            .downcast_mut::<Window>()?
            .child_by_id_mut(self.table)?
            .as_any_mut()
            .downcast_mut::<Table>()?;
        Some((table.selected_row()?, table.selected_col()))
    }

    /// What the message box says about one cell. The Region column and the
    /// Total column stand for the whole year.
    fn describe(&self, row: usize, col: usize) -> String {
        let cells = &self.rows[row];
        let total = cells.len() - 1;
        let (period, sales) = match col {
            1..=12 => (MONTH_NAMES[col - 1], &cells[col]),
            _ => ("Whole year", &cells[total]),
        };
        format!("Region: {}\nMonth:  {period}\nSales:  {sales}", cells[0])
    }
}

impl AppHandler for SalesApp {
    fn handle_command(&mut self, app: &mut Application, command: CommandId, _: &Event) -> bool {
        if command != CM_SHOW_CELL {
            return false;
        }
        if let Some((row, col)) = self.focus(app) {
            message_box_ok(app, &self.describe(row, col));
        }
        true
    }
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let (width, height) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItemBuilder::new()
                .text("~Alt-X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
        ],
    ));

    // Region, twelve months and a total: about 110 cells wide.
    let mut columns = vec![Column::new("Region", 14)];
    columns.extend(MONTHS.iter().map(|m| Column::right(*m, 6)));
    columns.push(Column::right("Total", 7));

    // Every region's row, then the totals row put first.
    let mut totals = [0u32; 13];
    let mut rows: Vec<Vec<String>> = REGIONS
        .iter()
        .enumerate()
        .map(|(r, name)| {
            let months: Vec<u32> = (0..12).map(|m| sales(r, m)).collect();
            let total: u32 = months.iter().sum();
            for (sum, value) in totals.iter_mut().zip(months.iter().chain([&total])) {
                *sum += value;
            }
            std::iter::once((*name).to_string())
                .chain(months.iter().chain([&total]).map(u32::to_string))
                .collect()
        })
        .collect();
    rows.insert(
        0,
        std::iter::once("All regions".to_string())
            .chain(totals.iter().map(u32::to_string))
            .collect(),
    );

    let mut window = WindowBuilder::new()
        .bounds(Rect::new(4, 2, 76, 21))
        .title("Sales by region")
        .build();
    let mut hint = StaticText::new(
        Rect::new(1, 0, 70, 1),
        "Region and totals stay put. Enter on a cell shows its sales.",
    );
    hint.set_grow_mode(Grow::HI_X);
    window.add(hint);

    let mut table = TableBuilder::new()
        .bounds(Rect::new(1, 2, 69, 17))
        .columns(columns)
        .rows(rows.clone())
        .separators(true)
        .frozen_cols(1)
        .frozen_rows(1)
        .on_select(CM_SHOW_CELL)
        .build();
    // Follow the window's right and bottom edges when it is resized or
    // zoomed (Borland: gfGrowHiX | gfGrowHiY); without grow bits a child
    // keeps the size it was given.
    table.set_grow_mode(Grow::HI_X | Grow::HI_Y);
    let table = window.add(table);
    window.set_initial_focus();
    let window = app.desktop.add(window);

    app.run_with(&mut SalesApp {
        rows,
        window,
        table,
    });
    Ok(())
}
