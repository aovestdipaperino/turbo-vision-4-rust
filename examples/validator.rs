// (C) 2025 - Enzo Lombardi
// Comprehensive Validator Demo
//
// Demonstrates all Validator types in one example:
// - FilterValidator (character filtering)
// - RangeValidator (numeric ranges)
// - PictureValidator (format masks)

use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::command::{CM_OK, CM_CANCEL};
use turbo_vision::views::{
    dialog::DialogBuilder,
    button::ButtonBuilder,
    static_text::StaticTextBuilder,
    label::LabelBuilder,
    input_line::InputLineBuilder,
    validator::{FilterValidator, RangeValidator, Validator},
    picture_validator::PictureValidator,
};
use std::rc::Rc;
use std::cell::RefCell;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Show all validators in a single comprehensive dialog
    demo_all_validators(&mut app);

    Ok(())
}

fn demo_all_validators(app: &mut Application) {
    let (width, height) = app.terminal.size();

    // Create larger dialog to fit all validators
    let dialog_width = 65;
    let dialog_height = 34;
    let dialog_x = (width as i16 - dialog_width) / 2;
    let dialog_y = (height as i16 - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height))
        .title("All Validator Types")
        .build();

    // Instructions
    let instructions = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, dialog_width - 4, 3))
        .text("Try typing in each field. Invalid characters are rejected.\nClick OK to validate final values.")
        .build();
    dialog.add(Box::new(instructions));

    let mut y = 4;

    // Section 1: Filter & Range Validators
    let section1 = StaticTextBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("=== Filter & Range Validators ===")
        .build();
    dialog.add(Box::new(section1));
    y += 2;

    // Field 1: Digits only (FilterValidator)
    let label1 = LabelBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("Digits only:")
        .build();
    dialog.add(Box::new(label1));
    y += 1;

    let field1_data = Rc::new(RefCell::new(String::from("12345")));
    let field1_validator = Rc::new(RefCell::new(FilterValidator::new("0123456789")));
    let input1 = InputLineBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .max_length(20)
        .data(field1_data.clone())
        .validator(field1_validator.clone())
        .build();
    dialog.add(Box::new(input1));
    y += 2;

    // Field 2: Range 0-100 (RangeValidator)
    let label2 = LabelBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("Number (0-100):")
        .build();
    dialog.add(Box::new(label2));
    y += 1;

    let field2_data = Rc::new(RefCell::new(String::from("50")));
    let field2_validator = Rc::new(RefCell::new(RangeValidator::new(0, 100)));
    let input2 = InputLineBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .max_length(20)
        .data(field2_data.clone())
        .validator(field2_validator.clone())
        .build();
    dialog.add(Box::new(input2));
    y += 2;

    // Field 3: Range -50 to 50 (negative numbers allowed)
    let label3 = LabelBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("Number (-50 to 50):")
        .build();
    dialog.add(Box::new(label3));
    y += 1;

    let field3_data = Rc::new(RefCell::new(String::from("-25")));
    let field3_validator = Rc::new(RefCell::new(RangeValidator::new(-50, 50)));
    let input3 = InputLineBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .max_length(20)
        .data(field3_data.clone())
        .validator(field3_validator.clone())
        .build();
    dialog.add(Box::new(input3));
    y += 2;

    // Field 4: Hex numbers 0x00-0xFF (RangeValidator with hex support)
    let label4 = LabelBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("Hex (0x00-0xFF):")
        .build();
    dialog.add(Box::new(label4));
    y += 1;

    let field4_data = Rc::new(RefCell::new(String::from("0xAB")));
    let field4_validator = Rc::new(RefCell::new(RangeValidator::new(0, 255)));
    let input4 = InputLineBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .max_length(20)
        .data(field4_data.clone())
        .validator(field4_validator.clone())
        .build();
    dialog.add(Box::new(input4));
    y += 3;

    // Section 2: Picture Mask Validators
    let section2 = StaticTextBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("=== Picture Mask Validators ===")
        .build();
    dialog.add(Box::new(section2));
    y += 2;

    // Phone number field with validator
    let phone_label = LabelBuilder::new()
        .bounds(Rect::new(2, y, 18, y + 1))
        .text("~P~hone Number:")
        .build();
    dialog.add(Box::new(phone_label));

    let phone_data = Rc::new(RefCell::new(String::new()));
    let mut phone_input = InputLineBuilder::new()
        .bounds(Rect::new(18, y, 35, y + 1))
        .max_length(20)
        .data(phone_data.clone())
        .build();
    phone_input.set_validator(
        Rc::new(RefCell::new(
            PictureValidator::new("(###) ###-####")
        ))
    );
    dialog.add(Box::new(phone_input));

    let phone_hint = StaticTextBuilder::new()
        .bounds(Rect::new(36, y, 51, y + 1))
        .text("(###) ###-####")
        .build();
    dialog.add(Box::new(phone_hint));
    y += 2;

    // Date field with validator
    let date_label = LabelBuilder::new()
        .bounds(Rect::new(2, y, 18, y + 1))
        .text("~D~ate:")
        .build();
    dialog.add(Box::new(date_label));

    let date_data = Rc::new(RefCell::new(String::new()));
    let mut date_input = InputLineBuilder::new()
        .bounds(Rect::new(18, y, 30, y + 1))
        .max_length(10)
        .data(date_data.clone())
        .build();
    date_input.set_validator(
        Rc::new(RefCell::new(
            PictureValidator::new("##/##/####")
        ))
    );
    dialog.add(Box::new(date_input));

    let date_hint = StaticTextBuilder::new()
        .bounds(Rect::new(31, y, 51, y + 1))
        .text("##/##/####")
        .build();
    dialog.add(Box::new(date_hint));
    y += 2;

    // Product code field
    let code_label = LabelBuilder::new()
        .bounds(Rect::new(2, y, 18, y + 1))
        .text("Product ~C~ode:")
        .build();
    dialog.add(Box::new(code_label));

    let code_data = Rc::new(RefCell::new(String::new()));
    let mut code_input = InputLineBuilder::new()
        .bounds(Rect::new(18, y, 31, y + 1))
        .max_length(9)
        .data(code_data.clone())
        .build();
    code_input.set_validator(
        Rc::new(RefCell::new(
            PictureValidator::new("@@@@-####")
        ))
    );
    dialog.add(Box::new(code_input));

    let code_hint = StaticTextBuilder::new()
        .bounds(Rect::new(32, y, 51, y + 1))
        .text("@@@@-####")
        .build();
    dialog.add(Box::new(code_hint));
    y += 2;

    // Legend
    let legend = StaticTextBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 2))
        .text("Legend: # = digit, @ = letter, ! = any\nLiterals (like /, -, ()) are inserted automatically")
        .build();
    dialog.add(Box::new(legend));
    y += 3;

    // Buttons
    let ok_button = ButtonBuilder::new()
        .bounds(Rect::new(20, y, 30, y + 2))
        .title("  OK  ")
        .command(CM_OK)
        .default(true)
        .build();
    dialog.add(Box::new(ok_button));

    let cancel_button = ButtonBuilder::new()
        .bounds(Rect::new(35, y, 45, y + 2))
        .title("Cancel")
        .command(CM_CANCEL)
        .build();
    dialog.add(Box::new(cancel_button));

    dialog.set_initial_focus();

    // Execute dialog
    let result = dialog.execute(app);

    if result == CM_OK {
        // Validate all fields
        let mut all_valid = true;

        println!("\n\nValidation Results:");
        println!("==================");

        // Filter/Range validators
        let field1_text = field1_data.borrow().clone();
        let field1_valid = field1_validator.borrow().is_valid(&field1_text);
        println!("Field 1 (Digits only): \"{}\" - {}", field1_text, if field1_valid { "VALID" } else { "INVALID" });
        all_valid &= field1_valid;

        let field2_text = field2_data.borrow().clone();
        let field2_valid = field2_validator.borrow().is_valid(&field2_text);
        println!("Field 2 (0-100): \"{}\" - {}", field2_text, if field2_valid { "VALID" } else { "INVALID" });
        all_valid &= field2_valid;

        let field3_text = field3_data.borrow().clone();
        let field3_valid = field3_validator.borrow().is_valid(&field3_text);
        println!("Field 3 (-50 to 50): \"{}\" - {}", field3_text, if field3_valid { "VALID" } else { "INVALID" });
        all_valid &= field3_valid;

        let field4_text = field4_data.borrow().clone();
        let field4_valid = field4_validator.borrow().is_valid(&field4_text);
        println!("Field 4 (0x00-0xFF): \"{}\" - {}", field4_text, if field4_valid { "VALID" } else { "INVALID" });
        all_valid &= field4_valid;

        // Picture mask validators
        println!("\nFormatted Data Entered:");
        println!("Phone: {}", phone_data.borrow());
        println!("Date: {}", date_data.borrow());
        println!("Code: {}", code_data.borrow());

        println!("\nOverall: {}", if all_valid { "ALL FIELDS VALID" } else { "SOME FIELDS INVALID" });
    } else {
        println!("\nDialog cancelled");
    }
}
