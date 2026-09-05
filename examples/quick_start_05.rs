// (C) 2025 - Enzo Lombardi
// Support global shortcuts (press F1 to see it in action)

use turbo_vision::core::event::{KB_ALT_X, KB_CTRL_O, KB_F1};
use turbo_vision::core::menu_data::MenuItemBuilder;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::prelude::*;

use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};

use turbo_vision::views::msgbox::message_box_ok;
use turbo_vision::views::status_line::StatusLine;

// Custom command IDs for this example
const CMD_ABOUT: u16 = 100; // [100, 255] + [1_000, 65_535]

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Add status line
    let status_line = setup_status_line(&app);
    app.set_status_line(status_line);

    let menu_bar = setup_menu_bar(&app);
    app.set_menu_bar(menu_bar);

    run_event_loop(&mut app);
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

fn run_event_loop(app: &mut Application) {
    app.running = true;
    while app.running {
        redraw_screen(app);

        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            // order matters: global > menu > status > command
            handle_global_shortcuts(&mut event);

            if let Some(ref mut menu_bar) = app.menu_bar {
                turbo_vision::views::view::dispatch_to_child(menu_bar, &mut event); // Handle F10 when pressed
            }

            if let Some(ref mut status_line) = app.status_line {
                turbo_vision::views::view::dispatch_to_child(status_line, &mut event);
            }

            if event.what == EventType::Command {
                redraw_screen(app);
                handle_command(app, event.command);
            }
        }
    }
}

/// Dispatch commands to appropriate handlers
fn handle_command(app: &mut Application, command: u16) {
    match command {
        CM_QUIT => {
            app.running = false;
        }

        CM_OPEN => {
            message_box_ok(app, "Open...");
        }

        CMD_ABOUT => {
            message_box_ok(app, "About...");
        }
        _ => {}
    }
}

fn redraw_screen(app: &mut Application) {
    app.terminal.draw_view(&mut app.desktop);
    if let Some(ref mut menu_bar) = app.menu_bar {
        app.terminal.draw_view(menu_bar);
    }
    if let Some(ref mut status_line) = app.status_line {
        app.terminal.draw_view(status_line);
    }
    let _ = app.terminal.flush();
}

/// Convert global keyboard shortcuts to command events
/// These shortcuts work regardless of whether menus are open or not
fn handle_global_shortcuts(event: &mut Event) {
    if event.what != EventType::Keyboard {
        return;
    }

    let command = match event.key_code {
        KB_CTRL_O => Some(CM_OPEN),
        KB_ALT_X => Some(CM_QUIT),
        KB_F1 => Some(CMD_ABOUT),
        _ => None,
    };

    if let Some(cmd) = command {
        *event = Event::command(cmd);
    }
}
