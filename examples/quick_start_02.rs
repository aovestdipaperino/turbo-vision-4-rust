// (C) 2025 - Enzo Lombardi
// The status line code is in a function

use turbo_vision::core::event::KB_ALT_X;
use turbo_vision::prelude::*;
use turbo_vision::views::status_line::{StatusItem, StatusLine};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Add status line
    let status_line = setup_status_line(&app);
    app.set_status_line(status_line);
    app.run();
    Ok(())
}

/// Create and configure the status line at the bottom of the screen
fn setup_status_line(app: &Application) -> StatusLine {
    let (w, h) = app.terminal.size();

    StatusLine::new(
        Rect::new(0, h as i16 - 1, w as i16, h as i16),
        vec![StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT)],
    )
}
