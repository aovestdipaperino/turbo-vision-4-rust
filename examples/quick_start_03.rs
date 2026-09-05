// (C) 2025 - Enzo Lombardi
// Add a menu bar but no action behind yet

use turbo_vision::core::menu_data::MenuItemBuilder;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::prelude::*;

use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};

use turbo_vision::views::status_line::StatusLine;

// Custom command IDs for this example
const CMD_ABOUT: u16 = 100; // [100, 255] + [1_000, 65_535]

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let status_line = setup_status_line(&app);
    app.set_status_line(status_line);

    // Add a menu bar
    let menu_bar = setup_menu_bar(&app);
    app.set_menu_bar(menu_bar);

    app.run();
    Ok(())
}

/// Create and configure the status line at the bottom of the screen
fn setup_status_line(app: &Application) -> StatusLine {
    let (w, h) = app.terminal.size();

    StatusLine::new(
        Rect::new(0, h as i16 - 1, w as i16, h as i16),
        vec![
            StatusItemBuilder::new()
                .text("~Alt-X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
        ],
    )
}

/// Create and configure the menu bar with File and Help menus
fn setup_menu_bar(app: &Application) -> MenuBar {
    let file_menu_items = vec![
        MenuItemBuilder::new()
            .text("~O~pen...")
            .command(CM_OPEN)
            .key("Ctrl+O")
            .build(),
        MenuItem::separator(),
        MenuItemBuilder::new()
            .text("E~x~it")
            .command(CM_QUIT)
            .key("Alt+X")
            .build(),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    let help_menu_items = vec![
        MenuItemBuilder::new()
            .text("~A~bout")
            .command(CMD_ABOUT)
            .key("F1")
            .build(), //
    ];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));

    let (w, _) = app.terminal.size();
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, w as i16, 1));
    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(help_menu);
    menu_bar
}
