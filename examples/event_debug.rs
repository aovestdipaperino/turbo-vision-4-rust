// (C) 2025 - Enzo Lombardi
// Event Debug Tool
// Prints and log all events to help diagnose mouse/keyboard issues

use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::event::{EventType, KB_CTRL_C};

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut log_file = OpenOptions::new().create(true).write(true).truncate(true).open("event_test.txt").expect("Failed to create log file");

    // Macro to log both to stderr and to the log file
    macro_rules! log {
        ($($arg:tt)*) => {{
            eprintln!($($arg)*);
            writeln!(log_file, $($arg)*).ok();
        }};
    }

    log!("\n=== Event Debug Tool ===");
    log!("This tool prints all events to help diagnose input issues.");
    log!("- Try clicking the mouse");
    log!("- Try pressing keys");
    log!("- Press Ctrl+C to exit");
    log!("========================\n");

    let mut event_count = 0;
    app.running = true;

    while app.running {
        if let Ok(Some(event)) = app.terminal.poll_event(Duration::from_millis(100)) {
            event_count += 1;

            match event.what {
                EventType::MouseDown => {
                    log!(
                        "[{}] MouseDown at ({}, {}) buttons=0x{:02x} double={}",
                        event_count,
                        event.mouse.pos.x,
                        event.mouse.pos.y,
                        event.mouse.buttons,
                        event.mouse.double_click
                    );
                }
                EventType::MouseUp => {
                    log!("[{}] MouseUp at ({}, {})", event_count, event.mouse.pos.x, event.mouse.pos.y);
                }
                EventType::MouseMove => {
                    // Only print occasionally to avoid spam
                    if event_count % 10 == 0 {
                        log!("[{}] MouseMove at ({}, {}) buttons=0x{:02x}", event_count, event.mouse.pos.x, event.mouse.pos.y, event.mouse.buttons);
                    }
                }
                EventType::Keyboard => {
                    log!("[{}] Keyboard key_code=0x{:04x}", event_count, event.key_code);

                    // Exit on Ctrl+C
                    if event.key_code == KB_CTRL_C {
                        eprintln!("\nCtrl+C detected, exiting...");
                        app.running = false;
                    }
                }
                EventType::Command => {
                    log!("[{}] Command command={}", event_count, event.command);
                }
                _ => {
                    log!("[{}] Other event type: {:?}", event_count, event.what);
                }
            }
        }
    }

    log!("\nTotal events captured: {}", event_count);
    Ok(())
}
