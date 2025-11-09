// (C) 2025 - Enzo Lombardi
// Extended menu example demonstrating:
// - Menu bar with submenus
// - Popup/context menu on right-click

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_NEW, CM_OK, CM_OPEN, CM_QUIT, CM_SAVE};
use turbo_vision::core::event::{EventType, KB_F10, MB_RIGHT_BUTTON};
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::menu_box::MenuBox;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::View;

// Custom command IDs for this example
const CMD_ABOUT: u16 = 100;
const CMD_CUT: u16 = 200;
const CMD_COPY: u16 = 201;
const CMD_PASTE: u16 = 202;
//const CMD_PREFERENCES: u16 = 203;
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

    // Create menu bar
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

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F10~ Menu", KB_F10, CM_QUIT),
            StatusItem::new("~Right-Click~ Popup", 0, 0),
        ],
    );
    app.set_status_line(status_line);

    // Add welcome message to desktop
    let msg_width = 60;
    let msg_x = (width as i16 - msg_width) / 2;
    let msg = StaticText::new_centered(
        Rect::new(
            msg_x,
            height as i16 / 2,
            msg_x + msg_width,
            height as i16 / 2 + 4,
        ),
        "Extended Menu Example\n\nTry the menu bar with submenus\nor right-click for a popup menu!",
    );
    app.desktop.add(Box::new(msg));

    // Draw initial screen
    app.desktop.draw(&mut app.terminal);
    if let Some(ref mut menu_bar) = app.menu_bar {
        menu_bar.draw(&mut app.terminal);
    }
    if let Some(ref mut status_line) = app.status_line {
        status_line.draw(&mut app.terminal);
    }
    let _ = app.terminal.flush();

    // Show about dialog on startup
    show_about(&mut app);

    // Redraw after dialog closes
    app.desktop.draw(&mut app.terminal);
    if let Some(ref mut menu_bar) = app.menu_bar {
        menu_bar.draw(&mut app.terminal);
    }
    if let Some(ref mut status_line) = app.status_line {
        status_line.draw(&mut app.terminal);
    }
    let _ = app.terminal.flush();

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
            // Menu bar handles events first
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);

                // Check if a cascading submenu should be shown
                // Only check if event wasn't fully processed (Enter/MouseUp on submenu item)
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = turbo_vision::core::event::Event::command(command);
                        }
                    }
                }
            }

            // Handle desktop events (like right-click for popup menu)
            if event.what == EventType::MouseDown && event.mouse.buttons & MB_RIGHT_BUTTON != 0 {
                // Show popup menu at mouse position
                let command = show_popup_menu(&mut app, event.mouse.pos);
                if command != 0 {
                    event = turbo_vision::core::event::Event::command(command);
                }
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
                        show_message(&mut app, "New", "Create a new file");
                    }
                    CM_OPEN => {
                        show_message(&mut app, "Open", "Open an existing file");
                    }
                    CM_SAVE => {
                        show_message(&mut app, "Save", "Save the current file");
                    }
                    CMD_CUT => {
                        show_message(&mut app, "Cut", "Cut selection to clipboard");
                    }
                    CMD_COPY => {
                        show_message(&mut app, "Copy", "Copy selection to clipboard");
                    }
                    CMD_PASTE => {
                        show_message(&mut app, "Paste", "Paste from clipboard");
                    }
                    CMD_RECENT_1 => {
                        show_message(&mut app, "Recent File", "Opening: document.txt");
                    }
                    CMD_RECENT_2 => {
                        show_message(&mut app, "Recent File", "Opening: project.rs");
                    }
                    CMD_RECENT_3 => {
                        show_message(&mut app, "Recent File", "Opening: readme.md");
                    }
                    CMD_CLEAR_RECENT => {
                        show_message(&mut app, "Clear Recent", "Recent files list cleared");
                    }
                    CMD_GENERAL_PREFS => {
                        show_message(&mut app, "Preferences", "General preferences");
                    }
                    CMD_APPEARANCE_PREFS => {
                        show_message(&mut app, "Preferences", "Appearance preferences");
                    }
                    CMD_SHORTCUTS_PREFS => {
                        show_message(&mut app, "Preferences", "Keyboard shortcuts");
                    }
                    CMD_POPUP_NEW => {
                        show_message(&mut app, "Popup Menu", "New from popup menu");
                    }
                    CMD_POPUP_OPEN => {
                        show_message(&mut app, "Popup Menu", "Open from popup menu");
                    }
                    CMD_POPUP_PROPERTIES => {
                        show_message(&mut app, "Popup Menu", "Properties from popup menu");
                    }
                    CMD_ABOUT => {
                        show_about(&mut app);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn show_popup_menu(app: &mut Application, position: Point) -> u16 {
    // Create popup/context menu
    let popup_menu = Menu::from_items(vec![
        MenuItem::with_shortcut("~N~ew File", CMD_POPUP_NEW, 0, "Ctrl+N", 0),
        MenuItem::with_shortcut("~O~pen File", CMD_POPUP_OPEN, 0, "Ctrl+O", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~P~roperties", CMD_POPUP_PROPERTIES, 0, "", 0),
    ]);

    // Create menu box at mouse position
    let mut menu_box = MenuBox::new(position, popup_menu);

    // Execute the popup menu modally
    menu_box.execute(&mut app.terminal)
}

fn show_message(app: &mut Application, title: &str, message: &str) {
    let (term_width, term_height) = app.terminal.size();

    // Dialog dimensions
    let dialog_width = 40;
    let dialog_height = 7;

    // Center dialog on screen
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(
            dialog_x,
            dialog_y,
            dialog_x + dialog_width,
            dialog_y + dialog_height,
        ),
        title,
    );

    // Text positioned relative to dialog interior (coordinates are relative)
    let text_width = dialog_width - 4; // Leave margin
    let text = StaticText::new_centered(Rect::new(2, 1, text_width, 2), message);
    dialog.add(Box::new(text));

    // Center button horizontally
    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2; // -2 for frame
    let button = Button::new(
        Rect::new(button_x, 3, button_x + button_width, 5),
        "  ~O~K  ",
        CM_OK,
        true,
    );
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

fn show_about(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();

    // Dialog dimensions
    let dialog_width = 56;
    let dialog_height = 12;

    // Center dialog on screen
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(
            dialog_x,
            dialog_y,
            dialog_x + dialog_width,
            dialog_y + dialog_height,
        ),
        "Turbo Vision for Rust",
    );

    // Text positioned relative to dialog interior (coordinates are relative)
    let text_width = dialog_width - 4; // Leave margin
    let text = StaticText::new_centered(
        Rect::new(2, 1, text_width, 7),
        "Welcome To Turbo Vision for Rust!\n\nExtended Menu Example\n\nFeatures:\n- Menu bar with nested submenus\n- Right-click popup/context menus",
    );
    dialog.add(Box::new(text));

    // Center button horizontally
    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2; // -2 for frame
    let button = Button::new(
        Rect::new(button_x, 8, button_x + button_width, 10),
        "  ~O~K  ",
        CM_OK,
        true,
    );
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}
