// (C) 2025 - Enzo Lombardi
// Standard Library Dialogs Demo
//
// Demonstrates the three standard library dialogs:
// - MessageBox (OK button)
// - ConfirmationBox (Yes/No/Cancel buttons)
// - InputBox (text input)

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_NO, CM_YES};
use turbo_vision::views::msgbox::{confirmation_box, confirmation_box_ok_cancel, confirmation_box_yes_no, input_box, message_box_error, message_box_ok, message_box_warning};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // 1. Simple message box
    message_box_ok(&mut app, "Welcome to the Dialogs Demo!");

    // 2. Error message box
    message_box_error(&mut app, "This is an error message.");

    // 3. Warning message box
    message_box_warning(&mut app, "This is a warning message.");

    // 4. Confirmation box with Yes/No/Cancel
    let result = confirmation_box(&mut app, "Do you want to save changes?");
    let action = match result {
        r if r == CM_YES => "You chose: Yes",
        r if r == CM_NO => "You chose: No",
        _ => "You chose: Cancel",
    };
    message_box_ok(&mut app, action);

    // 5. Confirmation box with Yes/No only
    let result = confirmation_box_yes_no(&mut app, "Continue with operation?");
    let action = match result {
        r if r == CM_YES => "Continuing...",
        _ => "Cancelled",
    };
    message_box_ok(&mut app, action);

    // 6. Confirmation box with OK/Cancel
    let result = confirmation_box_ok_cancel(&mut app, "Proceed with deletion?");
    let action = match result {
        r if r == turbo_vision::core::command::CM_OK => "Deleted!",
        _ => "Cancelled",
    };
    message_box_ok(&mut app, action);

    // 7. Input box
    if let Some(name) = input_box(&mut app, "Input Box", "Enter your name:", "", 50) {
        let greeting = format!("Hello, {name}!");
        message_box_ok(&mut app, &greeting);
    } else {
        message_box_ok(&mut app, "Input cancelled");
    }

    message_box_ok(&mut app, "Demo complete!");

    Ok(())
}
