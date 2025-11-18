// (C) 2025 - Enzo Lombardi
// Function Keys Test - Tests F1-F10 keyboard input
//
// This example demonstrates:
// - Function key detection (F1-F10)
// - Real-time key press display
// - Simple dialog-based UI

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{
    EventType, KB_ALT_X, KB_F1, KB_F2, KB_F3, KB_F4, KB_F5, KB_F6, KB_F7, KB_F8, KB_F9, KB_F10,
};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::colors;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::View;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![StatusItem::new("~Alt+X~ Quit", KB_ALT_X, CM_QUIT)],
    );
    app.set_status_line(status_line);

    let mut last_key = String::from("None");

    // Event loop
    app.running = true;
    while app.running {
        // Draw everything
        app.desktop.draw(&mut app.terminal);

        // Draw title
        let title = "Function Keys Test (F1-F10)";
        let mut buf = DrawBuffer::new(title.len());
        for (i, ch) in title.chars().enumerate() {
            buf.put_char(i, ch, colors::HIGHLIGHTED);
        }
        write_line_to_terminal(
            &mut app.terminal,
            (width as usize / 2 - title.len() / 2) as i16,
            2,
            &buf,
        );

        // Draw instructions
        let instructions = vec![
            "Press any function key F1-F10",
            "to test keyboard input.",
            "",
            "The last pressed key will be",
            "displayed below.",
        ];

        for (i, line) in instructions.iter().enumerate() {
            let mut buf = DrawBuffer::new(line.len());
            for (j, ch) in line.chars().enumerate() {
                buf.put_char(j, ch, colors::NORMAL);
            }
            write_line_to_terminal(
                &mut app.terminal,
                (width as usize / 2 - line.len() / 2) as i16,
                5 + i as i16,
                &buf,
            );
        }

        // Draw last key pressed
        let display = format!("Last Key: {}", last_key);
        let mut buf = DrawBuffer::new(display.len());
        for (i, ch) in display.chars().enumerate() {
            buf.put_char(i, ch, colors::SELECTED);
        }
        write_line_to_terminal(
            &mut app.terminal,
            (width as usize / 2 - display.len() / 2) as i16,
            12,
            &buf,
        );

        // Draw key reference
        let key_ref = vec![
            "F1  F2  F3  F4  F5",
            "F6  F7  F8  F9  F10",
        ];

        for (i, line) in key_ref.iter().enumerate() {
            let mut buf = DrawBuffer::new(line.len());
            for (j, ch) in line.chars().enumerate() {
                buf.put_char(j, ch, colors::NORMAL);
            }
            write_line_to_terminal(
                &mut app.terminal,
                (width as usize / 2 - line.len() / 2) as i16,
                15 + i as i16,
                &buf,
            );
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
