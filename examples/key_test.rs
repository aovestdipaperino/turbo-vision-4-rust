// (C) 2025 - Enzo Lombardi
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{self};

fn main() -> io::Result<()> {
    // Setup terminal
    execute!(io::stdout(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    println!("Press Shift+Arrow keys (or any keys) to see what crossterm sends.");
    println!("Press ESC to exit.\r");
    println!("\r");

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    println!("Key: code={:?}, modifiers={:?}\r", key.code, key.modifiers);

                    if key.code == KeyCode::Esc {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    // Cleanup
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
