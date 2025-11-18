// (C) 2025 - Enzo Lombardi
// Minimal Application Example
// Demonstrates a stripped-down application similar to deriving from TProgram
// instead of TApplication in Borland Turbo Vision.

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{KB_ALT_X, KB_ESC, KB_ESC_ESC};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::label::LabelBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::WindowBuilder;

fn main() -> turbo_vision::core::error::Result<()> {
    // Create a minimal application
    // In Borland TV, this would be: class MinimalApp : public TProgram
    let mut app = Application::new()?;

    // Add minimal status line (no menu bar!)
    let (width, height) = app.terminal.size();
    let status_line = StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItem::new("~Esc-X~ Exit", KB_ESC, CM_QUIT),
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Esc-Esc~ Exit", KB_ESC_ESC, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Create a simple information window
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(15, 5, 65, 15))
        .title("Minimal Application")
        .build();

    // Add some text and make sure user know how to quit
    let label1 = LabelBuilder::new()
        .bounds(Rect::new(2, 2, 46, 2))
        .text("Demonstrates a stripped-down application.")
        .build();
    let label2 = LabelBuilder::new()
        .bounds(Rect::new(2, 3, 46, 3))
        .text("No menu bar, just a status line.")
        .build();
    let label3 = LabelBuilder::new()
        .bounds(Rect::new(2, 5, 46, 5))
        .text("To exit: Alt-X, Esc-X, Esc-Esc, F10, Ctrl-C")
        .build();
    let label4 = LabelBuilder::new()
        .bounds(Rect::new(2, 6, 46, 6))
        .text("macOS  : Esc-X works if Alt fails")
        .build();

    window.add(Box::new(label1));
    window.add(Box::new(label2));
    window.add(Box::new(label3));
    window.add(Box::new(label4));

    app.desktop.add(Box::new(window));

    // Run the application
    app.run();

    Ok(())
}
