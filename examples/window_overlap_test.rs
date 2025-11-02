// Example to test window redrawing with overlapping windows
//
// This example demonstrates that the desktop background is properly redrawn
// when windows move and overlap. It creates two overlapping windows that you
// can drag around. The background should remain clean without any visual trails.
//
// Usage:
//   cargo run --example window_overlap_test
//
// Instructions:
//   1. Drag the windows around by clicking and holding on the title bar
//   2. Overlap them to test that the background window gets redrawn
//   3. Verify there are no trails or visual artifacts
//   4. Press Escape twice to exit
//
// This test verifies the fix for the window dragging trails bug.

use turbo_vision::app::Application;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::{View, window::Window, static_text::StaticText};

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    // Create first window (background)
    let mut window1 = Window::new(
        Rect::new(5, 5, 45, 20),
        "Background Window"
    );

    let text1 = StaticText::new(
        Rect::new(2, 2, 38, 6),
        "This is the background window.\n\nDrag the other window over this\none to test redrawing."
    );
    window1.add(Box::new(text1));

    let text2 = StaticText::new(
        Rect::new(2, 8, 38, 13),
        "If you see trails or corruption\nwhen dragging, the bug exists.\n\nNo trails = bug is fixed!"
    );
    window1.add(Box::new(text2));

    app.desktop.add(Box::new(window1));

    // Create second window (foreground) - overlapping the first
    let mut window2 = Window::new(
        Rect::new(25, 10, 70, 25),
        "Foreground Window"
    );

    let text3 = StaticText::new(
        Rect::new(2, 2, 42, 7),
        "This window overlaps the other.\n\nDrag it around and watch the\nbackground window underneath.\n\nPress ESC ESC to exit."
    );
    window2.add(Box::new(text3));

    let text4 = StaticText::new(
        Rect::new(2, 9, 42, 12),
        "The desktop background should\nalways be clean with no trails!"
    );
    window2.add(Box::new(text4));

    app.desktop.add(Box::new(window2));

    // Run the application event loop
    // This properly redraws everything on each frame
    app.run();

    println!("\nOverlap test completed!");
    println!("If you saw no trails while dragging windows, the redrawing works correctly.");

    Ok(())
}
