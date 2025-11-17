// (C) 2025 - Enzo Lombardi
// Function Keys Test - Tests F1-F10 keyboard input
//
// This example demonstrates:
// - Function key detection (F1-F10)
// - Real-time key press display
// - Simple dialog-based UI
//
// Note : under WIN11 F11 => full screen

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{EventType, KB_ALT_X, KB_F1, KB_F2, KB_F3, KB_F4, KB_F5, KB_F6, KB_F7, KB_F8, KB_F9, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Attr;
use turbo_vision::views::View;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::view::write_line_to_terminal;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let status_line = setup_status_line(&app);
    app.set_status_line(status_line);

    let (width, _) = app.terminal.size();
    let width = width as i16;
    let mut last_key = String::from("None");
    app.running = true;

    // Event loop
    while app.running {
        // Draw everything
        app.desktop.draw(&mut app.terminal);

        // Configuration for the "box"
        let box_width = 40;
        let box_x = (width - box_width) / 2;

        // Draw title box
        let title = " Function Keys Test (F1-F10)            ";
        let title_y = 2;
        let title_x = (width - title.len() as i16) / 2;
        let mut buf = DrawBuffer::new(title.len());
        for (i, ch) in title.chars().enumerate() {
            buf.put_char(i, ch, Attr::from_u8(0x0F)); // White on black
        }
        write_line_to_terminal(&mut app.terminal, title_x, title_y, &buf);

        // Draw instructions box
        let instructions = [
            " Press any function key F1-F10 ",
            " to test keyboard input.       ",
            "                               ",
            " The last pressed key will be  ",
            " displayed below.              ",
        ];

        let instr_y = 5;
        for (i, line) in instructions.iter().enumerate() {
            let mut buf = DrawBuffer::new(box_width as usize);
            for j in 0..box_width as usize {
                if j < line.len() {
                    buf.put_char(j, line.chars().nth(j).unwrap(), Attr::from_u8(0x0F)); // White on black
                } else {
                    buf.put_char(j, ' ', Attr::from_u8(0x0F));
                }
            }
            write_line_to_terminal(&mut app.terminal, box_x, instr_y + i as i16, &buf);
        }

        // Draw last key box
        let key_display = format!(" Last Key: {:15} ", last_key);
        let key_y = 12;
        let mut buf = DrawBuffer::new(box_width as usize);
        for j in 0..box_width as usize {
            if j < key_display.len() {
                buf.put_char(j, key_display.chars().nth(j).unwrap(), Attr::from_u8(0x3F)); // Cyan on black
            } else {
                buf.put_char(j, ' ', Attr::from_u8(0x3F));
            }
        }
        write_line_to_terminal(&mut app.terminal, box_x, key_y, &buf);

        // Draw key reference box
        let key_ref = [" F1  F2  F3  F4  F5             ", " F6  F7  F8  F9  F10            "];

        let ref_y = 15;
        for (i, line) in key_ref.iter().enumerate() {
            let mut buf = DrawBuffer::new(box_width as usize);
            for j in 0..box_width as usize {
                if j < line.len() {
                    buf.put_char(j, line.chars().nth(j).unwrap(), Attr::from_u8(0x0F)); // White on black
                } else {
                    buf.put_char(j, ' ', Attr::from_u8(0x0F));
                }
            }
            write_line_to_terminal(&mut app.terminal, box_x, ref_y + i as i16, &buf);
        }

        // Draw status line
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }

        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            // Status line handles shortcuts
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Handle commands
            match event.what {
                EventType::Command => {
                    if event.command == CM_QUIT {
                        app.running = false;
                    }
                }
                EventType::Keyboard => {
                    // Check for function keys
                    match event.key_code {
                        KB_F1 => last_key = "F1".to_string(),
                        KB_F2 => last_key = "F2".to_string(),
                        KB_F3 => last_key = "F3".to_string(),
                        KB_F4 => last_key = "F4".to_string(),
                        KB_F5 => last_key = "F5".to_string(),
                        KB_F6 => last_key = "F6".to_string(),
                        KB_F7 => last_key = "F7".to_string(),
                        KB_F8 => last_key = "F8".to_string(),
                        KB_F9 => last_key = "F9".to_string(),
                        KB_F10 => last_key = "F10".to_string(),
                        _ => {}
                    }
                    event.clear();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Create and configure the status line at the bottom of the screen
fn setup_status_line(app: &Application) -> StatusLine {
    let (w, h) = app.terminal.size();

    StatusLine::new(Rect::new(0, h as i16 - 1, w as i16, h as i16), vec![StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT)])
}
