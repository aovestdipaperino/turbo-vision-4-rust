// (C) 2025 - Enzo Lombardi
// Window Resize and Menu Shortcuts Demo
//
// This example demonstrates:
// - Window resizing by dragging the bottom-right corner
// - Menu shortcuts displayed right-aligned
// - Multiple resizable windows
// - Window dragging by title bar
//
// Usage:
//   cargo run --example window_resize_demo
//
// Instructions:
// - Click and drag window title bars to move them
// - Click and drag the bottom-right corner (last 2 columns, last row) to resize
// - Minimum size is enforced (16x6)
// - Use File menu to see keyboard shortcuts displayed
// - F10 to exit

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_OPEN, CM_SAVE};
use turbo_vision::core::event::{EventType, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::text_viewer::TextViewer;
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::views::View;

const CMD_ABOUT: u16 = 100;
const CMD_HELP: u16 = 101;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar with keyboard shortcuts
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // File menu with shortcuts
    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, 0, "Ctrl+O", 0),
        MenuItem::with_shortcut("~S~ave", CM_SAVE, 0, "Ctrl+S", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    // Window menu
    let window_menu_items = vec![
        MenuItem::new("~T~ile", 0, 0, 0),
        MenuItem::new("~C~ascade", 0, 0, 0),
        MenuItem::separator(),
        MenuItem::new("~N~ext", 0, 0, 0),
    ];
    let window_menu = SubMenu::new("~W~indow", Menu::from_items(window_menu_items));

    // Help menu with shortcuts
    let help_menu_items = vec![
        MenuItem::with_shortcut("~H~elp Index", CMD_HELP, 0, "F1", 0),
        MenuItem::separator(),
        MenuItem::new("~A~bout", CMD_ABOUT, 0, 0),
    ];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));

    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(window_menu);
    menu_bar.add_submenu(help_menu);
    app.set_menu_bar(menu_bar);

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F10~ Exit", KB_F10, CM_QUIT),
            StatusItem::new("~F1~ Help", 0, CMD_HELP),
        ],
    );
    app.set_status_line(status_line);

    // Create window 1 - Instructions
    let mut window1 = WindowBuilder::new()
        .bounds(Rect::new(2, 2, 50, 18))
        .title("Resize & Shortcuts Demo")
        .build();

    let instructions =
        "WINDOW RESIZING:\n\
        \n\
        Try resizing this window!\n\
        \n\
        1. Move your mouse to the\n\
           bottom-right corner\n\
        \n\
        2. Click and drag when you\n\
           see the last 2 columns\n\
           and last row\n\
        \n\
        3. The window will resize\n\
           as you drag\n\
        \n\
        MINIMUM SIZE: 16x6\n\
        (enforced automatically)";

    let mut text_viewer = TextViewer::new(Rect::new(1, 1, 46, 14))
        .with_scrollbars(false)
        .with_indicator(false);
    text_viewer.set_text(instructions);
    window1.add(Box::new(text_viewer));
    app.desktop.add(Box::new(window1));

    // Create window 2 - Menu Shortcuts Info
    let mut window2 = WindowBuilder::new()
        .bounds(Rect::new(52, 2, 100, 18))
        .title("Menu Shortcuts")
        .build();

    let shortcuts_info =
        "KEYBOARD SHORTCUTS:\n\
        \n\
        Open the File menu to see\n\
        keyboard shortcuts displayed\n\
        right-aligned:\n\
        \n\
        New      Ctrl+N\n\
        Open     Ctrl+O\n\
        Save     Ctrl+S\n\
        Exit     Alt+X\n\
        \n\
        Help menu also shows:\n\
        \n\
        Help     F1\n\
        \n\
        Shortcuts are visual aids\n\
        showing users what keys\n\
        to press.";

    let mut text_viewer2 = TextViewer::new(Rect::new(1, 1, 44, 14))
        .with_scrollbars(false)
        .with_indicator(false);
    text_viewer2.set_text(shortcuts_info);
    window2.add(Box::new(text_viewer2));
    app.desktop.add(Box::new(window2));

    // Create window 3 - Features
    let mut window3 = WindowBuilder::new()
        .bounds(Rect::new(26, 10, 76, 24))
        .title("Features")
        .build();

    let features =
        "FEATURES DEMONSTRATED:\n\
        \n\
        ✓ Window Resizing\n\
          - Drag bottom-right corner\n\
          - Minimum size enforced\n\
          - Child views auto-update\n\
        \n\
        ✓ Menu Shortcuts Display\n\
          - Right-aligned in menus\n\
          - Auto-width calculation\n\
          - Professional appearance\n\
        \n\
        ✓ Window Dragging\n\
          - Click title bar to drag\n\
          - Z-order management\n\
        \n\
        All matching Borland TV!";

    let mut text_viewer3 = TextViewer::new(Rect::new(1, 1, 48, 12))
        .with_scrollbars(false)
        .with_indicator(false);
    text_viewer3.set_text(features);
    window3.add(Box::new(text_viewer3));
    app.desktop.add(Box::new(window3));

    // Show welcome dialog
    show_welcome(&mut app);

    // Event loop
    app.running = true;
    while app.running {
        // Draw everything
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Poll for events
        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            // Desktop handles events (window drag, resize, z-order)
            app.desktop.handle_event(&mut event);

            // Menu bar handles events
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);
            }

            // Status line handles events
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Redraw before showing dialog
            if event.what == EventType::Command {
                app.desktop.draw(&mut app.terminal);
                if let Some(ref mut menu_bar) = app.menu_bar {
                    menu_bar.draw(&mut app.terminal);
                }
                if let Some(ref mut status_line) = app.status_line {
                    status_line.draw(&mut app.terminal);
                }
                let _ = app.terminal.flush();
            }

            // Handle commands
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => {
                        app.running = false;
                    }
                    CM_NEW => {
                        show_message(&mut app, "New File", "Create a new file\n(Ctrl+N)");
                    }
                    CM_OPEN => {
                        show_message(&mut app, "Open File", "Open an existing file\n(Ctrl+O)");
                    }
                    CM_SAVE => {
                        show_message(&mut app, "Save File", "Save the current file\n(Ctrl+S)");
                    }
                    CMD_ABOUT => {
                        show_about(&mut app);
                    }
                    CMD_HELP => {
                        show_help(&mut app);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn show_message(app: &mut Application, title: &str, message: &str) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 50;
    let dialog_height = 9;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height))
        .title(title)
        .build();

    let text = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, dialog_width - 4, 5))
        .text(message)
        .centered(true)
        .build();
    dialog.add(Box::new(text));

    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 6, button_x + button_width, 8))
        .title("  ~O~K  ")
        .command(0)
        .default(true)
        .build();
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

fn show_welcome(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 60;
    let dialog_height = 14;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height))
        .title("Welcome to Turbo Vision!")
        .build();

    let welcome_text =
        "Window Resize & Menu Shortcuts Demo\n\
        \n\
        This example demonstrates:\n\
        \n\
        • Window resizing (drag bottom-right corner)\n\
        • Menu keyboard shortcuts (displayed right-aligned)\n\
        • Window dragging (drag title bar)\n\
        • Multiple overlapping windows\n\
        \n\
        Try resizing the windows and check out the File menu!";

    let text = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, dialog_width - 4, 10))
        .text(welcome_text)
        .centered(true)
        .build();
    dialog.add(Box::new(text));

    let button_width = 12;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 11, button_x + button_width, 13))
        .title(" ~G~et Started ")
        .command(0)
        .default(true)
        .build();
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

fn show_about(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 55;
    let dialog_height = 12;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height))
        .title("About")
        .build();

    let about_text =
        "Turbo Vision for Rust\n\
        \n\
        Version 0.1.8\n\
        \n\
        Features:\n\
        • Status Line Hot Spots (v0.1.8)\n\
        • Window Resize Support (v0.1.6)\n\
        • Menu Shortcuts Display (v0.1.7)\n\
        \n\
        Based on Borland Turbo Vision";

    let text = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, dialog_width - 4, 9))
        .text(about_text)
        .centered(true)
        .build();
    dialog.add(Box::new(text));

    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 9, button_x + button_width, 11))
        .title("  ~O~K  ")
        .command(0)
        .default(true)
        .build();
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

fn show_help(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 60;
    let dialog_height = 16;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height))
        .title("Help (F1)")
        .build();

    let help_text =
        "WINDOW OPERATIONS:\n\
        \n\
        Move:   Click and drag the title bar\n\
        Resize: Click and drag the bottom-right corner\n\
                (Last 2 columns, last row)\n\
        Close:  Click the [■] button in the title bar\n\
        Focus:  Click anywhere on the window\n\
        \n\
        MENUS:\n\
        \n\
        Open:   Click on menu name or press Alt+Letter\n\
        Select: Use arrow keys or click on item\n\
        Close:  Press Esc or click outside menu\n\
        \n\
        Keyboard shortcuts are shown in menus!";

    let text = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, dialog_width - 4, 13))
        .text(help_text)
        .build();
    dialog.add(Box::new(text));

    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 13, button_x + button_width, 15))
        .title("  ~O~K  ")
        .command(0)
        .default(true)
        .build();
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}
