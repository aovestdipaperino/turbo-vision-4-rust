// (C) 2025 - Enzo Lombardi
// Non-interactive test for clipping
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::prelude::*;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::{
    button::Button, checkbox::CheckBox, dialog::Dialog, input_line::InputLine,
    static_text::StaticText,
};

fn main() -> turbo_vision::core::error::Result<()> {
    // Create terminal in a non-interactive way for testing
    let mut terminal = Terminal::init()?;

    // Create a dialog with various controls
    let mut dialog = Dialog::new(Rect::new(10, 5, 70, 20), "Clipping Test Dialog");

    // Add text that would overflow without clipping (relative coordinates)
    let text = StaticText::new(
        Rect::new(1, 1, 57, 2),
        "This dialog tests clipping. Text should not overflow past frame borders.",
    );
    dialog.add(Box::new(text));

    // Add an input field
    let input_data = Rc::new(RefCell::new("Sample Input Text".to_string()));
    let input = InputLine::new(Rect::new(1, 4, 39, 5), 50, input_data);
    dialog.add(Box::new(input));

    // Add checkboxes
    let checkbox1 = CheckBox::new(Rect::new(1, 6, 29, 7), "Enable feature A");
    dialog.add(Box::new(checkbox1));

    let checkbox2 = CheckBox::new(
        Rect::new(1, 7, 29, 8),
        "Enable feature B that has a really long name that should clip",
    );
    dialog.add(Box::new(checkbox2));

    // Add buttons
    let ok_button = Button::new(Rect::new(14, 10, 24, 12), "  OK  ", CM_OK, true);
    dialog.add(Box::new(ok_button));

    let cancel_button = Button::new(Rect::new(29, 10, 41, 12), " Cancel ", CM_CANCEL, false);
    dialog.add(Box::new(cancel_button));

    // Draw the dialog
    dialog.draw(&mut terminal);
    terminal.flush()?;

    // Dump to files
    terminal.dump_screen("test_clipping_screen.ans")?;
    dialog.dump_to_file(&terminal, "test_clipping_dialog.ans")?;

    println!("Clipping test complete. Check test_clipping_screen.ans");

    terminal.shutdown()?;
    Ok(())
}
