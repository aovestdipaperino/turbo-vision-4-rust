// (C) 2025 - Enzo Lombardi
// Comprehensive example demonstrating List Components
//
// This example shows:
// - ListBox with ListViewer trait
// - MenuBar with MenuViewer trait
// - MenuBox popup menu
// - Menu building with MenuBuilder

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_NEW, CM_OPEN, CM_QUIT, CM_SAVE};
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::menu_data::MenuBuilder;
use turbo_vision::views::listbox::ListBox;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::menu_box::MenuBox;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::View;

// Custom command IDs
const CMD_SHOW_MENU: u16 = 100;
const CMD_LIST_SELECT: u16 = 101;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar using MenuBuilder
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // File menu with MenuBuilder
    let file_menu = MenuBuilder::new()
        .item_with_shortcut("~N~ew", CM_NEW, 0, "Ctrl+N")
        .item_with_shortcut("~O~pen...", CM_OPEN, 0, "Ctrl+O")
        .item_with_shortcut("~S~ave", CM_SAVE, 0, "Ctrl+S")
        .separator()
        .item("E~x~it", CM_QUIT, 0)
        .build();

    // Help menu
    let help_menu = MenuBuilder::new().item("~A~bout", CMD_SHOW_MENU, 0).build();

    menu_bar.add_submenu(SubMenu::new("~F~ile", file_menu));
    menu_bar.add_submenu(SubMenu::new("~H~elp", help_menu));
    app.set_menu_bar(menu_bar);

    // Create ListBox demonstrating ListViewer trait
    let mut listbox = ListBox::new(Rect::new(5, 3, 35, 13), CMD_LIST_SELECT);
    listbox.set_items(vec![
        "First Item".to_string(),
        "Second Item".to_string(),
        "Third Item".to_string(),
        "Fourth Item".to_string(),
        "Fifth Item".to_string(),
        "Sixth Item".to_string(),
        "Seventh Item".to_string(),
        "Eighth Item".to_string(),
        "Ninth Item".to_string(),
        "Tenth Item".to_string(),
        "Eleventh Item".to_string(),
        "Twelfth Item".to_string(),
    ]);

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~↑~↓~ Navigate", 0, 0),
            StatusItem::new("~Enter~ Select", 0, 0),
            StatusItem::new("~F1~ Popup Menu", 0, CMD_SHOW_MENU),
            StatusItem::new("~F10~ Quit", 0, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Draw instructions
    let instructions = vec![
        "List Components Demo",
        "",
        "ListBox (left):",
        "- Use ↑/↓ to navigate",
        "- Use PgUp/PgDn for pages",
        "- Use Home/End for first/last",
        "- Press Enter to select",
        "",
        "MenuBar (top):",
        "- Click or press Alt+F for File",
        "- Press Alt+H for Help",
        "- Use ↑/↓ in dropdown",
        "- Press letter for accelerator",
        "",
        "Press F1 for popup MenuBox",
    ];

    // Event loop
    app.running = true;
    while app.running {
        // Draw everything
        app.desktop.draw(&mut app.terminal);

        // Draw listbox
        listbox.draw(&mut app.terminal);

        // Draw instructions
        for (i, line) in instructions.iter().enumerate() {
            use turbo_vision::core::draw::DrawBuffer;
            use turbo_vision::core::palette::colors;
            use turbo_vision::views::view::write_line_to_terminal;

            let mut buf = DrawBuffer::new(line.len());
            for (j, ch) in line.chars().enumerate() {
                buf.put_char(j, ch, colors::DIALOG_NORMAL);
            }
            write_line_to_terminal(&mut app.terminal, 40, 3 + i as i16, &buf);
        }

        // Draw menu bar
        if let Some(ref mut menu_bar) = app.menu_bar {
            menu_bar.draw(&mut app.terminal);
        }

        // Draw status line
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }

        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            // MenuBar gets first chance
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);
            }

            // Then listbox
            if event.what != EventType::Nothing && event.what != EventType::Command {
                listbox.handle_event(&mut event);
            }

            // Then status line
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Handle commands
            match event.what {
                EventType::Command => {
                    match event.command {
                        CM_QUIT => app.running = false,
                        CM_NEW => {
                            // Show message (in real app would open new file)
                            app.desktop.draw(&mut app.terminal);
                            show_message(&mut app, "New file created", 20, 10);
                        }
                        CM_OPEN => {
                            show_message(&mut app, "Open file dialog", 20, 10);
                        }
                        CM_SAVE => {
                            show_message(&mut app, "Save file", 20, 10);
                        }
                        CMD_SHOW_MENU => {
                            // Demonstrate MenuBox popup
                            let popup_menu = MenuBuilder::new()
                                .item("~N~ew Window", CM_NEW, 0)
                                .item("~C~lose Window", 102, 0)
                                .separator()
                                .item("~R~efresh", 103, 0)
                                .build();

                            let mut menubox = MenuBox::new(Point::new(35, 8), popup_menu);
                            let selected_cmd = menubox.execute(&mut app.terminal);

                            // Redraw after popup closes
                            app.desktop.draw(&mut app.terminal);
                            listbox.draw(&mut app.terminal);

                            if selected_cmd != 0 {
                                let msg = match selected_cmd {
                                    CM_NEW => "New Window selected",
                                    //102 => "Close Window selected",
                                    103 => "Refresh selected",
                                    _ => "Unknown command",
                                };
                                show_message(&mut app, msg, 20, 10);
                            }
                        }
                        CMD_LIST_SELECT => {
                            if let Some(selected) = listbox.get_selected_item() {
                                let msg = format!("Selected: {}", selected);
                                show_message(&mut app, &msg, 20, 10);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Show a simple message box
fn show_message(app: &mut Application, message: &str, width: i16, height: i16) {
    use turbo_vision::core::draw::DrawBuffer;
    use turbo_vision::core::palette::colors;
    use turbo_vision::views::view::write_line_to_terminal;

    let (term_width, term_height) = app.terminal.size();
    let x = (term_width as i16 - width) / 2;
    let y = (term_height as i16 - height) / 2;

    // Draw border
    let mut top = DrawBuffer::new(width as usize);
    top.put_char(0, '┌', colors::DIALOG_NORMAL);
    for i in 1..width as usize - 1 {
        top.put_char(i, '─', colors::DIALOG_NORMAL);
    }
    top.put_char(width as usize - 1, '┐', colors::DIALOG_NORMAL);
    write_line_to_terminal(&mut app.terminal, x, y, &top);

    // Draw message
    for i in 1..height - 1 {
        let mut line = DrawBuffer::new(width as usize);
        line.put_char(0, '│', colors::DIALOG_NORMAL);
        for j in 1..width as usize - 1 {
            line.put_char(j, ' ', colors::DIALOG_NORMAL);
        }
        line.put_char(width as usize - 1, '│', colors::DIALOG_NORMAL);

        if i == height / 2 {
            // Center the message
            let msg_x = (width as usize - message.len()) / 2;
            for (j, ch) in message.chars().enumerate() {
                line.put_char(msg_x + j, ch, colors::DIALOG_NORMAL);
            }
        }

        write_line_to_terminal(&mut app.terminal, x, y + i, &line);
    }

    // Draw bottom border
    let mut bottom = DrawBuffer::new(width as usize);
    bottom.put_char(0, '└', colors::DIALOG_NORMAL);
    for i in 1..width as usize - 1 {
        bottom.put_char(i, '─', colors::DIALOG_NORMAL);
    }
    bottom.put_char(width as usize - 1, '┘', colors::DIALOG_NORMAL);
    write_line_to_terminal(&mut app.terminal, x, y + height - 1, &bottom);

    let _ = app.terminal.flush();

    // Wait for keypress
    loop {
        if let Ok(Some(event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            if event.what == EventType::Keyboard {
                break;
            }
        }
    }
}
