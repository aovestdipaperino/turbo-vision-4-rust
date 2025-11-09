// (C) 2025 - Enzo Lombardi
use turbo_vision::prelude::*;
use turbo_vision::views::{button::Button, dialog::Dialog, static_text::StaticText};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create a dialog with OK and Cancel buttons
    // Dialog interior is 40 wide x 10 tall (excluding frame)
    let mut dialog = Dialog::new(Rect::new(20, 8, 60, 18), "Confirm Action");

    // Add a message (relative coordinates within dialog interior)
    let text = StaticText::new(Rect::new(2, 1, 36, 4), "Are you sure you want to\nproceed with this action?");
    dialog.add(Box::new(text));

    // Add OK button (default) - positioned relative to dialog interior
    let ok_button = Button::new(
        Rect::new(6, 5, 16, 7),
        "  ~O~K  ",
        CM_OK,
        true, // This is the default button
    );
    dialog.add(Box::new(ok_button));

    // Add Cancel button - positioned relative to dialog interior
    let cancel_button = Button::new(Rect::new(22, 5, 32, 7), "~C~ancel", CM_CANCEL, false);
    dialog.add(Box::new(cancel_button));

    // Set focus to the first button
    dialog.set_initial_focus();

    // Execute the dialog and get the result
    let result = dialog.execute(&mut app);

    // Print the result (after terminal is shut down)
    drop(app);

    match result {
        CM_OK => println!("User clicked OK"),
        CM_CANCEL => println!("User clicked Cancel"),
        _ => println!("Dialog closed with result: {}", result),
    }

    Ok(())
}
