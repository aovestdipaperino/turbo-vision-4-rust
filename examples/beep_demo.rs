// (C) 2025 - Enzo Lombardi
// Terminal Beep Demo - demonstrates audio feedback

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_OK;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;

// Custom command IDs for this example
const CM_BEEP: u16 = 100;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut dialog = DialogBuilder::new().bounds(Rect::new(20, 8, 60, 16)).title("Beep Demo").build();

    dialog.add(Box::new(
        StaticTextBuilder::new()
            .bounds(Rect::new(2, 2, 36, 4))
            .text("Click the Beep button to hear\nthe terminal bell sound")
            .build(),
    ));

    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(8, 5, 18, 7)).title("Beep!").command(CM_BEEP).default(false).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(20, 5, 30, 7)).title("Close").command(CM_OK).default(true).build()));

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
