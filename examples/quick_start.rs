use turbo_vision::prelude::*;
fn main() -> turbo_vision::core::error::Result<()> {
    // Create application with terminal
    let mut app = Application::new()?;
    // Create a window
    let mut window =
        turbo_vision::views::window::Window::new(Rect::new(10, 5, 50, 15), "My First Window");
    // Create a button
    let button = turbo_vision::views::button::Button::new(
        Rect::new(15, 5, 25, 7),
        "Click Me",
        turbo_vision::core::command::CM_OK,
        false,
    );
    // Add the button to the window
    window.add(Box::new(button));
    // Add the window to the desktop
    app.desktop.add(Box::new(window));
    // Run event loop
    app.running = true;
    while app.running {
        app.desktop.draw(&mut app.terminal);
        app.terminal.flush()?;
        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            app.desktop.handle_event(&mut event);
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => app.running = false,
                    CM_OK => {
                        // Handle button click
                        app.running = false;
                    }
                    _ => {}
                }
            }
        }
    }
    app.terminal.shutdown()?;
    Ok(())
}
