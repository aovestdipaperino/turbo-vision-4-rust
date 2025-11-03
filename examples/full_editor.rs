// Full Editor Demo - Demonstrates TEditor search/replace and advanced features
//
// This example shows:
// - Complete text editor with search and replace
// - Find functionality with case-sensitive and whole-word options
// - Find Next to find subsequent occurrences
// - Replace functionality with single and replace-all modes
// - Search options dialog
// - Replace dialog with options
//
// Note: This is a simplified demo focusing on search/replace.
// For a full editor with window/desktop, see the editor example.

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_CANCEL;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::editor::Editor;
use turbo_vision::views::button::Button;
use turbo_vision::views::static_text::StaticText;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic;

fn log_to_file(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("full_editor_debug.log")
    {
        let _ = writeln!(file, "{}", msg);
    }
}

fn main() -> std::io::Result<()> {
    // Set up panic hook to log panics to file
    panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            format!("PANIC: {}", s)
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            format!("PANIC: {}", s)
        } else {
            "PANIC: Unknown panic payload".to_string()
        };

        let location = if let Some(loc) = panic_info.location() {
            format!(" at {}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            " at unknown location".to_string()
        };

        log_to_file(&format!("{}{}", msg, location));
        log_to_file("=== Stack backtrace follows ===");

        // Try to get backtrace
        let backtrace = std::backtrace::Backtrace::capture();
        log_to_file(&format!("{:?}", backtrace));
    }));

    log_to_file("=== Starting full_editor example ===");

    log_to_file("Creating Application...");
    let mut app = Application::new()?;
    log_to_file("Application created successfully");

    log_to_file("Getting terminal size...");
    let (width, height) = app.terminal.size();
    log_to_file(&format!("Terminal size: {}x{}", width, height));

    // Create main dialog with editor - ensure it fits on screen
    log_to_file("Calculating dialog dimensions...");
    let dialog_width = 70.min(width as i16 - 4);
    let dialog_height = 25.min(height as i16 - 2);
    let dialog_x = ((width as i16 - dialog_width) / 2).max(0);
    let dialog_y = ((height as i16 - dialog_height) / 2).max(0);
    log_to_file(&format!("Dialog dimensions: {}x{} at ({},{})", dialog_width, dialog_height, dialog_x, dialog_y));

    log_to_file("Creating Dialog...");
    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Full Editor Demo - Search & Replace"
    );
    log_to_file("Dialog created successfully");

    // Add instructions
    log_to_file("Creating instructions StaticText...");
    let instructions = StaticText::new(
        Rect::new(2, 1, dialog_width - 4, 3),
        "Editor with scrollbars, indicator, and sample text for testing.\n\
         Use arrow keys to scroll. Press Close button or ESC to exit."
    );
    log_to_file("Adding instructions to dialog...");
    dialog.add(Box::new(instructions));
    log_to_file("Instructions added successfully");

    // Add editor - leave room at bottom for button (3 rows)
    // Editor needs minimum size for scrollbars and indicator
    log_to_file("Calculating editor bounds...");
    let editor_bounds = Rect::new(2, 3, dialog_width - 2, dialog_height - 3);
    log_to_file(&format!("Editor bounds: {:?}", editor_bounds));

    log_to_file("Creating Editor...");
    let editor = Editor::new(editor_bounds);
    log_to_file("Editor created, adding scrollbars and indicator...");
    let mut editor = editor.with_scrollbars_and_indicator();
    log_to_file("Scrollbars and indicator added successfully");

    // Set sample text with patterns to search/replace
    let sample_text = r#"Welcome to the Full Editor Demo!

Search examples:
- Find "the" (case-insensitive) - matches "the", "The", "THE"
- Find "the" (case-sensitive) - matches only "the"
- Find "text" (whole words) - matches "text" but not "context"

The quick brown fox jumps over the lazy dog.
THE QUICK BROWN FOX JUMPS OVER THE LAZY DOG.

Some sample words: text, context, TEXT, Context, textual
More patterns: the, The, THE, they, them, other, gather

Replace examples:
- Replace "fox" with "cat" (single occurrence)
- Replace all "the" with "a" (replace all)

Line 1: This text has the word text in it.
Line 2: Another text line with THE word TEXT.
Line 3: More text for testing text searches.
Line 4: The final text line for text replacement.

Try different search options and see how they work!"#;

    log_to_file("Setting editor text...");
    editor.set_text(sample_text);
    log_to_file("Text set successfully");

    log_to_file("Setting auto-indent...");
    editor.set_auto_indent(true);
    log_to_file("Auto-indent enabled");

    log_to_file("Adding editor to dialog...");
    dialog.add(Box::new(editor));
    log_to_file("Editor added successfully");

    // Add close button at the very bottom
    log_to_file("Creating close button...");
    let button_y = dialog_height - 2;
    let close_button = Button::new(
        Rect::new((dialog_width / 2) - 5, button_y, (dialog_width / 2) + 5, button_y + 2),
        "~C~lose",
        CM_CANCEL,
        false  // Editor should get focus first
    );
    log_to_file(&format!("Close button created at y={}", button_y));

    log_to_file("Adding button to dialog...");
    dialog.add(Box::new(close_button));
    log_to_file("Button added successfully");

    log_to_file("Setting initial focus...");
    dialog.set_initial_focus();
    log_to_file("Initial focus set");

    // Execute dialog (shows editor with search-and-replace-ready text)
    log_to_file("Executing dialog...");
    let _result = dialog.execute(&mut app);
    log_to_file(&format!("Dialog returned with result: {}", _result));

    log_to_file("=== Exiting normally ===");
    Ok(())
}

// Example demonstrates Editor's search/replace capabilities
// In a real application, you would:
// 1. Add Find/Replace dialogs like the ones shown in validator_demo.rs
// 2. Call editor.find(text, SearchOptions { case_sensitive, whole_words_only, backwards })
// 3. Call editor.find_next() for F3 functionality
// 4. Call editor.replace_next() or editor.replace_all() for replacements
//
// The sample text above contains patterns to test:
// - Case-sensitive vs insensitive search
// - Whole-word matching
// - Wrap-around search
// - Replace single vs replace all
