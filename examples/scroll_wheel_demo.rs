use turbo_vision::prelude::*;
use turbo_vision::app::Application;
use turbo_vision::views::{
    window::Window,
    text_viewer::TextViewer,
    listbox::ListBox,
    memo::Memo,
};

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Create a window with TextView showing a long document
    let mut text_window = Window::new(
        Rect::new(2, 1, 50, 20),
        "TextView - Scroll with Mouse Wheel"
    );

    // Use relative coordinates (0-based, relative to window interior)
    let mut text_viewer = TextViewer::new(Rect::new(0, 0, 46, 17))
        .with_scrollbars(true)
        .with_indicator(true);

    // Generate a long text document
    let long_text = generate_long_text();
    text_viewer.set_text(&long_text);

    text_window.add(Box::new(text_viewer));
    app.desktop.add(Box::new(text_window));

    // Create a window with ListBox
    let mut list_window = Window::new(
        Rect::new(52, 1, 95, 20),
        "ListBox - Scroll with Mouse Wheel"
    );

    // Use relative coordinates
    let mut listbox = ListBox::new(Rect::new(0, 0, 41, 17), 1000);

    // Add many items to the list
    for i in 1..=100 {
        listbox.add_item(format!("List Item {:3} - Scroll with wheel!", i));
    }

    list_window.add(Box::new(listbox));
    app.desktop.add(Box::new(list_window));

    // Create a window with Memo
    let mut memo_window = Window::new(
        Rect::new(22, 10, 75, 24),
        "Memo - Scroll with Mouse Wheel"
    );

    // Use relative coordinates
    let mut memo = Memo::new(Rect::new(0, 0, 51, 12))
        .with_scrollbars(true);

    // Set some editable text
    memo.set_text(&generate_memo_text());

    memo_window.add(Box::new(memo));
    app.desktop.add(Box::new(memo_window));

    app.run();
    Ok(())
}

fn generate_long_text() -> String {
    let mut text = String::new();

    text.push_str("SCROLL WHEEL DEMO - TextView\n");
    text.push_str("============================\n\n");
    text.push_str("This example demonstrates mouse wheel scrolling in Turbo Vision.\n\n");
    text.push_str("FEATURES:\n");
    text.push_str("- Mouse wheel scrolls content up and down\n");
    text.push_str("- Works in TextView, ListBox, and Memo components\n");
    text.push_str("- Only scrolls when mouse is over the component\n");
    text.push_str("- Keyboard navigation still works (arrows, PgUp/PgDn)\n\n");
    text.push_str("INSTRUCTIONS:\n");
    text.push_str("1. Hover your mouse over this window\n");
    text.push_str("2. Scroll the mouse wheel up and down\n");
    text.push_str("3. Notice the content scrolls smoothly\n");
    text.push_str("4. Try the ListBox window on the right\n");
    text.push_str("5. Try the Memo window at the bottom\n\n");
    text.push_str("TECHNICAL DETAILS:\n");
    text.push_str("=================\n\n");

    for i in 1..=50 {
        text.push_str(&format!("Line {:3}: The mouse wheel support was added in version 0.1.3. ", i));
        text.push_str("It uses crossterm's ScrollUp and ScrollDown events, which are ");
        text.push_str("converted to internal MouseWheelUp and MouseWheelDown event types. ");
        text.push_str("Each scrollable component checks if the mouse is within its bounds ");
        text.push_str("before handling the wheel event. This prevents multiple components ");
        text.push_str("from scrolling simultaneously.\n");
    }

    text.push_str("\n");
    text.push_str("ORIGINAL TURBO VISION:\n");
    text.push_str("=====================\n\n");
    text.push_str("The original Borland Turbo Vision framework (1990s) did not have\n");
    text.push_str("mouse wheel support because mouse wheels didn't exist at that time!\n");
    text.push_str("The first mouse with a scroll wheel was the Microsoft IntelliMouse,\n");
    text.push_str("released in 1996. Turbo Vision was created in 1990.\n\n");
    text.push_str("This implementation brings modern mouse wheel support to the classic\n");
    text.push_str("Turbo Vision architecture, while maintaining compatibility with the\n");
    text.push_str("original event-driven design.\n\n");

    for i in 51..=100 {
        text.push_str(&format!("Line {:3}: Keep scrolling to see more content! ", i));
        text.push_str("You can also use the keyboard: Arrow keys for line-by-line, ");
        text.push_str("PgUp/PgDn for page-by-page, Home/End for horizontal scrolling. ");
        text.push_str("The scrollbars on the right and bottom can also be clicked. ");
        text.push_str("Notice how the indicator at the top shows your current position.\n");
    }

    text.push_str("\n=== END OF DOCUMENT ===\n");

    text
}

fn generate_memo_text() -> String {
    let mut text = String::new();

    text.push_str("MEMO COMPONENT DEMO\n");
    text.push_str("===================\n\n");
    text.push_str("This is an editable Memo component.\n\n");
    text.push_str("TRY THESE:\n");
    text.push_str("- Scroll with mouse wheel\n");
    text.push_str("- Click to position cursor\n");
    text.push_str("- Type to edit text\n");
    text.push_str("- Use arrow keys to navigate\n");
    text.push_str("- Ctrl+A to select all\n");
    text.push_str("- Ctrl+C to copy\n");
    text.push_str("- Ctrl+V to paste\n");
    text.push_str("- Ctrl+X to cut\n\n");

    for i in 1..=30 {
        text.push_str(&format!("Editable line {}: You can modify this text!\n", i));
    }

    text
}
