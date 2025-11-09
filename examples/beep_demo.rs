// (C) 2025 - Enzo Lombardi
// Terminal Beep Demo - demonstrates audio feedback

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_OK;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::static_text::StaticText;

const CM_BEEP: u16 = 100;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut dialog = Dialog::new(Rect::new(20, 8, 60, 16), "Beep Demo");

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 2, 36, 4),
        "Click the Beep button to hear\nthe terminal bell sound",
    )));

    dialog.add(Box::new(Button::new(
        Rect::new(8, 5, 18, 7),
        "Beep!",
        CM_BEEP,
        false,
    )));
    dialog.add(Box::new(Button::new(
        Rect::new(20, 5, 30, 7),
        "Close",
        CM_OK,
        true,
    )));

    loop {
        let result = dialog.execute(&mut app);

        if result == CM_BEEP {
            // Make a beep sound!
            app.beep();
            // Continue the dialog
            continue;
        }

        // Any other command closes the dialog
        break;
    }

    Ok(())
}
