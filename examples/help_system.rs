// (C) 2025 - Enzo Lombardi
// Help System Example
//
// Demonstrates the markdown-based help system:
//
// **Markdown Format:**
// Topics are defined with headings and IDs:
//   # Topic Title {#topic-id}
//
// Cross-references (hyperlinks) are defined as:
//   [Link Text](#topic-id)
//
// The parser extracts these IDs and creates a topic database.
// Topics automatically detect cross-references and display them as "See also:" links.
//
// **Navigation in Help Window:**
// - Arrow keys / Page Up/Down: Scroll
// - Tab / Shift+Tab: Navigate between "See also:" links
// - Enter: Follow the selected link
// - ESC or Ctrl+X: Close help window

use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::prelude::*;
use turbo_vision::views::{
    help_file::HelpFile,
    help_window::HelpWindow,
};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Load the markdown help file
    let help_file = Rc::new(RefCell::new(
        HelpFile::new("examples/help.md").expect("Failed to load help file"),
    ));

    // Create a help window (designed specifically for help display)
    // This properly handles ESC, scrolling, and cross-references
    let mut help_window = HelpWindow::new(Rect::new(2, 0, 78, 25), "Help Topics", help_file.clone());

    // Show the default topic (first one in the markdown file)
    help_window.show_default_topic();

    // Debug: check what topic is displayed
    if let Some(topic_id) = help_window.current_topic() {
        eprintln!("Current topic: {}", topic_id);
    } else {
        eprintln!("ERROR: No topic loaded in help window!");
    }

    // Run the help window modally
    // Users can:
    // - Scroll with arrow keys or Page Up/Down
    // - Navigate cross-references with Tab/Shift+Tab + Enter
    // - Close with ESC or Ctrl+X
    help_window.execute(&mut app);

    Ok(())
}
