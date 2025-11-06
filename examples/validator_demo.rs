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
    dialog::Dialog,
    button::Button,
    static_text::StaticText,
    label::Label,
    input_line::InputLine,
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

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "All Validator Types"
    );

    // Instructions
    let instructions = StaticText::new(
        Rect::new(2, 1, dialog_width - 4, 3),
        "Try typing in each field. Invalid characters are rejected.\nClick OK to validate final values.",
    );
    dialog.add(Box::new(instructions));

    let mut y = 4;

    // Section 1: Filter & Range Validators
    let section1 = StaticText::new(
        Rect::new(2, y, dialog_width - 4, y + 1),
        "=== Filter & Range Validators ==="
    );
    dialog.add(Box::new(section1));
    y += 2;

    // Field 1: Digits only (FilterValidator)
    let label1 = Label::new(Rect::new(2, y, dialog_width - 4, y + 1), "Digits only:");
    dialog.add(Box::new(label1));
    y += 1;

    let field1_data = Rc::new(RefCell::new(String::from("12345")));
    let field1_validator = Rc::new(RefCell::new(FilterValidator::new("0123456789")));
    let input1 = InputLine::with_validator(
        Rect::new(2, y, dialog_width - 4, y + 1),
        20,
        field1_data.clone(),
        field1_validator.clone()
    );
    dialog.add(Box::new(input1));
    y += 2;

    // Field 2: Range 0-100 (RangeValidator)
    let label2 = Label::new(Rect::new(2, y, dialog_width - 4, y + 1), "Number (0-100):");
    dialog.add(Box::new(label2));
    y += 1;

    let field2_data = Rc::new(RefCell::new(String::from("50")));
    let field2_validator = Rc::new(RefCell::new(RangeValidator::new(0, 100)));
    let input2 = InputLine::with_validator(
        Rect::new(2, y, dialog_width - 4, y + 1),
        20,
        field2_data.clone(),
        field2_validator.clone()
    );
    dialog.add(Box::new(input2));
    y += 2;

    // Field 3: Range -50 to 50 (negative numbers allowed)
    let label3 = Label::new(Rect::new(2, y, dialog_width - 4, y + 1), "Number (-50 to 50):");
    dialog.add(Box::new(label3));
    y += 1;

    let field3_data = Rc::new(RefCell::new(String::from("-25")));
    let field3_validator = Rc::new(RefCell::new(RangeValidator::new(-50, 50)));
    let input3 = InputLine::with_validator(
        Rect::new(2, y, dialog_width - 4, y + 1),
        20,
        field3_data.clone(),
        field3_validator.clone()
    );
    dialog.add(Box::new(input3));
    y += 2;

    // Field 4: Hex numbers 0x00-0xFF (RangeValidator with hex support)
    let label4 = Label::new(Rect::new(2, y, dialog_width - 4, y + 1), "Hex (0x00-0xFF):");
    dialog.add(Box::new(label4));
    y += 1;

    let field4_data = Rc::new(RefCell::new(String::from("0xAB")));
    let field4_validator = Rc::new(RefCell::new(RangeValidator::new(0, 255)));
    let input4 = InputLine::with_validator(
        Rect::new(2, y, dialog_width - 4, y + 1),
        20,
        field4_data.clone(),
        field4_validator.clone()
    );
    dialog.add(Box::new(input4));
    y += 3;

    // Section 2: Picture Mask Validators
    let section2 = StaticText::new(
        Rect::new(2, y, dialog_width - 4, y + 1),
        "=== Picture Mask Validators ==="
    );
    dialog.add(Box::new(section2));
    y += 2;

    // Phone number field with validator
    let phone_label = Label::new(
        Rect::new(2, y, 18, y + 1),
        "~P~hone Number:"
    );
    dialog.add(Box::new(phone_label));

    let phone_data = Rc::new(RefCell::new(String::new()));
    let mut phone_input = InputLine::new(Rect::new(18, y, 35, y + 1), 20, phone_data.clone());
    phone_input.set_validator(
        Rc::new(RefCell::new(
            PictureValidator::new("(###) ###-####")
        ))
    );
    dialog.add(Box::new(phone_input));

    let phone_hint = StaticText::new(
        Rect::new(36, y, 51, y + 1),
        "(###) ###-####"
    );
    dialog.add(Box::new(phone_hint));
    y += 2;

    // Date field with validator
    let date_label = Label::new(
        Rect::new(2, y, 18, y + 1),
        "~D~ate:"
    );
    dialog.add(Box::new(date_label));

    let date_data = Rc::new(RefCell::new(String::new()));
    let mut date_input = InputLine::new(Rect::new(18, y, 30, y + 1), 10, date_data.clone());
    date_input.set_validator(
        Rc::new(RefCell::new(
            PictureValidator::new("##/##/####")
        ))
    );
    dialog.add(Box::new(date_input));

    let date_hint = StaticText::new(
        Rect::new(31, y, 51, y + 1),
        "##/##/####"
    );
    dialog.add(Box::new(date_hint));
    y += 2;

    // Product code field
    let code_label = Label::new(
        Rect::new(2, y, 18, y + 1),
        "Product ~C~ode:"
    );
    dialog.add(Box::new(code_label));

    let code_data = Rc::new(RefCell::new(String::new()));
    let mut code_input = InputLine::new(Rect::new(18, y, 31, y + 1), 9, code_data.clone());
    code_input.set_validator(
        Rc::new(RefCell::new(
            PictureValidator::new("@@@@-####")
        ))
    );
    dialog.add(Box::new(code_input));

    let code_hint = StaticText::new(
        Rect::new(32, y, 51, y + 1),
        "@@@@-####"
    );
    dialog.add(Box::new(code_hint));
    y += 2;

    // Legend
    let legend = StaticText::new(
        Rect::new(2, y, dialog_width - 4, y + 2),
        "Legend: # = digit, @ = letter, ! = any\nLiterals (like /, -, ()) are inserted automatically"
    );
    dialog.add(Box::new(legend));
    y += 3;

    // Buttons
    let ok_button = Button::new(
        Rect::new(20, y, 30, y + 2),
        "  OK  ",
        CM_OK,
        true
    );
    dialog.add(Box::new(ok_button));

    let cancel_button = Button::new(
        Rect::new(35, y, 45, y + 2),
        "Cancel",
        CM_CANCEL,
        false
    );
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
