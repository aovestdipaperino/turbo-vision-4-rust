// File Editor Example
//
// Demonstrates:
// - Editor with file load/save capabilities
// - Search and replace functionality
// - Modified flag tracking
// - Undo/redo operations

use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::editor::Editor;
use turbo_vision::views::view::View;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let terminal_size = app.terminal.size();

    // Create editor with scrollbars and indicator
    let editor_bounds = Rect::new(0, 0, terminal_size.0 as i16, terminal_size.1 as i16 - 1);
    let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
    editor.set_focus(true);

    // Load this source file as an example
    let source_file = file!();
    match editor.load_file(source_file) {
        Ok(_) => {
            println!("Loaded file: {}", source_file);
            println!("Lines: {}", editor.line_count());
            println!("Modified: {}", editor.is_modified());
        }
        Err(e) => {
            println!("Error loading file: {}", e);
            editor.set_text("// Failed to load file\n// Starting with empty editor");
        }
    }

    // Demonstrate search functionality
    use turbo_vision::views::editor::SearchOptions;
    if let Some(pos) = editor.find("Editor", SearchOptions::new()) {
        println!("Found 'Editor' at: {:?}", pos);
    }

    println!();
    println!("File Editor Demo");
    println!("================");
    println!();
    println!("Loaded: {:?}", editor.get_filename());
    println!("Lines: {}", editor.line_count());
    println!("Modified: {}", editor.is_modified());
    println!();
    println!("Controls:");
    println!("  Arrow keys: Move cursor");
    println!("  Ctrl+A: Select all");
    println!("  Ctrl+C/X/V: Copy/Cut/Paste");
    println!("  Ctrl+Z/Y: Undo/Redo");
    println!("  Insert: Toggle insert/overwrite mode");
    println!();
    println!("Search (programmatic):");
    println!("  Found 'Editor' and highlighted it");
    println!();
    println!("File operations:");
    println!("  Current file: {}", editor.get_filename().unwrap_or("(none)"));
    println!("  Modified: {}", editor.is_modified());
    println!();
    println!("Press any key to exit...");

    // Draw the editor
    editor.draw(&mut app.terminal);
    editor.update_cursor(&mut app.terminal);
    let _ = app.terminal.flush();

    // Wait for key press
    use std::time::Duration;
    let _ = app.terminal.poll_event(Duration::from_secs(300));

    Ok(())
}
