// (C) 2025 - Enzo Lombardi
// Add a status line to the desktop

use turbo_vision::core::event::{KB_ALT_X, KB_ESC, KB_ESC_ESC};
use turbo_vision::prelude::*;
use turbo_vision::views::status_line::{StatusItem, StatusLine};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Add status line
    let (w, h) = app.terminal.size();
    let status_line = StatusLine::new(
        Rect::new(0, h as i16 - 1, w as i16, h as i16),
        vec![
            StatusItem::new("~Esc-X~ Exit", KB_ESC, CM_QUIT),
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Esc-Esc~ Exit", KB_ESC_ESC, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);
    app.run();
    Ok(())
}
