// (C) 2025 - Enzo Lombardi
// Add a menu bar but no action behind yet

use turbo_vision::core::event::{KB_ALT_X, KB_ESC, KB_ESC_ESC};
use turbo_vision::prelude::*;

use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};

use turbo_vision::views::status_line::{StatusItem, StatusLine};

// Custom command IDs for this example
const CMD_ABOUT: u16 = 100;

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
            StatusItem::new("~Esc-X~ Exit", KB_ESC, CM_QUIT),
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Esc-Esc~ Exit", KB_ESC_ESC, CM_QUIT),
        ],
    )
}

/// Create and configure the menu bar with File and Help menus
fn setup_menu_bar(app: &Application) -> MenuBar {
    let file_menu_items = vec![
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, 0, "Ctrl+O", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    let help_menu_items = vec![
        MenuItem::with_shortcut("~A~bout", CMD_ABOUT, 0, "F1", 0), //
    ];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));

    let (w, _) = app.terminal.size();
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, w as i16, 1));
    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(help_menu);
    menu_bar
}
