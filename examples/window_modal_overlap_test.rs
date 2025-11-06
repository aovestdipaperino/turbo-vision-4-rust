// (C) 2025 - Enzo Lombardi
// Example to test modal vs non-modal window behavior
//
// This example demonstrates:
// 1. Non-modal windows: clicking brings them to front
// 2. Modal dialogs: clicking background windows has no effect
//
// Usage:
//   cargo run --example window_modal_overlap_test
//
// Instructions:
//   1. Initially you'll see two non-modal windows
//   2. Click on the background window - it should come to the front
//   3. Drag the windows around to test z-order and redrawing
//   4. Press ESC ESC to exit
//
// To test modal behavior, uncomment the modal dialog code below

use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::{
    window::Window,
    static_text::StaticText,
};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create first non-modal window (background)
    let mut window1 = Window::new(
        Rect::new(5, 3, 55, 16),
        "Non-Modal Window 1 (Background)"
    );

    let text1 = StaticText::new(
        Rect::new(2, 2, 46, 11),
        "This is a NON-MODAL window.\n\nClick anywhere on Window 2 to bring\nit to the front.\n\nThen click here to bring this\nwindow back to the front.\n\nPress ESC ESC to exit."
    );
    window1.add(Box::new(text1));

    app.desktop.add(Box::new(window1));

    // Create second non-modal window (foreground)
    let mut window2 = Window::new(
        Rect::new(20, 8, 70, 22),
        "Non-Modal Window 2 (Foreground)"
    );

    let text2 = StaticText::new(
        Rect::new(2, 2, 46, 12),
        "This is also NON-MODAL.\n\nClick on Window 1 behind this one\nto bring it to the front.\n\nYou can drag both windows around.\n\nTry clicking back and forth to\nsee z-order changes.\n\nPress ESC ESC to exit."
    );
    window2.add(Box::new(text2));

    app.desktop.add(Box::new(window2));

    // Create third overlapping window to make z-order more obvious
    let mut window3 = Window::new(
        Rect::new(35, 5, 78, 18),
        "Non-Modal Window 3"
    );

    let text3 = StaticText::new(
        Rect::new(2, 2, 39, 10),
        "Third NON-MODAL window.\n\nClick on any window behind to\nbring it forward.\n\nDrag windows to test overlap\nredrawing.\n\nPress ESC ESC to exit."
    );
    window3.add(Box::new(text3));

    app.desktop.add(Box::new(window3));

    // Run the application
    // The desktop will automatically handle:
    // - Bringing clicked windows to front
    // - Z-order management
    // - Proper redrawing on overlaps
    app.run();

    println!("\nModal/Non-Modal test completed!");
    println!("\nWhat you should have observed:");
    println!("  1. Three overlapping non-modal windows");
    println!("  2. Clicking any window brings it to the front");
    println!("  3. Windows can be dragged and overlap correctly");
    println!("  4. Background redraws properly (no trails)");
    println!("\nTo test modal behavior:");
    println!("  - Modal dialogs block interaction with background");
    println!("  - See the file_dialog example for modal behavior");

    Ok(())
}
