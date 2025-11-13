// (C) 2025 - Enzo Lombardi
// ESC Feature Demo - demonstrates configurable ESC timeout for ESC+letter sequences

use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_OK};
use turbo_vision::core::geometry::Rect;
use turbo_vision::helpers::msgbox::{message_box, MF_ERROR, MF_INFORMATION, MF_OK_BUTTON};
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::input_line::InputLineBuilder;
use turbo_vision::views::label::LabelBuilder;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create a dialog asking for ESC timeout value
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(15, 5, 65, 16))
        .title("ESC Timeout Configuration")
        .build();

    // Instructions
    dialog.add(Box::new(
        LabelBuilder::new()
            .bounds(Rect::new(2, 2, 48, 3))
            .text("Configure the ESC timeout (250-1500 ms)")
            .build(),
    ));

    dialog.add(Box::new(
        LabelBuilder::new()
            .bounds(Rect::new(2, 3, 48, 4))
            .text("This controls ESC+letter key detection")
            .build(),
    ));

    // Timeout input field
    let timeout_data = Rc::new(RefCell::new(String::from("500")));
    let mut timeout_label = LabelBuilder::new()
        .bounds(Rect::new(2, 5, 20, 5))
        .text("~T~imeout (ms):")
        .build();

    timeout_label.set_link(dialog.add(Box::new(
        InputLineBuilder::new()
            .bounds(Rect::new(20, 5, 30, 5))
            .max_length(4)
            .data(Rc::clone(&timeout_data))
            .build(),
    )));
    dialog.add(Box::new(timeout_label));

    // Help text
    dialog.add(Box::new(
        LabelBuilder::new()
            .bounds(Rect::new(2, 7, 48, 8))
            .text("Valid range: 250 to 1500 milliseconds")
            .build(),
    ));

    // Buttons
    dialog.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(13, 9, 23, 11))
            .title("  OK  ")
            .command(CM_OK)
            .default(true)
            .build(),
    ));
    dialog.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(25, 9, 35, 11))
            .title("Cancel")
            .command(CM_CANCEL)
            .build(),
    ));

    dialog.set_initial_focus();

    let result = dialog.execute(&mut app);

    if result == CM_OK {
        let timeout_str = timeout_data.borrow();

        // Parse and validate the timeout
        match timeout_str.parse::<u64>() {
            Ok(timeout_ms) => {
                // Try to set the timeout with validation
                match app.set_esc_timeout(timeout_ms) {
                    Ok(()) => {
                        // Success! Show instructions for testing
                        let message = format!(
                            "ESC timeout set to {} ms\n\n\
                             Test it now:\n\
                             - Press ESC followed by a letter (e.g., ESC+F)\n\
                             - If typed within {}ms, it will be treated as Alt+letter\n\
                             - Otherwise, it will be treated as two separate keys\n\n\
                             Press ESC+X or F10 to exit when done testing.",
                            timeout_ms, timeout_ms
                        );
                        message_box(&mut app, &message, MF_INFORMATION | MF_OK_BUTTON);

                        // Run the app so user can test the ESC timeout
                        app.run();
                    }
                    Err(e) => {
                        // Validation failed - show error
                        let error_msg = format!("Error: {}", e);
                        message_box(&mut app, &error_msg, MF_ERROR | MF_OK_BUTTON);
                    }
                }
            }
            Err(_) => {
                // Parse failed - show error
                let error_msg = format!(
                    "Invalid input: '{}'\n\nPlease enter a number between 250 and 1500.",
                    timeout_str
                );
                message_box(&mut app, &error_msg, MF_ERROR | MF_OK_BUTTON);
            }
        }
    }

    println!("Final ESC timeout configuration: {}", timeout_data.borrow());

    Ok(())
}
