// (C) 2025 - Enzo Lombardi
// Status Line Hot Spots Demo
//
// This example demonstrates:
// - Mouse hover highlighting on status line items
// - Context-sensitive help hints
// - Clickable status line items with visual feedback
// - Keyboard shortcuts for status line items
//
// Usage:
//   cargo run --example status_line_demo
//
// Instructions:
// - Hover mouse over status line items to see them highlight
// - Click status line items to execute commands
// - Press F1-F4 to trigger status line commands
// - Watch the hint text change as you interact with different parts

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_QUIT, CM_NEW, CM_OPEN, CM_SAVE};
use turbo_vision::core::event::{EventType, KB_F1, KB_F2, KB_F3, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::Window;
use turbo_vision::views::text_viewer::TextViewer;
use turbo_vision::views::View;

const CMD_HELP: u16 = 200;
const CMD_ABOUT: u16 = 201;
const CMD_DOCS: u16 = 202;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // File menu
    let file_menu_items = vec![
        MenuItem::with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N", 0),
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, 0, "Ctrl+O", 0),
        MenuItem::with_shortcut("~S~ave", CM_SAVE, 0, "Ctrl+S", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    // Help menu
    let help_menu_items = vec![
        MenuItem::with_shortcut("~H~elp Index", CMD_HELP, 0, "F1", 0),
        MenuItem::with_shortcut("~D~ocumentation", CMD_DOCS, 0, "F2", 0),
        MenuItem::separator(),
        MenuItem::new("~A~bout", CMD_ABOUT, 0, 0),
    ];
    let help_menu = SubMenu::new("~H~elp", Menu::from_items(help_menu_items));

    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(help_menu);
    app.set_menu_bar(menu_bar);

    // Create status line with hints
    let mut status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F1~ Help", KB_F1, CMD_HELP),
            StatusItem::new("~F2~ Docs", KB_F2, CMD_DOCS),
            StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
            StatusItem::new("~F10~ Exit", KB_F10, CM_QUIT),
        ],
    );

    // Set initial hint text
    status_line.set_hint(Some("Hover over or click status line items".to_string()));
    app.set_status_line(status_line);

    // Create demo window with instructions
    let mut window = Window::new(
        Rect::new(5, 2, 75, 22),
        "Status Line Hot Spots Demo"
    );

    let instructions =
        "STATUS LINE HOT SPOTS DEMO\n\
        \n\
        This demo showcases v0.1.8 status line improvements:\n\
        \n\
        FEATURES:\n\
        \n\
        1. Mouse Hover Highlighting\n\
           Move your mouse over the status line items at the\n\
           bottom. They will highlight in green when hovered.\n\
        \n\
        2. Clickable Hot Spots\n\
           Click on any status line item to execute its command.\n\
           Try clicking \"F1 Help\" or \"F3 Open\"!\n\
        \n\
        3. Keyboard Shortcuts\n\
           Press F1, F2, F3, or F10 to trigger the corresponding\n\
           status line command.\n\
        \n\
        4. Context-Sensitive Hints\n\
           The hint text on the right side of the status line\n\
           provides context-sensitive help. (Future: will update\n\
           based on focused control)\n\
        \n\
        This matches Borland's TStatusLine::drawSelect() pattern!";

    let mut text_viewer = TextViewer::new(Rect::new(1, 1, 68, 18))
        .with_scrollbars(false)
        .with_indicator(false);
    text_viewer.set_text(instructions);
    window.add(Box::new(text_viewer));
    app.desktop.add(Box::new(window));

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
                        show_message(&mut app, "New File", "Create a new file\n(Ctrl+N)\n\nStatus line item clicked!");
                    }
                    CM_OPEN => {
                        show_message(&mut app, "Open File", "Open an existing file\n(F3)\n\nStatus line item clicked!");
                    }
                    CM_SAVE => {
                        show_message(&mut app, "Save File", "Save the current file\n(Ctrl+S)\n\nStatus line item clicked!");
                    }
                    CMD_HELP => {
                        show_help(&mut app);
                    }
                    CMD_DOCS => {
                        show_message(&mut app, "Documentation", "Show documentation\n(F2)\n\nStatus line item clicked!");
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

fn show_message(app: &mut Application, title: &str, message: &str) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 50;
    let dialog_height = 11;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        title
    );

    let text = StaticText::new_centered(
        Rect::new(2, 1, dialog_width - 4, 7),
        message
    );
    dialog.add(Box::new(text));

    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = Button::new(
        Rect::new(button_x, 8, button_x + button_width, 10),
        "  ~O~K  ",
        0,
        true
    );
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

fn show_welcome(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 60;
    let dialog_height = 13;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Welcome to v0.1.8!"
    );

    let welcome_text =
        "Status Line Hot Spots Demo\n\
        \n\
        New in v0.1.8:\n\
        \n\
        • Mouse hover highlighting on status line items\n\
        • Context-sensitive hint display\n\
        • Improved clickable hot spots\n\
        \n\
        Try hovering over or clicking the status line items\n\
        at the bottom of the screen!";

    let text = StaticText::new_centered(
        Rect::new(2, 1, dialog_width - 4, 9),
        welcome_text
    );
    dialog.add(Box::new(text));

    let button_width = 12;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = Button::new(
        Rect::new(button_x, 10, button_x + button_width, 12),
        " ~G~et Started ",
        0,
        true
    );
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

fn show_about(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 55;
    let dialog_height = 13;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "About"
    );

    let about_text =
        "Turbo Vision for Rust\n\
        \n\
        Version 0.1.8\n\
        \n\
        New in this version:\n\
        • Status line hot spots with hover highlighting\n\
        • Context-sensitive help hints\n\
        • Improved mouse interaction\n\
        \n\
        Matches Borland's TStatusLine::drawSelect() pattern";

    let text = StaticText::new_centered(
        Rect::new(2, 1, dialog_width - 4, 9),
        about_text
    );
    dialog.add(Box::new(text));

    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = Button::new(
        Rect::new(button_x, 10, button_x + button_width, 12),
        "  ~O~K  ",
        0,
        true
    );
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}

fn show_help(app: &mut Application) {
    let (term_width, term_height) = app.terminal.size();

    let dialog_width = 60;
    let dialog_height = 18;
    let dialog_x = (term_width as i16 - dialog_width) / 2;
    let dialog_y = (term_height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Help (F1)"
    );

    let help_text =
        "STATUS LINE OPERATIONS:\n\
        \n\
        Mouse Hover:\n\
          Move your mouse over status line items.\n\
          Items highlight in green when hovered.\n\
        \n\
        Mouse Click:\n\
          Click on any status line item to execute\n\
          its associated command.\n\
        \n\
        Keyboard Shortcuts:\n\
          F1  - Help Index\n\
          F2  - Documentation\n\
          F3  - Open File\n\
          F10 - Exit\n\
        \n\
        The status line provides quick access to\n\
        frequently used commands!";

    let text = StaticText::new(
        Rect::new(2, 1, dialog_width - 4, 15),
        help_text
    );
    dialog.add(Box::new(text));

    let button_width = 10;
    let button_x = (dialog_width - 2 - button_width) / 2;
    let button = Button::new(
        Rect::new(button_x, 15, button_x + button_width, 17),
        "  ~O~K  ",
        0,
        true
    );
    dialog.add(Box::new(button));
    dialog.set_initial_focus();

    dialog.execute(app);
}
