// (C) 2025 - Enzo Lombardi
// Debug key codes to see what's actually being received
use turbo_vision::terminal::Terminal;
use turbo_vision::core::event::{KB_F11, KB_F12};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut terminal = Terminal::init()?;

    println!("Key Debug Test - Press keys to see their codes");
    println!("Expected F11: 0x{:04X}", KB_F11);
    println!("Expected F12: 0x{:04X}", KB_F12);
    println!("Press Ctrl+C to exit\n");

    loop {
        if let Ok(Some(event)) = terminal.poll_event(Duration::from_millis(50)) {
            println!("Event: {:?}", event);
            println!("  what: {:?}", event.what);
            println!("  key_code: 0x{:04X}", event.key_code);
            println!();

            if event.key_code == 0x0003 {
                break;
            }
        }
    }

    terminal.shutdown()?;
    Ok(())
}
