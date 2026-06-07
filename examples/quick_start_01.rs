// (C) 2025 - Enzo Lombardi
// Add a status line to the desktop

use turbo_vision::core::event::KB_ALT_X;
use turbo_vision::prelude::*;
use turbo_vision::views::status_line::{StatusItem, StatusLine};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Add status line
    let (w, h) = app.terminal.size();
    let status_line = StatusLine::new(
        Rect::new(0, h as i16 - 1, w as i16, h as i16),
        vec![StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT)],
    );
    app.set_status_line(status_line);
    app.run();
    Ok(())
}
