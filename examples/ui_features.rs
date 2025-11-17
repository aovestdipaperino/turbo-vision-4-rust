// (C) 2025 - Enzo Lombardi
// UI Features Demo - demonstrates beep, dynamic titles, and message boxes

use turbo_vision::prelude::*;

use std::time::Duration;
use turbo_vision::core::command::{CM_NO, CM_QUIT, CM_YES};
use turbo_vision::core::event::KB_ALT_X;
use turbo_vision::helpers::msgbox::{MF_CONFIRMATION, MF_INFORMATION, MF_OK_BUTTON, MF_YES_NO_CANCEL, input_box, message_box};
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;

const CMD_BEEP: u16 = 100;
const CMD_MSGBOX: u16 = 101;
const CMD_INPUT: u16 = 102;
const CMD_TITLE: u16 = 103;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut dialog = DialogBuilder::new().bounds(Rect::new(10, 3, 70, 20)).title("UI Features Demo").build();
    dialog.add(Box::new(StaticTextBuilder::new().bounds(Rect::new(2, 2, 56, 4)).text("Demonstration of common UI features:").build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(5, 5, 30, 7)).title("Beep Sound").command(CMD_BEEP).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(5, 8, 30, 10)).title("Message Box").command(CMD_MSGBOX).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(32, 5, 52, 7)).title("Input Box").command(CMD_INPUT).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(32, 8, 52, 10)).title("Change Title").command(CMD_TITLE).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(20, 12, 38, 14)).title("Quit").command(CM_QUIT).default(true).build()));

    let mut app = Application::new()?;
    app.desktop.add(Box::new(dialog));

    loop {
        app.desktop.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)).ok().flatten() {
            app.desktop.handle_event(&mut event);

            // Alt+X to leave
            if event.what == EventType::Keyboard && event.key_code == KB_ALT_X {
                break;
            }

            if event.what == EventType::Command {
                match event.command {
                    CMD_BEEP => {
                        // Feature 1: Terminal Beep
                        app.beep();
                        message_box(&mut app, "Beep sound played!", MF_INFORMATION | MF_OK_BUTTON);
                    }
                    CMD_MSGBOX => {
                        // Feature 3: Message Box
                        let result = message_box(&mut app, "Do you like this demo?", MF_CONFIRMATION | MF_YES_NO_CANCEL);
                        let response = match result {
                            CM_YES => "Great! Thank you!",
                            CM_NO => "Sorry to hear that.",
                            _ => "No problem.",
                        };
                        message_box(&mut app, response, MF_INFORMATION | MF_OK_BUTTON);
                    }
                    CMD_INPUT => {
                        // Feature 3: Input Box
                        let (result, text) = input_box(&mut app, "Input Demo", "Enter your name:", "", 50);
                        if result == turbo_vision::core::command::CM_OK && !text.trim().is_empty() {
                            let msg = format!("Hello, {}!", text);
                            message_box(&mut app, &msg, MF_INFORMATION | MF_OK_BUTTON);
                        }
                    }
                    CMD_TITLE => {
                        // message_box is fixed in sized 40x9 => 3 lines of 32 chars max.
                        message_box(&mut app, &format!("Change window title dynamically.\nSee examples/dynamic_title.rs"), MF_INFORMATION | MF_OK_BUTTON);
                    }
                    CM_QUIT => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
