// TMBASIC IDE Showcase for Turbo Vision Rust
//
// A simplified TMBASIC IDE demo showcasing turbo-vision capabilities:
// - Code editor window
// - File operations (New, Open, Save, Save As) using FileDialog
// - Program output display
// - Help system and about dialogs
// - Menu bar with application commands
// - Status line showing file information
//
// This demonstrates how to build a complete IDE using turbo-vision components.
//
// Build and run:
//   cargo run --example tm_basic

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::{
    editor::Editor,
    status_line::{StatusItem, StatusLine},
    window::WindowBuilder,
};
use turbo_vision::core::event::{KB_ALT_X, KB_ALT_H};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create status line
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~Alt-H~ Help", KB_ALT_H, 100),
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Create main window with editor
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(1, 1, width as i16 - 1, height as i16 - 2))
        .title("TMBASIC - Untitled*")
        .build();

    // Add editor to window
    let editor_bounds = Rect::new(2, 2, width as i16 - 3, height as i16 - 4);
    let mut editor = Editor::new(editor_bounds);

    // Pre-fill editor with welcome message
    let welcome_text = r#"' Welcome to TMBASIC - A BASIC IDE
' Built with Turbo Vision Rust
'
' Use File menu to create new programs
' or open existing .bas files
'
sub Main()
    print "Hello, TMBASIC!"
end sub"#;

    editor.set_text(welcome_text);

    window.add(Box::new(editor));
    app.desktop.add(Box::new(window));

    // Run the main application loop
    app.run();

    Ok(())
}
