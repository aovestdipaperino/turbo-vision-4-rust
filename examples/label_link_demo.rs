// (C) 2025 - Enzo Lombardi
// Label Link Demo - shows how to create clickable labels to set focus on the associated input field

use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_OK};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::input_line::InputLineBuilder;
use turbo_vision::views::label::LabelBuilder;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut dialog = DialogBuilder::new().bounds(Rect::new(15, 5, 65, 18)).title("Label Link Demo").build();

    dialog.add(Box::new(
        LabelBuilder::new().bounds(Rect::new(2, 1, 46, 1)).text("Click on the labels to focus the input fields").build(),
    ));

    //
    //
    // First Name
    // Add an input field and capture its index
    let first_name_data = Rc::new(RefCell::new(String::new()));
    let first_name_idx = dialog.add(Box::new(
        InputLineBuilder::new().bounds(Rect::new(15, 3, 37, 3)).max_length(22).data(Rc::clone(&first_name_data)).build(),
    ));

    // Create a label and link the lable to the input field using the index
    let mut first_name_label = LabelBuilder::new().bounds(Rect::new(2, 3, 15, 4)).text("~F~irst Name:").build();
    first_name_label.set_link(first_name_idx);
    dialog.add(Box::new(first_name_label));

    //
    //
    // Last Name
    let last_name_data = Rc::new(RefCell::new(String::new()));
    let last_name_idx = dialog.add(Box::new(
        InputLineBuilder::new().bounds(Rect::new(15, 5, 37, 5)).max_length(22).data(Rc::clone(&last_name_data)).build(),
    ));

    let mut last_name_label = LabelBuilder::new().bounds(Rect::new(2, 5, 15, 6)).text("~L~ast Name:").build();
    last_name_label.set_link(last_name_idx);
    dialog.add(Box::new(last_name_label));

    //
    //
    // Email
    let email_data = Rc::new(RefCell::new(String::new()));
    let email_idx = dialog.add(Box::new(InputLineBuilder::new().bounds(Rect::new(15, 7, 37, 7)).max_length(22).data(Rc::clone(&email_data)).build()));

    let mut email_label = LabelBuilder::new().bounds(Rect::new(2, 7, 15, 8)).text("~E~mail:").build();
    email_label.set_link(email_idx);
    dialog.add(Box::new(email_label));

    //
    //
    // Buttons
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(15, 9, 25, 11)).title("  OK  ").command(CM_OK).default(true).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(27, 9, 37, 11)).title("Cancel").command(CM_CANCEL).build()));

    dialog.set_initial_focus();

    let result = dialog.execute(&mut app);

    eprintln!("Dialog result: {result}");
    eprintln!("First Name: {}", first_name_data.borrow());
    eprintln!("Last Name: {}", last_name_data.borrow());
    eprintln!("Email: {}", email_data.borrow());

    Ok(())
}
