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
    button::Button, dialog::Dialog, help_context::HelpContext, help_file::HelpFile,
    help_window::HelpWindow, static_text::StaticText,
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
    let mut dialog = Dialog::new(Rect::new(15, 5, 65, 18), "Help System Demo");

    let text = StaticText::new(
        Rect::new(17, 7, 63, 11),
        "Press buttons to show different help topics:\n\n\
         - Welcome: Introduction\n\
         - File Menu: File operations\n\
         - Edit Menu: Editing commands",
    );
    dialog.add(Box::new(text));

    // Welcome button
    let welcome_btn = Button::new(Rect::new(20, 12, 34, 14), "~W~elcome", 100, false);
    dialog.add(Box::new(welcome_btn));

    // File Menu button
    let file_btn = Button::new(Rect::new(36, 12, 48, 14), "~F~ile", 101, false);
    dialog.add(Box::new(file_btn));

    // Edit Menu button
    let edit_btn = Button::new(Rect::new(50, 12, 62, 14), "~E~dit", 102, false);
    dialog.add(Box::new(edit_btn));

    // OK button
    let ok_btn = Button::new(Rect::new(37, 15, 47, 17), "  OK  ", CM_OK, true);
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
