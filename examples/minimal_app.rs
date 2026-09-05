// (C) 2025 - Enzo Lombardi
// Minimal Application Example
// Demonstrates a stripped-down application similar to deriving from TProgram
// instead of TApplication in Borland Turbo Vision.

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::KB_ESC_ESC;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::GroupLike;
use turbo_vision::views::label::LabelBuilder;
use turbo_vision::views::status_line::StatusLine;
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
            StatusItemBuilder::new()
                .text("~Esc-X~ Exit")
                .key("Esc")
                .command(CM_QUIT)
                .build(),
            StatusItemBuilder::new()
                .text("~Alt-X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
            StatusItemBuilder::new()
                .text("~Esc-Esc~ Exit")
                .key_code(KB_ESC_ESC)
                .command(CM_QUIT)
                .build(),
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

    window.add(label1);
    window.add(label2);
    window.add(label3);
    window.add(label4);

    app.desktop.add(window);

    // Run the application
    app.run();

    Ok(())
}
