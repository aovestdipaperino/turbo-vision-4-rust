// (C) 2025 - Enzo Lombardi
// Extended menu example demonstrating:
// - Menu bar with submenus
// - Popup/context menu on right-click
// - Global keyboard shortcuts
// - Event handling patterns

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_NEW, CM_OK, CM_OPEN, CM_QUIT, CM_SAVE};
use turbo_vision::core::event::{
    Event, EventType, KB_ALT_X, KB_CTRL_C, KB_CTRL_N, KB_CTRL_O, KB_CTRL_S, KB_CTRL_V,
    KB_CTRL_X, KB_F10, MB_RIGHT_BUTTON,
};
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::menu_box::MenuBox;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::View;

// Custom command IDs for this example
const CMD_ABOUT: u16 = 100;
const CMD_CUT: u16 = 200;
const CMD_COPY: u16 = 201;
const CMD_PASTE: u16 = 202;
const CMD_GENERAL_PREFS: u16 = 204;
const CMD_APPEARANCE_PREFS: u16 = 205;
const CMD_SHORTCUTS_PREFS: u16 = 206;
const CMD_RECENT_1: u16 = 210;
const CMD_RECENT_2: u16 = 211;
const CMD_RECENT_3: u16 = 212;
const CMD_CLEAR_RECENT: u16 = 213;
const CMD_POPUP_NEW: u16 = 220;
const CMD_POPUP_OPEN: u16 = 221;
const CMD_POPUP_PROPERTIES: u16 = 222;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Setup UI components
    setup_menu_bar(&mut app, width);
    setup_status_line(&mut app, width, height);
    setup_welcome_message(&mut app, width, height);

    // Initial draw and show welcome dialog
    redraw_screen(&mut app);
    show_about(&mut app);
    redraw_screen(&mut app);

    // Main event loop
    run_event_loop(&mut app)?;

    Ok(())
}

/// Create and configure the menu bar with File, Edit, and Help menus
fn setup_menu_bar(app: &mut Application, width: u16) {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // File menu with Recent Files submenu
    let recent_files_submenu = Menu::from_items(vec![
        MenuItem::with_shortcut("~1~. document.txt", CMD_RECENT_1, 0, "", 0),
        MenuItem::with_shortcut("~2~. project.rs", CMD_RECENT_2, 0, "", 0),
        MenuItem::with_shortcut("~3~. readme.md", CMD_RECENT_3, 0, "", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~C~lear Recent", CMD_CLEAR_RECENT, 0, "", 0),
    ]);

    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, 0, "Ctrl+O", 0),
        MenuItem::submenu("~R~ecent Files", 0, recent_files_submenu, 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~S~ave", CM_SAVE, 0, "Ctrl+S", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    // Edit menu with Preferences submenu
    let preferences_submenu = Menu::from_items(vec![
        MenuItem::with_shortcut("~G~eneral", CMD_GENERAL_PREFS, 0, "", 0),
        MenuItem::with_shortcut("~A~ppearance", CMD_APPEARANCE_PREFS, 0, "", 0),
        MenuItem::with_shortcut("~K~eyboard Shortcuts", CMD_SHORTCUTS_PREFS, 0, "", 0),
    ]);

    let edit_menu_items = vec![
        MenuItem::with_shortcut("Cu~t~", CMD_CUT, 0, "Ctrl+X", 0),
        MenuItem::with_shortcut("~C~opy", CMD_COPY, 0, "Ctrl+C", 0),
        MenuItem::with_shortcut("~P~aste", CMD_PASTE, 0, "Ctrl+V", 0),
        MenuItem::separator(),
        MenuItem::submenu("P~r~eferences", 0, preferences_submenu, 0),
    ];
    let edit_menu = SubMenu::new("~E~dit", Menu::from_items(edit_menu_items));

    // Help menu
    let help_menu_items = vec![MenuItem::with_shortcut("~A~bout", CMD_ABOUT, 0, "F1", 0)];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));

    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(edit_menu);
    menu_bar.add_submenu(help_menu);
    app.set_menu_bar(menu_bar);
}

/// Create and configure the status line at the bottom of the screen
fn setup_status_line(app: &mut Application, width: u16, height: u16) {
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F10~ Menu", KB_F10, 0),
            StatusItem::new("~F1~ Help", 0, 0),
            StatusItem::new("~Right-Click~ Popup", 0, 0),
        ],
    );
    app.set_status_line(status_line);
}

/// Add a welcome message to the desktop background
fn setup_welcome_message(app: &mut Application, width: u16, height: u16) {
    let msg_width = 60;
    let msg_x = (width as i16 - msg_width) / 2;
    let msg = StaticTextBuilder::new()
        .bounds(Rect::new(
            msg_x,
            height as i16 / 2,
            msg_x + msg_width,
            height as i16 / 2 + 4,
        ))
        .text("Extended Menu Example\n\nTry the menu bar with submenus\nor right-click for a popup menu!")
        .centered(true)
        .build();
    app.desktop.add(Box::new(msg));
}

/// Redraw all UI components (desktop, menu bar, status line)
fn redraw_screen(app: &mut Application) {
    app.desktop.draw(&mut app.terminal);
    if let Some(ref mut menu_bar) = app.menu_bar {
        menu_bar.draw(&mut app.terminal);
    }
    if let Some(ref mut status_line) = app.status_line {
        status_line.draw(&mut app.terminal);
    }
    let _ = app.terminal.flush();
}

/// Main event loop - handles user input and dispatches commands
fn run_event_loop(app: &mut Application) -> turbo_vision::core::error::Result<()> {
    app.running = true;
    while app.running {
        // Redraw screen
        redraw_screen(app);

        // Poll for user input
        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            // Step 1: Convert global keyboard shortcuts to commands
            // (Ctrl+N, Ctrl+O, etc. work even when menus are closed)
            handle_global_shortcuts(&mut event);

            // Step 2: Let menu bar process events
            // (handles menu navigation, Alt+F, F10, etc.)
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);

                // Check for cascading submenus (e.g., Recent Files, Preferences)
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = Event::command(command);
                        }
                    }
                }
            }

            // Step 3: Handle right-click popup menu
            if event.what == EventType::MouseDown && event.mouse.buttons & MB_RIGHT_BUTTON != 0
            {
                let command = show_popup_menu(app, event.mouse.pos);
                if command != 0 {
                    event = Event::command(command);
                }
            }

            // Step 4: Execute commands (redraw first for clean dialog display)
            if event.what == EventType::Command {
                redraw_screen(app);
                handle_command(app, event.command);
            }
        }
    }

    Ok(())
}

/// Convert global keyboard shortcuts to command events
/// These shortcuts work regardless of whether menus are open
fn handle_global_shortcuts(event: &mut Event) {
    if event.what != EventType::Keyboard {
        return;
    }

    let command = match event.key_code {
        KB_CTRL_N => Some(CM_NEW),
        KB_CTRL_O => Some(CM_OPEN),
        KB_CTRL_S => Some(CM_SAVE),
        KB_CTRL_X => Some(CMD_CUT),
        KB_CTRL_C => Some(CMD_COPY),
        KB_CTRL_V => Some(CMD_PASTE),
        KB_ALT_X => Some(CM_QUIT),
        _ => None,
    };

    if let Some(cmd) = command {
        *event = Event::command(cmd);
    }
}

/// Dispatch commands to appropriate handlers
fn handle_command(app: &mut Application, command: u16) {
    match command {
        CM_QUIT => {
            app.running = false;
        }
        CM_NEW => {
            show_message(app, "New", "Create a new file");
        }
        CM_OPEN => {
            show_message(app, "Open", "Open an existing file");
        }
        CM_SAVE => {
            show_message(app, "Save", "Save the current file");
        }
        CMD_CUT => {
            show_message(app, "Cut", "Cut selection to clipboard");
        }
        CMD_COPY => {
            show_message(app, "Copy", "Copy selection to clipboard");
        }
        CMD_PASTE => {
            show_message(app, "Paste", "Paste from clipboard");
        }
        CMD_RECENT_1 => {
            show_message(app, "Recent File", "Opening: document.txt");
        }
        CMD_RECENT_2 => {
            show_message(app, "Recent File", "Opening: project.rs");
        }
        CMD_RECENT_3 => {
            show_message(app, "Recent File", "Opening: readme.md");
        }
        CMD_CLEAR_RECENT => {
            show_message(app, "Clear Recent", "Recent files list cleared");
        }
        CMD_GENERAL_PREFS => {
            show_message(app, "Preferences", "General preferences");
        }
        CMD_APPEARANCE_PREFS => {
            show_message(app, "Preferences", "Appearance preferences");
        }
        CMD_SHORTCUTS_PREFS => {
            show_message(app, "Preferences", "Keyboard shortcuts");
        }
        CMD_POPUP_NEW => {
            show_message(app, "Popup Menu", "New from popup menu");
        }
        CMD_POPUP_OPEN => {
            show_message(app, "Popup Menu", "Open from popup menu");
        }
        CMD_POPUP_PROPERTIES => {
            show_message(app, "Popup Menu", "Properties from popup menu");
        }
        CMD_ABOUT => {
            show_about(app);
        }
        _ => {}
    }
}

/// Show a popup/context menu at the specified position
/// Returns the command ID of the selected item, or 0 if cancelled
fn show_popup_menu(app: &mut Application, position: Point) -> u16 {
    let popup_menu = Menu::from_items(vec![
        MenuItem::with_shortcut("~N~ew File", CMD_POPUP_NEW, 0, "Ctrl+N", 0),
        MenuItem::with_shortcut("~O~pen File", CMD_POPUP_OPEN, 0, "Ctrl+O", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~P~roperties", CMD_POPUP_PROPERTIES, 0, "", 0),
    ]);

    let mut menu_box = MenuBox::new(position, popup_menu);
    menu_box.execute(&mut app.terminal)
}

/// Show a simple message dialog
fn show_message(app: &mut Application, title: &str, message: &str) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 40;
    let dialog_height = 7;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(
            dialog_x,
            dialog_y,
            dialog_x + dialog_width,
            dialog_y + dialog_height,
        ))
        .title(title)
        .build();

    // Add centered message text (coordinates are relative to dialog interior)
    let text_width = dialog_width - 4;
    let text = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, text_width, 2))
        .text(message)
        .centered(true)
        .build();
    dialog.add(Box::new(text));

    // Add centered OK button
    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2; // -2 for dialog frame
    let button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 3, button_x + button_width, 5))
        .title("  ~O~K  ")
        .command(CM_OK)
        .default(true)
        .build();
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

/// Show the About dialog
fn show_about(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 56;
    let dialog_height = 12;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(
            dialog_x,
            dialog_y,
            dialog_x + dialog_width,
            dialog_y + dialog_height,
        ))
        .title("Turbo Vision for Rust")
        .build();

    // Add centered about text
    let text_width = dialog_width - 4;
    let text = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, text_width, 7))
        .text("Welcome To Turbo Vision for Rust!\n\nExtended Menu Example\n\nFeatures:\n- Menu bar with nested submenus\n- Right-click popup/context menus")
        .centered(true)
        .build();
    dialog.add(Box::new(text));

    // Add centered OK button
    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 8, button_x + button_width, 10))
        .title("  ~O~K  ")
        .command(CM_OK)
        .default(true)
        .build();
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}
