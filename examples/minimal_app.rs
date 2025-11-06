// (C) 2025 - Enzo Lombardi
// Minimal Application Example
// Demonstrates a stripped-down application similar to deriving from TProgram
// instead of TApplication in Borland Turbo Vision.

use turbo_vision::app::Application;
use turbo_vision::views::status_line::{StatusLine, StatusItem};
use turbo_vision::views::window::Window;
use turbo_vision::views::label::Label;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::event::{KB_ESC, KB_ALT_X};
use turbo_vision::core::command::CM_QUIT;

fn main() -> turbo_vision::core::error::Result<()> {
    // Create a minimal application
    // In Borland TV, this would be: class MinimalApp : public TProgram
    let mut app = Application::new()?;

    // Add minimal status line (no menu bar!)
    let (width, height) = app.terminal.size();
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~Esc~ Exit", KB_ESC, CM_QUIT),
            StatusItem::new("~Alt-X~ Quit", KB_ALT_X, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Create a simple information window
    let mut window = Window::new(
        Rect::new(15, 5, 65, 15),
        "Minimal Application",
    );

    // Add some text
    let label1 = Label::new(
        Rect::new(2, 2, 46, 3),
        "This is a minimal application!",
    );
    let label2 = Label::new(
        Rect::new(2, 4, 46, 5),
        "No menu bar - just status line.",
    );
    let label3 = Label::new(
        Rect::new(2, 6, 46, 7),
        "Similar to deriving from TProgram",
    );
    let label4 = Label::new(
        Rect::new(2, 7, 46, 8),
        "instead of TApplication.",
    );

    window.add(Box::new(label1));
    window.add(Box::new(label2));
    window.add(Box::new(label3));
    window.add(Box::new(label4));

    app.desktop.add(Box::new(window));

    // Run the application
    app.run();

    Ok(())
}
