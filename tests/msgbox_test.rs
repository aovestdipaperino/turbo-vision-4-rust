// (C) 2025 - Enzo Lombardi
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_NO, CM_OK, CM_YES};
use turbo_vision::views::msgbox::*;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Test 1: Information message with OK button
    println!("Test 1: Information message");
    let result = message_box(
        &mut app,
        "Welcome to MessageBox Test!\n\nThis is a simple information message.",
        MF_INFORMATION | MF_OK_BUTTON,
    );
    println!("User pressed: {}", if result == CM_OK { "OK" } else { "?" });

    // Test 2: Warning message with OK/Cancel
    println!("\nTest 2: Warning with OK/Cancel");
    let result = message_box(
        &mut app,
        "This is a warning message.\n\nDo you want to continue?",
        MF_WARNING | MF_OK_CANCEL,
    );
    println!(
        "User pressed: {}",
        if result == CM_OK { "OK" } else { "Cancel" }
    );

    // Test 3: Error message
    println!("\nTest 3: Error message");
    let result = message_box(
        &mut app,
        "An error has occurred!\n\nPlease check your input.",
        MF_ERROR | MF_OK_BUTTON,
    );
    println!("User pressed: {}", if result == CM_OK { "OK" } else { "?" });

    // Test 4: Confirmation with Yes/No/Cancel
    println!("\nTest 4: Confirmation with Yes/No/Cancel");
    let result = message_box(
        &mut app,
        "Do you want to save changes?",
        MF_CONFIRMATION | MF_YES_NO_CANCEL,
    );
    println!(
        "User pressed: {}",
        match result {
            CM_YES => "Yes",
            CM_NO => "No",
            CM_CANCEL => "Cancel",
            _ => "Unknown",
        }
    );

    // Test 5: Input box
    println!("\nTest 5: Input box");
    if let Some(name) = input_box(&mut app, "Enter Name", "~N~ame:", "John Doe", 50) {
        // Show confirmation with the entered name
        let msg = format!("You entered: {}", name);
        message_box(&mut app, &msg, MF_INFORMATION | MF_OK_BUTTON);
        println!("User entered: {}", name);
    } else {
        println!("User cancelled input");
    }

    // Test 6: Another input box for email
    println!("\nTest 6: Email input");
    if let Some(email) = input_box(&mut app, "Contact Information", "~E~mail:", "", 100) {
        let msg = format!("Email saved: {}", email);
        message_box(&mut app, &msg, MF_INFORMATION | MF_OK_BUTTON);
        println!("User entered email: {}", email);
    } else {
        println!("User cancelled email input");
    }

    println!("\nAll tests completed!");
    Ok(())
}
