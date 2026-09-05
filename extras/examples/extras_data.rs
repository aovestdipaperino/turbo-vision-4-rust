// (C) 2026 - Enzo Lombardi
// Extras Data Example
// Demonstrates the lazy data views from turbo-vision-extras:
// - GridView browsing 100,000 generated rows (tvDMX style)
// - VirtualListBox scrolling a one-million-item provider
//
// Neither control materializes more than the visible viewport, so both
// open instantly. Arrows/PgUp/PgDn/Home/End navigate; Tab switches
// windows via the desktop (F6 in classic TV; here click the other window).

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::KB_ESC_ESC;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::GroupLike;
use turbo_vision::views::status_line::StatusLine;
use turbo_vision::views::window::WindowBuilder;
use turbo_vision_extras::{GridColumn, GridView, ListProvider, RowProvider, VirtualListBox};

/// 100,000 synthetic inventory rows, computed on demand.
struct Inventory;

impl RowProvider for Inventory {
    fn rows(&self) -> usize {
        100_000
    }

    fn cell(&self, row: usize, col: usize) -> String {
        match col {
            0 => format!("{:06}", row + 1),
            1 => format!("Part {}", ["Alpha", "Bravo", "Charlie", "Delta"][row % 4]),
            2 => format!("{}", 3 + (row * 7) % 90),
            3 => format!("{}.{:02}", (row * 13) % 500, (row * 31) % 100),
            _ => String::new(),
        }
    }
}

/// One million lines, computed on demand.
struct Million;

impl ListProvider for Million {
    fn len(&self) -> usize {
        1_000_000
    }

    fn item(&self, index: usize) -> String {
        format!("Log line {:07}: everything is fine", index + 1)
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
            StatusItemBuilder::new()
                .text("~Esc-Esc~ Exit")
                .key_code(KB_ESC_ESC)
                .command(CM_QUIT)
                .build(),
        ],
    ));

    // Grid over 100k rows
    let mut grid_window = WindowBuilder::new()
        .bounds(Rect::new(2, 2, 46, 18))
        .title("Inventory (100,000 rows)")
        .build();
    grid_window.add(GridView::new(
        Rect::new(1, 1, 41, 14),
        vec![
            GridColumn::new("Id", 7),
            GridColumn::new("Name", 14),
            GridColumn::new("Qty", 5),
            GridColumn::new("Price", 9),
        ],
        Box::new(Inventory),
        1001, // broadcast on Enter/double-click with the row in event.info
    ));
    app.desktop.add(grid_window);

    // Virtual list over a million items
    let mut list_window = WindowBuilder::new()
        .bounds(Rect::new(48, 4, 90, 20))
        .title("Log (1,000,000 lines)")
        .build();
    list_window.add(VirtualListBox::new(
        Rect::new(1, 1, 39, 14),
        Box::new(Million),
        1002,
    ));
    app.desktop.add(list_window);

    app.run();
    Ok(())
}
