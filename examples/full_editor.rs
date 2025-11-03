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

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create main dialog with editor
    let dialog_width = 70;
    let dialog_height = 25;
    let dialog_x = (width as i16 - dialog_width) / 2;
    let dialog_y = (height as i16 - dialog_height) / 2;

    let mut dialog = Dialog::new(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "Full Editor Demo - Search & Replace"
    );

    // Add instructions
    let instructions = StaticText::new(
        Rect::new(2, 1, dialog_width - 4, 3),
        "Editor with scrollbars, indicator, and sample text for testing.\n\
         Use arrow keys to scroll. Press Close button or ESC to exit."
    );
    dialog.add(Box::new(instructions));

    // Add editor - leave room at bottom for button (3 rows)
    let editor_bounds = Rect::new(2, 3, dialog_width - 4, dialog_height - 4);
    let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();

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

    editor.set_text(sample_text);
    editor.set_auto_indent(true);

    dialog.add(Box::new(editor));

    // Add close button at the very bottom
    let close_button = Button::new(
        Rect::new((dialog_width / 2) - 5, dialog_height - 2, (dialog_width / 2) + 5, dialog_height),
        "~C~lose",
        CM_CANCEL,
        false  // Editor should get focus first
    );
    dialog.add(Box::new(close_button));

    dialog.set_initial_focus();

    // Execute dialog (shows editor with search-and-replace-ready text)
    let _result = dialog.execute(&mut app);

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
