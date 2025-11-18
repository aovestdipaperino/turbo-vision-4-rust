// (C) 2025 - Enzo Lombardi
// Wrong Palette Demo - Button with incorrect owner_type in a Window

use turbo_vision::prelude::*;

use turbo_vision::core::event::{KB_ALT_X, KB_ESC};
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::WindowBuilder;

const CMD_TEST: u16 = 100;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Status line
    let (width, height) = app.terminal.size();
    let status_line = StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Esc~ Exit", KB_ESC, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Create a Window
    let mut window = WindowBuilder::new().bounds(Rect::new(20, 5, 60, 15)).title("Window with Button").build();

    // Add a button with WRONG owner_type (Window instead of Dialog )
    let button = ButtonBuilder::new().bounds(Rect::new(10, 4, 30, 6)).title("~T~est Button").command(CMD_TEST).build();
    window.add(Box::new(button));

    app.desktop.add(Box::new(window));
    app.run();

    Ok(())
}
