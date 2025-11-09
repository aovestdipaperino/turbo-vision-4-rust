// (C) 2025 - Enzo Lombardi
// Event Debug Tool
// Prints all events to help diagnose mouse/keyboard issues

use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::event::EventType;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    eprintln!("\n=== Event Debug Tool ===");
    eprintln!("This tool prints all events to help diagnose input issues.");
    eprintln!("- Try clicking the mouse");
    eprintln!("- Try pressing keys");
    eprintln!("- Press Ctrl+C to exit");
    eprintln!("========================\n");

    let mut event_count = 0;
    app.running = true;

    while app.running {
        if let Ok(Some(event)) = app.terminal.poll_event(Duration::from_millis(100)) {
            event_count += 1;

            match event.what {
                EventType::MouseDown => {
                    eprintln!(
                        "[{}] MouseDown at ({}, {}) buttons=0x{:02x} double={}",
                        event_count,
                        event.mouse.pos.x,
                        event.mouse.pos.y,
                        event.mouse.buttons,
                        event.mouse.double_click
                    );
                }
                EventType::MouseUp => {
                    eprintln!(
                        "[{}] MouseUp at ({}, {})",
                        event_count, event.mouse.pos.x, event.mouse.pos.y
                    );
                }
                EventType::MouseMove => {
                    // Only print occasionally to avoid spam
                    if event_count % 10 == 0 {
                        eprintln!(
                            "[{}] MouseMove at ({}, {}) buttons=0x{:02x}",
                            event_count, event.mouse.pos.x, event.mouse.pos.y, event.mouse.buttons
                        );
                    }
                }
                EventType::Keyboard => {
                    eprintln!(
                        "[{}] Keyboard key_code=0x{:04x}",
                        event_count, event.key_code
                    );

                    // Exit on Ctrl+C
                    if event.key_code == 0x0003 {
                        eprintln!("\nCtrl+C detected, exiting...");
                        app.running = false;
                    }
                }
                EventType::Command => {
                    eprintln!("[{}] Command command={}", event_count, event.command);
                }
                _ => {
                    eprintln!("[{}] Other event type: {:?}", event_count, event.what);
                }
            }
        }
    }

    eprintln!("\nTotal events captured: {}", event_count);
    Ok(())
}
