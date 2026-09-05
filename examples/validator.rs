// (C) 2025 - Enzo Lombardi
// Comprehensive Validator Demo
//
// Demonstrates all Validator types in one example:
// - FilterValidator (character filtering)
// - RangeValidator (numeric ranges)
// - PictureValidator (format masks)

use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_OK};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::input_line::InputLine;
use turbo_vision::views::{GroupLike, Handle};
use turbo_vision::views::{
    button::ButtonBuilder,
    dialog::DialogBuilder,
    input_line::InputLineBuilder,
    label::LabelBuilder,
    picture_validator::PictureValidator,
    static_text::StaticTextBuilder,
    validator::{FilterValidator, RangeValidator, Validator},
};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Show all validators in a single comprehensive dialog
    demo_all_validators(&mut app);

    Ok(())
}

/// The text of an input field, read back through its handle.
fn text_of(dialog: &Dialog, field: Handle<InputLine>) -> String {
    dialog
        .get(field)
        .map(|f| f.text().to_string())
        .unwrap_or_default()
}

fn demo_all_validators(app: &mut Application) {
    let (width, height) = app.terminal.size();

    // Create larger dialog to fit all validators
    let dialog_width = 65;
    let dialog_height = 34;
    let dialog_x = (width - dialog_width) / 2;
    let dialog_y = (height - dialog_height) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(
            dialog_x,
            dialog_y,
            dialog_x + dialog_width,
            dialog_y + dialog_height,
        ))
        .title("All Validator Types")
        .build();

    // Instructions
    let instructions = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, dialog_width - 4, 3))
        .text("Try typing in each field. Invalid characters are rejected.\nClick OK to validate final values.")
        .build();
    dialog.add(instructions);

    let mut y = 4;

    // Section 1: Filter & Range Validators
    let section1 = StaticTextBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("=== Filter & Range Validators ===")
        .build();
    dialog.add(section1);
    y += 2;

    // Field 1: Digits only (FilterValidator)
    let label1 = LabelBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("Digits only:")
        .build();
    dialog.add(label1);
    y += 1;

    let field1_validator = Rc::new(RefCell::new(FilterValidator::new("0123456789")));
    let input1 = InputLineBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .max_length(20)
        .text("12345")
        .validator(field1_validator.clone())
        .build();
    let field1 = dialog.add_typed(input1);
    y += 2;

    // Field 2: Range 0-100 (RangeValidator)
    let label2 = LabelBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("Number (0-100):")
        .build();
    dialog.add(label2);
    y += 1;

    let field2_validator = Rc::new(RefCell::new(RangeValidator::new(0, 100)));
    let input2 = InputLineBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .max_length(20)
        .text("50")
        .validator(field2_validator.clone())
        .build();
    let field2 = dialog.add_typed(input2);
    y += 2;

    // Field 3: Range -50 to 50 (negative numbers allowed)
    let label3 = LabelBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("Number (-50 to 50):")
        .build();
    dialog.add(label3);
    y += 1;

    let field3_validator = Rc::new(RefCell::new(RangeValidator::new(-50, 50)));
    let input3 = InputLineBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .max_length(20)
        .text("-25")
        .validator(field3_validator.clone())
        .build();
    let field3 = dialog.add_typed(input3);
    y += 2;

    // Field 4: Hex numbers 0x00-0xFF (RangeValidator with hex support)
    let label4 = LabelBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("Hex (0x00-0xFF):")
        .build();
    dialog.add(label4);
    y += 1;

    let field4_validator = Rc::new(RefCell::new(RangeValidator::new(0, 255)));
    let input4 = InputLineBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .max_length(20)
        .text("0xAB")
        .validator(field4_validator.clone())
        .build();
    let field4 = dialog.add_typed(input4);
    y += 3;

    // Section 2: Picture Mask Validators
    let section2 = StaticTextBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 1))
        .text("=== Picture Mask Validators ===")
        .build();
    dialog.add(section2);
    y += 2;

    // Phone number field with validator
    let phone_label = LabelBuilder::new()
        .bounds(Rect::new(2, y, 18, y + 1))
        .text("~P~hone Number:")
        .build();
    dialog.add(phone_label);

    let mut phone_input = InputLineBuilder::new()
        .bounds(Rect::new(18, y, 35, y + 1))
        .max_length(20)
        .build();
    phone_input.set_validator(Rc::new(RefCell::new(PictureValidator::new(
        "(###) ###-####",
    ))));
    let phone = dialog.add_typed(phone_input);

    let phone_hint = StaticTextBuilder::new()
        .bounds(Rect::new(36, y, 51, y + 1))
        .text("(###) ###-####")
        .build();
    dialog.add(phone_hint);
    y += 2;

    // Date field with validator
    let date_label = LabelBuilder::new()
        .bounds(Rect::new(2, y, 18, y + 1))
        .text("~D~ate:")
        .build();
    dialog.add(date_label);

    let mut date_input = InputLineBuilder::new()
        .bounds(Rect::new(18, y, 30, y + 1))
        .max_length(10)
        .build();
    date_input.set_validator(Rc::new(RefCell::new(PictureValidator::new("##/##/####"))));
    let date = dialog.add_typed(date_input);

    let date_hint = StaticTextBuilder::new()
        .bounds(Rect::new(31, y, 51, y + 1))
        .text("##/##/####")
        .build();
    dialog.add(date_hint);
    y += 2;

    // Product code field
    let code_label = LabelBuilder::new()
        .bounds(Rect::new(2, y, 18, y + 1))
        .text("Product ~C~ode:")
        .build();
    dialog.add(code_label);

    let mut code_input = InputLineBuilder::new()
        .bounds(Rect::new(18, y, 31, y + 1))
        .max_length(9)
        .build();
    code_input.set_validator(Rc::new(RefCell::new(PictureValidator::new("@@@@-####"))));
    let code = dialog.add_typed(code_input);

    let code_hint = StaticTextBuilder::new()
        .bounds(Rect::new(32, y, 51, y + 1))
        .text("@@@@-####")
        .build();
    dialog.add(code_hint);
    y += 2;

    // Legend
    let legend = StaticTextBuilder::new()
        .bounds(Rect::new(2, y, dialog_width - 4, y + 2))
        .text("Legend: # = digit, @ = letter, ! = any\nLiterals (like /, -, ()) are inserted automatically")
        .build();
    dialog.add(legend);
    y += 3;

    // Buttons
    let ok_button = ButtonBuilder::new()
        .bounds(Rect::new(20, y, 30, y + 2))
        .title("  OK  ")
        .command(CM_OK)
        .default(true)
        .build();
    dialog.add(ok_button);

    let cancel_button = ButtonBuilder::new()
        .bounds(Rect::new(35, y, 45, y + 2))
        .title("Cancel")
        .command(CM_CANCEL)
        .build();
    dialog.add(cancel_button);

    dialog.set_initial_focus();

    // Execute dialog
    let result = dialog.execute(app);

    if result == CM_OK {
        // Validate all fields
        let mut all_valid = true;

        println!("\n\nValidation Results:");
        println!("==================");

        // Filter/Range validators
        let field1_text = text_of(&dialog, field1);
        let field1_valid = field1_validator.borrow().is_valid(&field1_text);
        println!(
            "Field 1 (Digits only): \"{}\" - {}",
            field1_text,
            if field1_valid { "VALID" } else { "INVALID" }
        );
        all_valid &= field1_valid;

        let field2_text = text_of(&dialog, field2);
        let field2_valid = field2_validator.borrow().is_valid(&field2_text);
        println!(
            "Field 2 (0-100): \"{}\" - {}",
            field2_text,
            if field2_valid { "VALID" } else { "INVALID" }
        );
        all_valid &= field2_valid;

        let field3_text = text_of(&dialog, field3);
        let field3_valid = field3_validator.borrow().is_valid(&field3_text);
        println!(
            "Field 3 (-50 to 50): \"{}\" - {}",
            field3_text,
            if field3_valid { "VALID" } else { "INVALID" }
        );
        all_valid &= field3_valid;

        let field4_text = text_of(&dialog, field4);
        let field4_valid = field4_validator.borrow().is_valid(&field4_text);
        println!(
            "Field 4 (0x00-0xFF): \"{}\" - {}",
            field4_text,
            if field4_valid { "VALID" } else { "INVALID" }
        );
        all_valid &= field4_valid;

        // Picture mask validators
        println!("\nFormatted Data Entered:");
        println!("Phone: {}", text_of(&dialog, phone));
        println!("Date: {}", text_of(&dialog, date));
        println!("Code: {}", text_of(&dialog, code));

        println!(
            "\nOverall: {}",
            if all_valid {
                "ALL FIELDS VALID"
            } else {
                "SOME FIELDS INVALID"
            }
        );
    } else {
        println!("\nDialog cancelled");
    }
}
