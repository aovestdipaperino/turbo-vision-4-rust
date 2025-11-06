// (C) 2025 - Enzo Lombardi
// Suspend/Resume Demo - demonstrates terminal suspend/resume functionality

use turbo_vision::app::Application;
use turbo_vision::views::window::Window;
use turbo_vision::views::button::Button;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::View;
use turbo_vision::helpers::msgbox::{message_box, MF_INFORMATION, MF_OK_BUTTON};
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use std::time::Duration;

const CM_SUSPEND: u16 = 100;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut window = Window::new(
        Rect::new(10, 3, 70, 18),
        "Suspend/Resume Demo"
    );

    window.add(Box::new(StaticText::new(
        Rect::new(2, 2, 56, 8),
        "This demo shows suspend/resume functionality.\n\n\
         Click 'Suspend' to temporarily return to shell.\n\
         The application will exit raw mode and show your\n\
         shell prompt. Type 'fg' to resume."
    )));

    window.add(Box::new(Button::new(
        Rect::new(15, 9, 35, 11),
        "Suspend",
        CM_SUSPEND,
        false
    )));

    window.add(Box::new(Button::new(
        Rect::new(15, 12, 35, 14),
        "Quit",
        CM_QUIT,
        true
    )));

    app.desktop.add(Box::new(window));

    loop {
        app.desktop.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)).ok().flatten() {
            app.desktop.handle_event(&mut event);

            if event.what == EventType::Command {
                match event.command {
                    CM_SUSPEND => {
                        // Suspend the application
                        app.suspend()?;

                        // At this point, the terminal is in normal mode
                        // The user can use the shell, and when they type 'fg',
                        // we'll continue here

                        // For this demo, we'll immediately resume
                        // In a real implementation with signal handlers,
                        // the process would be stopped here (SIGSTOP)
                        // and resumed later (SIGCONT)

                        println!("Application suspended. Press Enter to resume...");
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        // Resume the application
                        app.resume()?;

                        // Show a message that we're back
                        message_box(&mut app, "Welcome back! Application resumed.", MF_INFORMATION | MF_OK_BUTTON);
                    }
                    CM_QUIT => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
