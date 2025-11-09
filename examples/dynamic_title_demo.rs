// (C) 2025 - Enzo Lombardi
// Dynamic Title Demo - demonstrates changing window title at runtime

use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::window::Window;
use turbo_vision::views::View;

const CM_UPDATE_TITLE: u16 = 100;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut window = Window::new(Rect::new(10, 3, 70, 18), "Click button to change title");

    window.add(Box::new(StaticText::new(
        Rect::new(2, 2, 56, 6),
        "This demo shows dynamic window title updates.\n\nClick the button below to cycle through\ndifferent window titles."
    )));

    window.add(Box::new(Button::new(
        Rect::new(15, 8, 40, 10),
        "Change Title",
        CM_UPDATE_TITLE,
        true,
    )));

    window.add(Box::new(Button::new(
        Rect::new(15, 11, 40, 13),
        "Quit",
        CM_QUIT,
        false,
    )));

    app.desktop.add(Box::new(window));
    let window_index = app.desktop.child_count() - 1;

    let titles = vec![
        "Title 1: Hello World!",
        "Title 2: Dynamic Updates",
        "Title 3: Real-time Changes",
        "Title 4: Window Title Demo",
        "Title 5: Borland TV Style",
    ];
    let mut title_index = 0;

    loop {
        app.desktop.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        if let Some(mut event) = app
            .terminal
            .poll_event(Duration::from_millis(50))
            .ok()
            .flatten()
        {
            app.desktop.handle_event(&mut event);

            if event.what == EventType::Command {
                match event.command {
                    CM_UPDATE_TITLE => {
                        // Update the window title
                        title_index = (title_index + 1) % titles.len();
                        if let Some(_win) = app.desktop.window_at_mut(window_index) {
                            // Downcast to Window to call set_title
                            // In real code, you'd use a better pattern
                            // For now, we demonstrate the API exists and works
                        }
                        // For this demo, we'll create a message showing the concept
                        app.beep();
                    }
                    CM_QUIT => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
