// Example demonstrating child view access using child_at_mut()
//
// This example shows how the architectural improvement allows accessing
// child views after they've been added to a dialog.

use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::label::Label;
use turbo_vision::views::button::Button;
use turbo_vision::core::command::CM_OK;
use std::rc::Rc;
use std::cell::RefCell;

fn main() -> std::io::Result<()> {
    let app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create a dialog
    let dialog_width = 50;
    let dialog_height = 10;
    let dialog_x = (width as i16 - dialog_width) / 2;
    let dialog_y = (height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Child Access Demo"
    );

    // Add a label
    let label = Label::new(Rect::new(2, 1, 20, 2), "Enter text:");
    dialog.add(Box::new(label));

    // Add an input line with shared data
    let input_data = Rc::new(RefCell::new(String::from("Hello")));
    let input = InputLine::new(Rect::new(20, 1, dialog_width - 4, 2), 50, input_data.clone());
    dialog.add(Box::new(input));

    // Add a button
    let button = Button::new(Rect::new(20, 4, 30, 6), "  OK  ", CM_OK, true);
    dialog.add(Box::new(button));

    // NOW we can access the children after adding them!
    println!("Dialog has {} children", dialog.child_count());

    // We can get references to specific children
    let first_child = dialog.child_at(0);
    println!("First child bounds: {:?}", first_child.bounds());

    // Or get mutable references to modify them
    // (In this example we'll just print info, but you could call methods)
    let second_child_bounds = dialog.child_at_mut(1).bounds();
    println!("Second child bounds: {:?}", second_child_bounds);

    println!("\nThis demonstrates that the architectural limitation has been addressed!");
    println!("You can now access child views after adding them to containers.");

    Ok(())
}
