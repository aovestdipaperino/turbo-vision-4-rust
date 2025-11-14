// (C) 2025 - Enzo Lombardi
// From README.md

use turbo_vision::prelude::*;

fn main() -> turbo_vision::core::error::Result<()> {
    // Create a window
    let mut dialog = turbo_vision::views::dialog::DialogBuilder::new().bounds(Rect::new(10, 5, 50, 15)).title("My First Dialog").build();

    // Create a button and add it to the window
    let button = turbo_vision::views::button::Button::new(Rect::new(26, 6, 36, 8), "Quit", turbo_vision::core::command::CM_OK, true);
    dialog.add(Box::new(button));

    // Create the application and add the dialog to its desktop
    let mut app = Application::new()?;
    app.desktop.add(Box::new(dialog));

    // Event loop
    app.running = true;
    while app.running {
        app.desktop.draw(&mut app.terminal);
        app.terminal.flush()?;
        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            app.desktop.handle_event(&mut event);
            if event.command == CM_OK {
                // Handle button click
                app.running = false;
            }
        }
    }
    Ok(())
}
