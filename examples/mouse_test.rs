// (C) 2025 - Enzo Lombardi
// Simple mouse test - prints mouse events to verify mouse capture works

use turbo_vision::app::Application;
use turbo_vision::core::event::EventType;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;

    eprintln!("Mouse test started. Click anywhere or press Ctrl+C to exit.");
    eprintln!("If you see 'MouseDown' messages, mouse capture is working.");

    app.running = true;
    let mut event_count = 0;

    while app.running && event_count < 10 {
        if let Ok(Some(event)) = app.terminal.poll_event(Duration::from_millis(100)) {
            eprintln!("Event received: {:?}", event);

            match event.what {
                EventType::MouseDown => {
                    eprintln!("  -> Mouse click at ({}, {})", event.mouse.pos.x, event.mouse.pos.y);
                    event_count += 1;
                }
                EventType::Keyboard => {
                    if event.key_code == 0x0003 { // Ctrl+C
                        app.running = false;
                    }
                }
                _ => {}
            }
        }
    }

    eprintln!("Mouse test complete. Captured {} mouse clicks.", event_count);
    Ok(())
}
