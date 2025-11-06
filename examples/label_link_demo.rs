// (C) 2025 - Enzo Lombardi
// Label Link Demo - demonstrates clicking labels to focus linked input fields

use turbo_vision::app::Application;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::button::Button;
use turbo_vision::views::Label;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::core::command::{CM_OK, CM_CANCEL};
use turbo_vision::core::geometry::Rect;
use std::rc::Rc;
use std::cell::RefCell;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create a dialog demonstrating label links
    let mut dialog = Dialog::new(
        Rect::new(15, 5, 65, 18),
        "Label Link Demo"
    );

    // Instructions
    dialog.add(Box::new(Label::new(
        Rect::new(2, 2, 46, 3),
        "Click on the labels to focus the input fields"
    )));

    // First Name field with linked label
    // Add the input first and capture its index
    let first_name_data = Rc::new(RefCell::new(String::new()));
    let first_name_idx = dialog.add(Box::new(InputLine::new(Rect::new(15, 4, 35, 5), 20, Rc::clone(&first_name_data))));

    // Create label and link it to the input using the returned index
    let mut first_name_label = Label::new(Rect::new(2, 4, 15, 5), "~F~irst Name:");
    first_name_label.set_link(first_name_idx);
    dialog.add(Box::new(first_name_label));

    // Last Name field with linked label
    let last_name_data = Rc::new(RefCell::new(String::new()));
    let last_name_idx = dialog.add(Box::new(InputLine::new(Rect::new(15, 6, 35, 7), 20, Rc::clone(&last_name_data))));

    let mut last_name_label = Label::new(Rect::new(2, 6, 15, 7), "~L~ast Name:");
    last_name_label.set_link(last_name_idx);
    dialog.add(Box::new(last_name_label));

    // Email field with linked label
    let email_data = Rc::new(RefCell::new(String::new()));
    let email_idx = dialog.add(Box::new(InputLine::new(Rect::new(15, 8, 35, 9), 20, Rc::clone(&email_data))));

    let mut email_label = Label::new(Rect::new(2, 8, 15, 9), "~E~mail:");
    email_label.set_link(email_idx);
    dialog.add(Box::new(email_label));

    // Buttons
    dialog.add(Box::new(Button::new(Rect::new(15, 10, 25, 12), "  OK  ", CM_OK, true)));
    dialog.add(Box::new(Button::new(Rect::new(27, 10, 37, 12), "Cancel", CM_CANCEL, false)));

    dialog.set_initial_focus();

    let result = dialog.execute(&mut app);

    println!("Dialog result: {}", result);
    println!("First Name: {}", first_name_data.borrow());
    println!("Last Name: {}", last_name_data.borrow());
    println!("Email: {}", email_data.borrow());

    Ok(())
}
