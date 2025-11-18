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

use turbo_vision::core::command::{CM_NEW, CM_OPEN, CM_QUIT, CM_SAVE};
use turbo_vision::core::event::{KB_ALT_X, KB_CTRL_C, KB_CTRL_N, KB_CTRL_O, KB_CTRL_S, KB_ESC_ESC, KB_F1, KB_F10};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::prelude::*;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::msgbox::message_box_ok;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::text_viewer::TextViewer;
use turbo_vision::views::window::WindowBuilder;

const CMD_ABOUT: u16 = 100;
const CMD_HELP: u16 = 101;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    setup_menu_bar(&mut app);
    setup_status_line(&mut app);
    setup_win1(&mut app);
    setup_win2(&mut app);
    setup_win3(&mut app);

    // Welcome dialog
    show_welcome(&mut app);

    run_event_loop(&mut app);

    Ok(())
}

/// Main event loop - handles user input and dispatches commands
fn run_event_loop(app: &mut Application) {
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
        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            // Convert global keyboard shortcuts to commands so that F1, Ctrl+N etc. work even when menus are closed
            handle_global_shortcuts(&mut event);

            // IMPORTANT: Event handling order matters!
            // When a menu is open, it must handle events BEFORE the desktop to capture arrow keys.
            // Otherwise, the active window would consume arrow keys instead of the menu.
            //
            // Order when menu is open:
            //   1. Menu bar (consumes arrow keys for navigation)
            //   2. Desktop (only processes events the menu didn't consume)
            //
            // Order when menu is closed:
            //   1. Menu bar (captures Alt+F, Alt+W, etc. to open menus)
            //   2. Desktop (handles window interactions, arrow keys for scrolling)
            //
            // This matches Borland Turbo Vision behavior where menus have priority when active.

            // Menu bar handles events FIRST (priority when menu is open)
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);
            }

            // Desktop handles events AFTER menu bar (only if not consumed)
            app.desktop.handle_event(&mut event);

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
                        message_box_ok(app, "Create a new file (Ctrl+N)");
                    }
                    CM_OPEN => {
                        message_box_ok(app, "Open an existing file (Ctrl+O)");
                    }
                    CM_SAVE => {
                        message_box_ok(app, "Save the current file (Ctrl+S)");
                    }
                    CMD_ABOUT => {
                        show_about(app);
                    }
                    CMD_HELP => {
                        show_help(app);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Convert global keyboard shortcuts to command events
/// These shortcuts work regardless of whether menus are open or not
fn handle_global_shortcuts(event: &mut Event) {
    if event.what != EventType::Keyboard {
        return;
    }

    let command = match event.key_code {
        KB_CTRL_N => Some(CM_NEW),
        KB_CTRL_O => Some(CM_OPEN),
        KB_CTRL_S => Some(CM_SAVE),
        KB_ALT_X | KB_CTRL_C | KB_ESC_ESC => Some(CM_QUIT),
        KB_F1 => Some(CMD_HELP),
        _ => None,
    };

    if let Some(cmd) = command {
        *event = Event::command(cmd);
    }
}

/// Create and configure the menu bar
fn setup_menu_bar(app: &mut Application) {
    let (width, _) = app.terminal.size();

    // Create menu bar with keyboard shortcuts
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width, 1));

    // File menu with shortcuts
    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, 0, "Ctrl+O", 0),
        MenuItem::with_shortcut("~S~ave", CM_SAVE, 0, "Ctrl+S", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    // Window menu - No action behind
    let window_menu_items = vec![
        MenuItem::new("~T~ile", 0, 0, 0),
        MenuItem::new("~C~ascade", 0, 0, 0),
        MenuItem::separator(),
        MenuItem::new("~N~ext", 0, 0, 0),
    ];
    let window_menu = SubMenu::new("~W~indow", Menu::from_items(window_menu_items));

    // Help menu with shortcuts
    let help_menu_items = vec![
        MenuItem::with_shortcut("~H~elp", CMD_HELP, 0, "F1", 0),
        MenuItem::separator(),
        MenuItem::new("~A~bout", CMD_ABOUT, 0, 0),
    ];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));

    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(window_menu);
    menu_bar.add_submenu(help_menu);
    app.set_menu_bar(menu_bar);
}

/// Create and configure the status line at the bottom of the screen
fn setup_status_line(app: &mut Application) {
    let (width, height) = app.terminal.size();
    let status_line = StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![StatusItem::new("~F10~ Menu", KB_F10, 0), StatusItem::new("~F1~ Help", 0, CMD_HELP)],
    );
    app.set_status_line(status_line);
}

/// Create a window with a text viewer
///
/// This helper function creates a window containing a TextViewer with the given content.
/// The TextViewer automatically fills the window's client area with appropriate margins.
fn create_text_window(app: &mut Application, bounds: Rect, title: &str, content: &str) {
    let mut window = WindowBuilder::new().bounds(bounds).title(title).build();

    // Calculate TextViewer bounds to fit inside window with margins
    let text_viewer_bounds = Rect::new(1, 1, bounds.width() - 2, bounds.height() - 4);
    let mut text_viewer = TextViewer::new(text_viewer_bounds).with_scrollbars(false).with_indicator(false);

    text_viewer.set_text(content);
    window.add(Box::new(text_viewer));
    app.desktop.add(Box::new(window));
}

/// Create window 1 - Instructions
fn setup_win1(app: &mut Application) {
    let instructions = r"WINDOW RESIZING:

1. Move your mouse to the bottom-right corner
2. Click and drag when you see the last 2 columns and last row
3. The window will resize as you drag

Note: minimum size = 16x6
";
    create_text_window(app, Rect::new(2, 2, 50, 18), "Resize & Shortcuts Demo", instructions);
}

// Create window 2 - Menu Shortcuts Info
fn setup_win2(app: &mut Application) {
    let shortcuts_info = r"KEYBOARD SHORTCUTS:

Shortcuts are visual aids showing users what keys to press.

File menu shows:
New      Ctrl+N
Open     Ctrl+O
Save     Ctrl+S
Exit     Alt+X

Help menu shows:
Help     F1

Note: F1 can be pressed anytime to show help, but About must be accessed by opening the Help menu.
";
    create_text_window(app, Rect::new(52, 2, 100, 18), "Menu Shortcuts", shortcuts_info);
}

// Create window 3 - Features
fn setup_win3(app: &mut Application) {
    let features = r"FEATURES DEMONSTRATED:

✓ Window Resizing
  - Drag bottom-right corner
  - Minimum size enforced
  - Child views auto-update

✓ Menu Shortcuts Display
  - Right-aligned in menus
  - Auto-width calculation
  - Professional appearance

✓ Window Dragging
  - Click title bar to drag
  - Z-order management

All matching Borland TV!";
    create_text_window(app, Rect::new(26, 10, 76, 24), "Features", features);
}

/// Display a modal dialog box with text content and an OK button
///
/// Creates a centered dialog containing the given text and an OK button.
/// The dialog is modal and blocks until the user dismisses it.
fn show_msg(app: &mut Application, text: &str, title: &str, dialog_width: i16, dialog_height: i16) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_x = (term_width - dialog_width) / 2;
    let dialog_y = (term_height - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height))
        .title(title)
        .build();

    let text = StaticTextBuilder::new().bounds(Rect::new(2, 1, dialog_width - 4, dialog_height - 6)).text(text).build();
    dialog.add(Box::new(text));

    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, dialog_height - 4, button_x + button_width, dialog_height - 2))
        .title("~O~K")
        .command(CM_OK)
        .default(true)
        .build();
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

/// Prepare content of the welcome dialog box
fn show_welcome(app: &mut Application) {
    let dialog_width = 60;
    let dialog_height = 15;

    let title = "Window Resize & Menu Shortcuts Demo";
    let text = r"This example demonstrates:

• Window resizing (drag bottom-right corner)
• Menu keyboard shortcuts (displayed right-aligned)
• Window dragging (drag title bar)
• Multiple overlapping windows

Resize the windows and check out the menus.";

    show_msg(app, text, title, dialog_width, dialog_height);
}

/// Prepare content of the about dialog box
fn show_about(app: &mut Application) {
    let dialog_width = 55;
    let dialog_height = 14;

    let title = "About";
    let text = r"Turbo Vision for Rust: Version 0.1.8

Features:
• Status Line Hot Spots (v0.1.8)
• Window Resize Support (v0.1.6)
• Menu Shortcuts Display (v0.1.7)
• Based on Borland Turbo Vision";

    show_msg(app, text, title, dialog_width, dialog_height);
}

/// Prepare content of the help dialog box
fn show_help(app: &mut Application) {
    let dialog_width = 60;
    let dialog_height = 19;

    let title = "Help";
    let text = r"WINDOW OPERATIONS:
Move:   Click and drag the title bar
Resize: Click and drag the bottom-right corner
Close:  Click the [■] button in the title bar
Focus:  Click anywhere on the window

MENUS:
Open:   Click on menu name or press Alt+Letter
Select: Use arrow keys or click on item
Close:  Press Esc or click outside menu

Keyboard shortcuts are shown in menus.";
    show_msg(app, text, title, dialog_width, dialog_height);
}
