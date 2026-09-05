// (C) 2026 - Enzo Lombardi
// Screen Capture Example
//
// Demonstrates the screen-capture shortcuts:
//   F12      -> ASCII (ANSI) dump of the whole screen  -> screen-*.ans
//   Ctrl+F12 -> PNG screenshot of the whole screen      -> screenshot-*.png
// Both files land in the working directory. Exit with Alt+X (or Esc-X on macOS).
//
// Run with:
//   cargo run --example screenshot
//
// Testing without a physical key press: some terminals (e.g. macOS Terminal.app)
// do not forward Ctrl+F12. Enable the TCP remote-input listener and inject the
// chord instead:
//
//   TV_REMOTE_KEYS=8888 cargo run --example screenshot
//   # then, from another shell:
//   printf 'F12\n'             | nc 127.0.0.1 8888   # ASCII dump
//   printf 'CTRL+F12\n'        | nc 127.0.0.1 8888   # PNG screenshot
//   printf 'CTRL+F12 ALT+X\n'  | nc 127.0.0.1 8888   # screenshot, then quit

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_QUIT, CM_SCREENSHOT};
use turbo_vision::core::event::{KB_ALT_X, KB_ESC, KB_F12};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::GroupLike;
use turbo_vision::views::label::LabelBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::window::WindowBuilder;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Status line advertises the capture shortcuts.
    let status_line = StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItem::new("~F12~ ASCII", KB_F12, 0),
            StatusItem::new("~Ctrl+F12~ PNG", 0, CM_SCREENSHOT),
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Esc-X~ Exit", KB_ESC, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // A window with a frame and some colorful content to capture.
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(12, 4, 68, 17))
        .title("Screenshot Demo")
        .build();

    let intro = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, 54, 4))
        .text("\x03F12 = ASCII dump   Ctrl+F12 = PNG screenshot\n\nBoth capture the whole screen to the current\ndirectory. The PNG is rendered from the cell buffer.")
        .build();

    let charset = StaticTextBuilder::new()
        .bounds(Rect::new(2, 6, 54, 9))
        .text("ABCDEFG abcdefg 0123456789\n!@#$%^&*()_+-=[]{};:'\",.<>/?\nBox drawing and shades below:")
        .build();

    window.add(Box::new(intro));
    window.add(Box::new(charset));

    app.desktop.add(Box::new(window));

    // A second, differently-colored window so the screenshot shows overlap,
    // frames, and the dithered desktop background.
    let mut info = WindowBuilder::new()
        .bounds(Rect::new(20, 10, 60, 20))
        .title("Info")
        .build();
    let note = LabelBuilder::new()
        .bounds(Rect::new(2, 2, 36, 2))
        .text("Files land in the current directory:")
        .build();
    let note2 = LabelBuilder::new()
        .bounds(Rect::new(2, 4, 36, 4))
        .text("screenshot-YYYYMMDD-HHMMSS.png")
        .build();
    info.add(Box::new(note));
    info.add(Box::new(note2));
    app.desktop.add(Box::new(info));

    app.run();

    Ok(())
}
