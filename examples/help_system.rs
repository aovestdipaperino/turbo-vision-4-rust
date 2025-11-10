// (C) 2025 - Enzo Lombardi
// Help System Example
//
// Demonstrates the markdown-based help system with:
// - HelpFile - Parses and manages markdown help files
// - HelpViewer - Displays help content with scrolling
// - HelpWindow - Window wrapper for help viewer
// - Context-sensitive help support

use std::cell::RefCell;
use std::rc::Rc;
//use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_OK};
//use turbo_vision::core::geometry::Rect;
use turbo_vision::prelude::*;
use turbo_vision::views::{
    button::ButtonBuilder, dialog::DialogBuilder, help_context::HelpContext, help_file::HelpFile,
    help_window::HelpWindow, static_text::StaticTextBuilder,
};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Load help file
    let help_file = Rc::new(RefCell::new(
        HelpFile::new("examples/help.md").expect("Failed to load help file"),
    ));

    // Create help context manager
    let mut help_ctx = HelpContext::new();
    help_ctx.register(100, "welcome");
    help_ctx.register(101, "file-menu");
    help_ctx.register(102, "edit-menu");
    help_ctx.register(103, "search");

    // Create main dialog
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(15, 5, 65, 18))
        .title("Help System Demo")
        .build();

    let text = StaticTextBuilder::new()
        .bounds(Rect::new(17, 7, 63, 11))
        .text("Press buttons to show different help topics:\n\n\
         - Welcome: Introduction\n\
         - File Menu: File operations\n\
         - Edit Menu: Editing commands")
        .build();
    dialog.add(Box::new(text));

    // Welcome button
    let welcome_btn = ButtonBuilder::new()
        .bounds(Rect::new(20, 12, 34, 14))
        .title("~W~elcome")
        .command(100)
        .default(false)
        .build();
    dialog.add(Box::new(welcome_btn));

    // File Menu button
    let file_btn = ButtonBuilder::new()
        .bounds(Rect::new(36, 12, 48, 14))
        .title("~F~ile")
        .command(101)
        .default(false)
        .build();
    dialog.add(Box::new(file_btn));

    // Edit Menu button
    let edit_btn = ButtonBuilder::new()
        .bounds(Rect::new(50, 12, 62, 14))
        .title("~E~dit")
        .command(102)
        .default(false)
        .build();
    dialog.add(Box::new(edit_btn));

    // OK button
    let ok_btn = ButtonBuilder::new()
        .bounds(Rect::new(37, 15, 47, 17))
        .title("  OK  ")
        .command(CM_OK)
        .default(true)
        .build();
    dialog.add(Box::new(ok_btn));

    // Show dialog and handle help buttons
    loop {
        let result = dialog.execute(&mut app);

        match result {
            100 => show_help(&mut app, help_file.clone(), "welcome"),
            101 => show_help(&mut app, help_file.clone(), "file-menu"),
            102 => show_help(&mut app, help_file.clone(), "edit-menu"),
            CM_OK | CM_CANCEL => break,
            _ => {}
        }
    }

    Ok(())
}

fn show_help(app: &mut Application, help_file: Rc<RefCell<HelpFile>>, topic_id: &str) {
    let mut help_window = HelpWindow::new(Rect::new(10, 3, 70, 22), "Help", help_file);

    if help_window.show_topic(topic_id) {
        help_window.execute(app);
    } else {
        eprintln!("Help topic not found: {}", topic_id);
        help_window.show_default_topic();
        help_window.execute(app);
    }
}
