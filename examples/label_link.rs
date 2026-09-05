// (C) 2025 - Enzo Lombardi
// Label Link Demo - demonstrates clicking labels to focus linked input fields

// Win11: links do NOT work in VSCode integrated Terminal. Use an external terminal instead

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_OK};
use turbo_vision::core::geometry::Rect;
use turbo_vision::helpers::msgbox::{MF_INFORMATION, MF_OK_BUTTON, message_box};
use turbo_vision::views::GroupLike;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::input_line::{InputLine, InputLineBuilder};
use turbo_vision::views::label::LabelBuilder;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create a dialog demonstrating label links
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(15, 5, 66, 19))
        .title("Label Link Demo")
        .build();

    // Instructions
    dialog.add(Box::new(
        LabelBuilder::new()
            .bounds(Rect::new(2, 2, 46, 2))
            .text("Click on the labels to focus the input fields")
            .build(),
    ));

    // First Name field with linked label
    let mut first_name_label = LabelBuilder::new()
        .bounds(Rect::new(2, 4, 15, 4))
        .text("~F~irst Name:")
        .build();
    let first_name = dialog.add_typed(
        InputLineBuilder::new()
            .bounds(Rect::new(15, 4, 35, 4))
            .max_length(20)
            .build(),
    );
    first_name_label.set_link(first_name.id());
    dialog.add(Box::new(first_name_label));

    // Last Name field with linked label
    let mut last_name_label = LabelBuilder::new()
        .bounds(Rect::new(2, 6, 15, 6))
        .text("~L~ast Name:")
        .build();
    let last_name = dialog.add_typed(
        InputLineBuilder::new()
            .bounds(Rect::new(15, 6, 35, 6))
            .max_length(20)
            .build(),
    );
    last_name_label.set_link(last_name.id());
    dialog.add(Box::new(last_name_label));

    // Email field with linked label
    let mut email_label = LabelBuilder::new()
        .bounds(Rect::new(2, 8, 15, 8))
        .text("~E~mail:")
        .build();
    let email = dialog.add_typed(
        InputLineBuilder::new()
            .bounds(Rect::new(15, 8, 35, 8))
            .max_length(20)
            .build(),
    );
    email_label.set_link(email.id());
    dialog.add(Box::new(email_label));

    // Buttons
    dialog.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(15, 10, 25, 12))
            .title("  OK  ")
            .command(CM_OK)
            .default(true)
            .build(),
    ));
    dialog.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(27, 10, 37, 12))
            .title("Cancel")
            .command(CM_CANCEL)
            .build(),
    ));

    dialog.set_initial_focus();

    let result = dialog.execute(&mut app);

    // Show information box with user's choices if they clicked OK
    let text_of = |h| {
        dialog
            .get(h)
            .map(|f: &InputLine| f.text().to_string())
            .unwrap_or_default()
    };
    let first = text_of(first_name);
    let last = text_of(last_name);
    let email = text_of(email);

    if result == CM_OK {
        let message = format!(
            "You entered:\n\nFirst Name: {}\nLast Name: {}\nEmail: {}",
            if first.is_empty() { "(none)" } else { &first },
            if last.is_empty() { "(none)" } else { &last },
            if email.is_empty() { "(none)" } else { &email }
        );

        message_box(&mut app, &message, MF_INFORMATION | MF_OK_BUTTON);
    }

    println!("Dialog result: {result}");
    println!("First Name: {first}");
    println!("Last Name: {last}");
    println!("Email: {email}");

    Ok(())
}
