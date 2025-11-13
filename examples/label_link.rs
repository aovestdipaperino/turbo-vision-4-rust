// (C) 2025 - Enzo Lombardi
// Label Link Demo - demonstrates clicking labels to focus linked input fields

// Win11: links do NOT work in VSCode integrated Terminal. Use an external terminal instead

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

    // Create a dialog demonstrating label links
    let mut dialog = DialogBuilder::new().bounds(Rect::new(15, 5, 66, 19)).title("Label Link Demo").build();

    // Instructions
    dialog.add(Box::new(
        LabelBuilder::new().bounds(Rect::new(2, 2, 46, 2)).text("Click on the labels to focus the input fields").build(),
    ));

    // First Name field with linked label
    let first_name_data = Rc::new(RefCell::new(String::new()));
    let mut first_name_label = LabelBuilder::new().bounds(Rect::new(2, 4, 15, 5)).text("~F~irst Name:").build(); // bounds must be on 2 lines
    first_name_label.set_link(dialog.add(Box::new(
        InputLineBuilder::new().bounds(Rect::new(15, 4, 35, 4)).max_length(20).data(Rc::clone(&first_name_data)).build(),
    )));
    dialog.add(Box::new(first_name_label));

    // Last Name field with linked label
    let last_name_data = Rc::new(RefCell::new(String::new()));
    let mut last_name_label = LabelBuilder::new().bounds(Rect::new(2, 6, 15, 7)).text("~L~ast Name:").build();
    last_name_label.set_link(dialog.add(Box::new(
        InputLineBuilder::new().bounds(Rect::new(15, 6, 35, 6)).max_length(20).data(Rc::clone(&last_name_data)).build(),
    )));
    dialog.add(Box::new(last_name_label));

    // Email field with linked label
    let email_data = Rc::new(RefCell::new(String::new()));
    let mut email_label = LabelBuilder::new().bounds(Rect::new(2, 8, 15, 9)).text("~E~mail:").build();
    email_label.set_link(dialog.add(Box::new(InputLineBuilder::new().bounds(Rect::new(15, 8, 35, 8)).max_length(20).data(Rc::clone(&email_data)).build())));
    dialog.add(Box::new(email_label));

    // Buttons
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(15, 10, 25, 12)).title("  OK  ").command(CM_OK).default(true).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(27, 10, 37, 12)).title("Cancel").command(CM_CANCEL).build()));

    dialog.set_initial_focus();

    let result = dialog.execute(&mut app);

    println!("Dialog result: {result}");
    println!("First Name: {}", first_name_data.borrow());
    println!("Last Name: {}", last_name_data.borrow());
    println!("Email: {}", email_data.borrow());

    Ok(())
}
