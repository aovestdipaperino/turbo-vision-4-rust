// (C) 2025 - Enzo Lombardi
// Comprehensive Editor Demo
//
// Demonstrates all Editor features in one example:
// - Basic text editing with undo/redo
// - File I/O operations (load/save)
// - Search and replace
// - Syntax highlighting (Rust)
// - Selection and clipboard operations

use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::state::SF_MODAL;
use turbo_vision::views::{
    dialog::DialogBuilder,
    button::ButtonBuilder,
    static_text::StaticTextBuilder,
    window::WindowBuilder,
    editor::Editor,
    syntax::RustHighlighter,
    view::View,
};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Show menu to choose demo type
    loop {
        let choice = show_menu(&mut app);

        match choice {
            1 => demo_basic_editing(&mut app),
            2 => demo_search_replace(&mut app),
            3 => demo_syntax_highlighting(&mut app),
            4 => demo_file_operations(&mut app),
            _ => break,
        }
    }

    Ok(())
}

fn show_menu(app: &mut Application) -> u16 {
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(15, 6, 65, 18))
        .title("Editor Demonstrations")
        .build();

    let text = StaticTextBuilder::new()
        .bounds(Rect::new(2, 2, 48, 3))
        .text("Choose an editor feature to demonstrate:")
        .build();
    dialog.add(Box::new(text));

    let btn1 = ButtonBuilder::new()
        .bounds(Rect::new(5, 4, 45, 6))
        .title("1. ~B~asic Editing (undo/redo/clipboard)")
        .command(1)
        .default(true)
        .build();
    dialog.add(Box::new(btn1));

    let btn2 = ButtonBuilder::new()
        .bounds(Rect::new(5, 6, 45, 8))
        .title("2. ~S~earch and Replace")
        .command(2)
        .default(false)
        .build();
    dialog.add(Box::new(btn2));

    let btn3 = ButtonBuilder::new()
        .bounds(Rect::new(5, 8, 45, 10))
        .title("3. S~y~ntax Highlighting (Rust)")
        .command(3)
        .default(false)
        .build();
    dialog.add(Box::new(btn3));

    let btn4 = ButtonBuilder::new()
        .bounds(Rect::new(5, 10, 45, 12))
        .title("4. ~F~ile Operations (load/save)")
        .command(4)
        .default(false)
        .build();
    dialog.add(Box::new(btn4));

    dialog.execute(app)
}

fn demo_basic_editing(app: &mut Application) {
    let sample_text = "=== Basic Text Editing Demo ===\n\
\n\
This editor supports:\n\
\n\
EDITING:\n\
- Type to insert text\n\
- Arrow keys to navigate\n\
- Home/End for line start/end\n\
- PgUp/PgDn for page navigation\n\
\n\
UNDO/REDO:\n\
- Ctrl+Z to undo\n\
- Ctrl+Y to redo\n\
- Comprehensive action history\n\
\n\
CLIPBOARD:\n\
- Ctrl+C to copy (with OS clipboard)\n\
- Ctrl+X to cut\n\
- Ctrl+V to paste\n\
- Shift+Arrow keys to select\n\
- Ctrl+A to select all\n\
\n\
Try editing this text!\n\
Make some changes, then use Ctrl+Z to undo.\n\
\n\
Press ESC to exit.";

    // Create modal window with editor
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(5, 3, 75, 22))
        .title("Basic Editing Demo")
        .build();

    let editor_bounds = Rect::new(1, 1, 69, 18);
    let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
    editor.set_text(sample_text);
    editor.set_auto_indent(true);

    window.add(Box::new(editor));

    // Make window modal
    window.set_state(window.state() | SF_MODAL);

    app.exec_view(Box::new(window));
}

fn demo_search_replace(app: &mut Application) {
    let sample_text = "=== Search and Replace Demo ===\n\
\n\
The editor has powerful find and replace:\n\
\n\
SEARCH FEATURES:\n\
- find(pattern, options) - Find first match\n\
- find_next() - Find next occurrence\n\
- Case-sensitive and case-insensitive search\n\
- Whole-word matching option\n\
- Forward and backward search\n\
\n\
REPLACE FEATURES:\n\
- replace_selection(text) - Replace current selection\n\
- replace_next(pattern, replacement, options) - Find and replace next\n\
- replace_all(pattern, replacement, options) - Replace all occurrences\n\
\n\
EXAMPLE PATTERNS:\n\
Try searching for: SEARCH, features, text\n\
Try replacing: SEARCH with FIND, or features with capabilities\n\
\n\
The search API matches Borland's TEditor:\n\
- SearchOptions with case_sensitive, whole_words_only, backwards\n\
- Maintains last search pattern and options\n\
- Efficient string searching algorithms\n\
\n\
Note: In this demo, search/replace functions are available\n\
in the Editor API but UI controls would need to be added.\n\
\n\
Press ESC to exit.";

    // Create modal window with editor
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(5, 3, 75, 22))
        .title("Search and Replace Demo")
        .build();

    let editor_bounds = Rect::new(1, 1, 69, 18);
    let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
    editor.set_text(sample_text);

    window.add(Box::new(editor));

    // Make window modal
    window.set_state(window.state() | SF_MODAL);

    app.exec_view(Box::new(window));
}

fn demo_syntax_highlighting(app: &mut Application) {
    let sample_code = r#"// === Syntax Highlighting Demo (Rust) ===

fn main() {
    // Comments are highlighted in cyan
    let x: i32 = 42;           // Types in green
    let name = "Hello!";       // Strings in red

    /* Block comments
       also supported */

    // Keywords in yellow
    for i in 0..10 {
        if i % 2 == 0 {
            println!("Even: {}", i);
        } else {
            println!("Odd: {}", i);
        }
    }

    // Different number formats
    let decimal = 123;
    let hex = 0xFF;            // Numbers in magenta
    let float = 3.14159;

    // String escapes
    let path = "C:\\Users\\test\\file.txt";

    // Custom types (capitalized)
    struct Point {
        x: f64,
        y: f64,
    }

    let origin = Point { x: 0.0, y: 0.0 };

    // More keywords
    match origin.x {
        0.0 => println!("At origin"),
        _ => println!("Not at origin"),
    }
}

// Syntax highlighting features:
// - Keywords: Yellow (fn, let, if, for, match, etc.)
// - Strings: Light Red ("text", 'c')
// - Comments: Light Cyan (// and /* */)
// - Numbers: Light Magenta (123, 0xFF, 3.14)
// - Types: Light Green (i32, f64, String, custom types)
// - Operators: White (+, -, ==, etc.)
// - Functions: Cyan
//
// The system is extensible - add more languages
// by implementing the SyntaxHighlighter trait!

"#;

    // Create modal window with editor
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(5, 3, 75, 22))
        .title("Syntax Highlighting Demo")
        .build();

    let editor_bounds = Rect::new(1, 1, 69, 18);
    let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
    editor.set_text(sample_code);

    // Enable Rust syntax highlighting
    editor.set_highlighter(Box::new(RustHighlighter::new()));

    window.add(Box::new(editor));

    // Make window modal
    window.set_state(window.state() | SF_MODAL);

    app.exec_view(Box::new(window));
}

fn demo_file_operations(app: &mut Application) {
    let sample_text = "=== File I/O Operations Demo ===\n\
\n\
The Editor supports file operations:\n\
\n\
LOADING FILES:\n\
  editor.load_file(\"path/to/file.txt\")?;\n\
  - Reads entire file into editor\n\
  - Sets filename for future saves\n\
  - Clears modified flag\n\
  - Resets undo history\n\
\n\
SAVING FILES:\n\
  editor.save_file()?;  // Save to current file\n\
  - Writes content to associated filename\n\
  - Clears modified flag\n\
  - Returns error if no filename set\n\
\n\
SAVE AS:\n\
  editor.save_as(\"new_file.txt\")?;\n\
  - Saves with new filename\n\
  - Updates filename for future saves\n\
  - Clears modified flag\n\
\n\
CHECKING STATE:\n\
  if editor.is_modified() {\n\
      // Prompt to save changes\n\
  }\n\
\n\
FILE INFO:\n\
  if let Some(path) = editor.get_filename() {\n\
      println!(\"Editing: {}\", path);\n\
  }\n\
\n\
EXAMPLE WORKFLOW:\n\
  let mut editor = Editor::new(bounds);\n\
  editor.load_file(\"config.toml\")?;\n\
  \n\
  // User edits...\n\
  \n\
  if editor.is_modified() {\n\
      editor.save_file()?;\n\
  }\n\
\n\
The API matches Borland's TFileEditor:\n\
- load_file() for reading\n\
- save_file() for writing to current file\n\
- save_as() for writing to new file\n\
- Modified flag tracking\n\
- Filename association\n\
\n\
In this demo, the file operations are shown as API\n\
examples. A real editor would add menu commands for\n\
File > Open, File > Save, File > Save As, etc.\n\
\n\
Press ESC to exit.";

    // Create modal window with editor
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(5, 3, 75, 22))
        .title("File Operations Demo")
        .build();

    let editor_bounds = Rect::new(1, 1, 69, 18);
    let mut editor = Editor::new(editor_bounds).with_scrollbars_and_indicator();
    editor.set_text(sample_text);

    window.add(Box::new(editor));

    // Make window modal
    window.set_state(window.state() | SF_MODAL);

    app.exec_view(Box::new(window));
}
