// (C) 2025 - Enzo Lombardi

//! Message box and input box helpers - standard dialogs with pre-configured buttons.
//! Matches Borland: msgbox.h functions (messageBox, inputBox)

use crate::app::Application;
use crate::core::command::{CM_CANCEL, CM_NO, CM_OK, CM_YES, CommandId};
use crate::core::geometry::Rect;
use crate::views::View;
use crate::views::button::Button;
use crate::views::dialog::Dialog;
use crate::views::input_line::InputLine;
use crate::views::static_text::StaticText;
use std::cell::RefCell;
use std::rc::Rc;

// Message box type flags (matches Borland: mfWarning, mfError, etc.)
pub const MF_WARNING: u16 = 0x0000;
pub const MF_ERROR: u16 = 0x0001;
pub const MF_INFORMATION: u16 = 0x0002;
pub const MF_CONFIRMATION: u16 = 0x0003;

// Message box button flags (matches Borland: mfYesButton, mfNoButton, etc.)
pub const MF_YES_BUTTON: u16 = 0x0100;
pub const MF_NO_BUTTON: u16 = 0x0200;
pub const MF_OK_BUTTON: u16 = 0x0400;
pub const MF_CANCEL_BUTTON: u16 = 0x0800;

// Standard button combinations (matches Borland: mfYesNoCancel, mfOKCancel)
pub const MF_YES_NO_CANCEL: u16 = MF_YES_BUTTON | MF_NO_BUTTON | MF_CANCEL_BUTTON;
pub const MF_OK_CANCEL: u16 = MF_OK_BUTTON | MF_CANCEL_BUTTON;

/// Display a message box with the given message and options
/// Matches Borland: messageBox(const char *msg, ushort aOptions)
///
/// Options is a combination of message box type (lower 4 bits) and button flags:
/// - Type: MF_WARNING, MF_ERROR, MF_INFORMATION, MF_CONFIRMATION
/// - Buttons: MF_YES_BUTTON, MF_NO_BUTTON, MF_OK_BUTTON, MF_CANCEL_BUTTON
///
/// Returns the command ID of the button pressed (CM_YES, CM_NO, CM_OK, CM_CANCEL)
///
/// # Examples
///
/// ```ignore
/// let result = message_box(&mut app, "Save changes?", MF_CONFIRMATION | MF_YES_NO_CANCEL);
/// if result == CM_YES {
///     // Save
/// }
/// ```
pub fn message_box(app: &mut Application, msg: &str, options: u16) -> CommandId {
    let (width, height) = app.terminal.size();

    // Create centered dialog (40x9 as in Borland)
    let dialog_width = 40i16;
    let dialog_height = 9i16;
    let dialog_x = (width as i16 - dialog_width) / 2;
    let dialog_y = (height as i16 - dialog_height - 2) / 2; // -2 for menu and status

    let bounds = Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height);

    message_box_rect(app, bounds, msg, options)
}

/// Display a message box in the given rectangle
/// Matches Borland: messageBoxRect(const TRect &r, const char *msg, ushort aOptions)
pub fn message_box_rect(app: &mut Application, bounds: Rect, msg: &str, options: u16) -> CommandId {
    // Get title based on type (lower 4 bits)
    let title = match options & 0x03 {
        MF_WARNING => "Warning",
        MF_ERROR => "Error",
        MF_INFORMATION => "Information",
        MF_CONFIRMATION => "Confirm",
        _ => "Message",
    };

    let mut dialog = Dialog::new(bounds, title);

    // Add static text for message (inset by 3 from left, 2 from top/bottom/right)
    let text_bounds = Rect::new(3, 2, bounds.width() - 2, bounds.height() - 3);
    dialog.add(Box::new(StaticText::new(text_bounds, msg)));

    // Collect buttons to add
    let button_specs = [
        (MF_YES_BUTTON, "~Y~es", CM_YES),
        (MF_NO_BUTTON, "~N~o", CM_NO),
        (MF_OK_BUTTON, "O~K~", CM_OK),
        (MF_CANCEL_BUTTON, "Cancel", CM_CANCEL),
    ];

    let mut buttons = Vec::new();
    let mut total_width = -2i16; // Start at -2 to account for first button spacing

    for (flag, label, command) in button_specs.iter() {
        if (options & flag) != 0 {
            // Button is 10 wide, 2 tall (matches Borland)
            let button = Button::new(Rect::new(0, 0, 10, 2), label, *command, buttons.is_empty());
            total_width += 10 + 2; // Button width + spacing
            buttons.push((button, *command));
        }
    }

    // Center buttons horizontally
    let mut x = (bounds.width() - total_width) / 2;
    let y = bounds.height() - 4; // -3 initially

    for (mut button, _cmd) in buttons {
        // Position button
        let button_bounds = Rect::new(x, y, x + 10, y + 2);
        button.set_bounds(button_bounds);
        dialog.add(Box::new(button));
        x += 12; // Button width (10) + spacing (2)
    }

    dialog.set_initial_focus();
    dialog.execute(app)
}

/// Display an input box for text entry
/// Matches Borland: inputBox(const char *Title, const char *aLabel, char *s, uchar limit)
///
/// Returns CM_OK if OK was pressed, CM_CANCEL otherwise
/// The input string is returned as the second element of the tuple
///
/// # Examples
///
/// ```ignore
/// let (result, text) = input_box(&mut app, "Enter Name", "Name:", "", 50);
/// if result == CM_OK {
///     println!("Name entered: {}", text);
/// }
/// ```
pub fn input_box(app: &mut Application, title: &str, label: &str, default: &str, limit: usize) -> (CommandId, String) {
    let (width, height) = app.terminal.size();

    // Create centered dialog (60x8 as in Borland)
    let dialog_width = 60i16;
    let dialog_height = 8i16;
    let dialog_x = (width as i16 - dialog_width) / 2;
    let dialog_y = (height as i16 - dialog_height - 2) / 2;

    let bounds = Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height);

    input_box_rect(app, bounds, title, label, default, limit)
}

/// Display an input box in the given rectangle
/// Matches Borland: inputBoxRect(const TRect &bounds, const char *title, const char *aLabel, char *s, uchar limit)
pub fn input_box_rect(app: &mut Application, bounds: Rect, title: &str, label: &str, default: &str, limit: usize) -> (CommandId, String) {
    let mut dialog = Dialog::new(bounds, title);

    // Create shared data for the input line
    let input_data = Rc::new(RefCell::new(default.to_string()));

    // Add label (if provided)
    if !label.is_empty() {
        let label_bounds = Rect::new(2, 2, 2 + label.len() as i16 + 1, 3);
        dialog.add(Box::new(StaticText::new(label_bounds, label)));
    }

    // Add input line (positioned after label)
    let input_x = if !label.is_empty() { 4 + label.len() as i16 } else { 3 };
    let input_bounds = Rect::new(input_x, 2, bounds.width() - 3, 3);
    let input = InputLine::new(input_bounds, limit, Rc::clone(&input_data));
    dialog.add(Box::new(input));

    // Add OK button
    let ok_button = Button::new(
        Rect::new(bounds.width() / 2 - 12, bounds.height() - 4, bounds.width() / 2 - 2, bounds.height() - 2),
        "O~K~",
        CM_OK,
        true, // default button
    );
    dialog.add(Box::new(ok_button));

    // Add Cancel button
    let cancel_button = Button::new(
        Rect::new(bounds.width() / 2 + 2, bounds.height() - 4, bounds.width() / 2 + 12, bounds.height() - 2),
        "Cancel",
        CM_CANCEL,
        false,
    );
    dialog.add(Box::new(cancel_button));

    dialog.set_initial_focus();
    let result = dialog.execute(app);

    // Get the input text from the shared data
    let text = input_data.borrow().clone();

    (result, text)
}
