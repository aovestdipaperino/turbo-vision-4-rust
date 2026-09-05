// (C) 2025 - Enzo Lombardi
// Suspend/Resume Demo - demonstrates terminal suspend/resume functionality

use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::GroupLike;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::msgbox::{MsgBox, message_box};
use turbo_vision::views::static_text::StaticTextBuilder;

const CMD_SUSPEND: u16 = 100;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(10, 3, 65, 18))
        .title("Suspend/Resume Demo")
        .build();

    dialog.add(
        StaticTextBuilder::new()
            .bounds(Rect::new(2, 2, 53, 8))
            .text(
                "Demonstrates the suspend/resume functionality.\n\n\
                Click 'Suspend' to temporarily return to shell.\n\
                The application will exit raw mode and show your\n\
                shell prompt. Press 'Enter' to resume.",
            )
            .build(),
    );

    dialog.add(
        ButtonBuilder::new()
            .bounds(Rect::new(33, 9, 44, 11))
            .title("Quit")
            .command(CM_QUIT)
            .default(true)
            .build(),
    );
    dialog.add(
        ButtonBuilder::new()
            .bounds(Rect::new(11, 9, 22, 11))
            .title("Suspend")
            .command(CMD_SUSPEND)
            .build(),
    );

    app.desktop.add(dialog);

    loop {
        app.terminal.draw_view(&mut app.desktop);
        let _ = app.terminal.flush();

        if let Some(mut event) = app
            .terminal
            .poll_event(Duration::from_millis(50))
            .ok()
            .flatten()
        {
            turbo_vision::views::view::dispatch_to_child(&mut app.desktop, &mut event);

            if event.what == EventType::Command {
                match event.command {
                    CMD_SUSPEND => {
                        // Suspend the application
                        app.suspend()?;

                        // At this point, the terminal is in normal mode
                        // The user can use the shell, and when they press 'Enter',
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
                        message_box(
                            &mut app,
                            "Welcome back! Application resumed.",
                            MsgBox::INFORMATION | MsgBox::OK_BUTTON,
                        );
                    }
                    CM_QUIT => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
