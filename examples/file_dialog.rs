// Example demonstrating the file dialog
//
// This example shows how to use the FileDialog to let users select files.
//
// Usage:
//   cargo run --example file_dialog
//
// The dialog supports:
// - Mouse clicks to select files
// - Double-click to open files
// - Arrow keys to navigate
// - Enter to select files or navigate into folders
// - Directory navigation: Enter on folders opens them, dialog stays open
//
// Wildcard patterns:
// - "*" shows all files
// - "*.rs" shows only Rust files
// - "*.toml" shows only TOML files
// - etc.

use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::file_dialog::FileDialog;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create file dialog
    let dialog_width = 60;
    let dialog_height = 20;
    let dialog_x = (width as i16 - dialog_width) / 2;
    let dialog_y = (height as i16 - dialog_height) / 2;

    // Show all files with "*" wildcard
    // You can use "*.rs" to show only Rust files, "*.toml" for TOML files, etc.
    let mut file_dialog = FileDialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Open File",
        "*",  // Wildcard: "*" = all files, "*.ext" = specific extension
        None,    // Start in current directory
    ).build();

    // Execute (now redraws desktop background on each frame - fixes trailing bug!)
    match file_dialog.execute(&mut app) {
        Some(path) => {
            println!("\nSelected file: {}", path.display());
        }
        None => {
            println!("\nDialog canceled");
        }
    }

    Ok(())
}
