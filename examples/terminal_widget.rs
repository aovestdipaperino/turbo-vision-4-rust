// (C) 2025 - Enzo Lombardi
// Terminal Widget Demo - demonstrates scrolling output viewer
//
// This matches Borland's terminal.cc example structure:
// - Terminal widget inside a Dialog/Window
// - With horizontal and vertical scrollbars
// - Buttons inside the same Dialog/Window

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_OK, CM_QUIT};
use turbo_vision::core::event::{EventType, KB_ALT_X};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::terminal_widget::TerminalWidget;
use turbo_vision::views::View;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Create dialog (matches Borland: TDialog(TRect(0,0,60,18),"Dumb terminal"))
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(10, 3, 70, 21))
        .title("Dumb Terminal")
        .build();

    // Create terminal widget inside dialog (matches Borland: TRect(1,1,57,12))
    // Note: Borland coordinates are relative to dialog interior
    let mut terminal = TerminalWidget::new(Rect::new(1, 1, 57, 12)).with_scrollbar();

    // Add initial content (matches Borland's do_sputn call)
    terminal.append_text("Hello!");
    terminal.append_text("That's just a test in the buffer.");
    terminal.append_text("That's all folks.");
    terminal.append_text("");
    terminal.append_text("Use arrow keys or PgUp/PgDn to scroll.");
    terminal.append_text("Auto-scrolls when at bottom.");

    dialog.add(Box::new(terminal));

    // Add OK button (matches Borland: TRect(25,15,35,17),"O~K~")
    dialog.add(Box::new(ButtonBuilder::new()
        .bounds(Rect::new(25, 13, 35, 15))
        .title("~O~K")
        .command(CM_OK)
        .default(true)
        .build()));

    app.desktop.add(Box::new(dialog));

    // Main event loop
    app.running = true;
    while app.running {
        app.desktop.draw(&mut app.terminal);

        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }

        let _ = app.terminal.flush();

        if let Some(mut event) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
            .ok()
            .flatten()
        {
            app.desktop.handle_event(&mut event);

            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            if event.what == EventType::Command {
                match event.command {
                    CM_OK | CM_QUIT => {
                        app.running = false;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
