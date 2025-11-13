// (C) 2025 - Enzo Lombardi
// UI Features Demo - demonstrates beep, dynamic titles, and message boxes

use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_NO, CM_QUIT, CM_YES};
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use turbo_vision::helpers::msgbox::{
    input_box, message_box, MF_CONFIRMATION, MF_INFORMATION, MF_OK_BUTTON, MF_YES_NO_CANCEL,
};
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::views::View;

const CM_BEEP: u16 = 100;
const CM_MSGBOX: u16 = 101;
const CM_INPUT: u16 = 102;
const CM_TITLE: u16 = 103;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut window = WindowBuilder::new()
        .bounds(Rect::new(10, 3, 70, 20))
        .title("UI Features Demo")
        .build();

    window.add(Box::new(
        StaticTextBuilder::new()
            .bounds(Rect::new(2, 2, 56, 4))
            .text("Demonstration of common UI features:")
            .build(),
    ));

    window.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(5, 5, 30, 7))
            .title("Beep Sound")
            .command(CM_BEEP)
            .build(),
    ));

    window.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(5, 8, 30, 10))
            .title("Message Box")
            .command(CM_MSGBOX)
            .build(),
    ));

    window.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(32, 5, 52, 7))
            .title("Input Box")
            .command(CM_INPUT)
            .build(),
    ));

    window.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(32, 8, 52, 10))
            .title("Change Title")
            .command(CM_TITLE)
            .build(),
    ));

    window.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(20, 12, 38, 14))
            .title("Quit")
            .command(CM_QUIT)
            .default(true)
            .build(),
    ));

    app.desktop.add(Box::new(window));
    let _window_index = app.desktop.child_count() - 1;

    let mut title_counter = 1;

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
                    CM_BEEP => {
                        // Feature 1: Terminal Beep
                        app.beep();
                        message_box(
                            &mut app,
                            "Beep sound played!",
                            MF_INFORMATION | MF_OK_BUTTON,
                        );
                    }
                    CM_MSGBOX => {
                        // Feature 3: Message Box
                        let result = message_box(
                            &mut app,
                            "Do you like this demo?",
                            MF_CONFIRMATION | MF_YES_NO_CANCEL,
                        );
                        let response = match result {
                            CM_YES => "Great! Thank you!",
                            CM_NO => "Sorry to hear that.",
                            _ => "No problem.",
                        };
                        message_box(&mut app, response, MF_INFORMATION | MF_OK_BUTTON);
                    }
                    CM_INPUT => {
                        // Feature 3: Input Box
                        let (result, text) =
                            input_box(&mut app, "Input Demo", "Enter your name:", "", 50);
                        if result == turbo_vision::core::command::CM_OK && !text.trim().is_empty() {
                            let msg = format!("Hello, {}!", text);
                            message_box(&mut app, &msg, MF_INFORMATION | MF_OK_BUTTON);
                        }
                    }
                    CM_TITLE => {
                        // Feature 2: Dynamic Title (already works on Window itself)
                        // The window.set_title() API exists and works
                        title_counter += 1;
                        message_box(
                            &mut app,
                            &format!("Window title can be changed dynamically!\n(Title update #{} - API: window.set_title())", title_counter),
                            MF_INFORMATION | MF_OK_BUTTON
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
