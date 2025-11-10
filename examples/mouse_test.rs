// (C) 2025 - Enzo Lombardi
// Simple mouse test - prints mouse events to verify mouse capture works

use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::event::{EventType, KB_CTRL_C};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut log_file = OpenOptions::new().create(true).write(true).truncate(true).open("mouse_test.txt").expect("Failed to create log file");

    // Macro to log both to stderr and to the log file
    macro_rules! log {
        ($($arg:tt)*) => {{
            eprintln!($($arg)*);
            writeln!(log_file, $($arg)*).ok();
        }};
    }

    log!("Mouse test started. Click anywhere or press Ctrl+C to exit.");
    log!("If you see 'MouseDown' messages, mouse capture is working.");

    app.running = true;
    let mut event_count = 0;

    while app.running && event_count < 10 {
        if let Ok(Some(event)) = app.terminal.poll_event(Duration::from_millis(100)) {
            log!("Event received: {:?}", event);

            match event.what {
                EventType::MouseDown => {
                    log!("  -> Mouse click at ({}, {})", event.mouse.pos.x, event.mouse.pos.y);
                    event_count += 1;
                }
                EventType::Keyboard => {
                    if event.key_code == KB_CTRL_C {
                        app.running = false;
                    }
                }
                _ => {}
            }
        }
    }

    log!("Mouse test complete. Captured {} mouse clicks.", event_count);
    Ok(())
}
