//! SSH TUI Server Example
//!
//! This example demonstrates how to serve a turbo-vision application over SSH.
//!
//! # Running
//!
//! ```bash
//! cargo run --example ssh_server --features ssh
//! ```
//!
//! # Connecting
//!
//! ```bash
//! ssh -p 2222 user@localhost
//! ```
//!
//! Any password will work (demo only - implement real auth for production!).

use std::time::Duration;

use turbo_vision::prelude::*;
use turbo_vision::terminal::{Backend, Terminal};
use turbo_vision::views::{
    button::Button,
    dialog::Dialog,
    static_text::StaticText,
};
use turbo_vision::ssh::{SshServer, SshServerConfig};

/// Run the TUI application with the provided backend.
fn run_tui_app(backend: Box<dyn Backend>) {
    log::info!("TUI app starting...");
    let terminal = match Terminal::with_backend(backend) {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to create terminal: {}", e);
            return;
        }
    };
    log::info!("Terminal created, running TUI...");

    run_tui_inner(terminal);
    log::info!("TUI app finished.");
}

fn run_tui_inner(mut terminal: Terminal) {
    // Get terminal size
    let (width, height) = terminal.size();

    // Create a dialog
    let dialog_width: i16 = 50;
    let dialog_height: i16 = 12;
    let dialog_x = (width - dialog_width) / 2;
    let dialog_y = (height - dialog_height) / 2;

    let mut dialog = Dialog::new_modal(
        Rect::new(dialog_x, dialog_y, dialog_x + dialog_width, dialog_y + dialog_height),
        "SSH Admin Console"
    );

    // Add welcome text
    let text = StaticText::new(
        Rect::new(2, 2, dialog_width - 4, 4),
        "Welcome to the turbo-vision SSH TUI Demo!\n\nThis interface is served over SSH."
    );
    dialog.add(Box::new(text));

    // Add quit button
    let button = Button::new(
        Rect::new((dialog_width - 12) / 2, dialog_height - 4, (dialog_width + 12) / 2, dialog_height - 2),
        "Quit",
        CM_QUIT,
        true
    );
    dialog.add(Box::new(button));

    // Event loop
    log::info!("Entering event loop...");
    let mut running = true;
    let mut frame_count = 0;
    while running {
        // Draw
        terminal.clear();
        dialog.draw(&mut terminal);
        if terminal.flush().is_err() {
            log::error!("Flush failed, breaking loop");
            break;
        }
        frame_count += 1;
        if frame_count <= 3 {
            log::info!("Frame {} drawn and flushed", frame_count);
        }

        // Handle events
        if let Ok(Some(mut event)) = terminal.poll_event(Duration::from_millis(50)) {
            dialog.handle_event(&mut event);

            // Check for quit command
            if event.what == EventType::Command && event.command == CM_QUIT {
                running = false;
            }

            // Also quit on Escape
            if event.what == EventType::Keyboard && event.key_code == turbo_vision::core::event::KB_ESC {
                running = false;
            }
        }
    }

    let _ = terminal.shutdown();
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize file logging
    let log_file = std::fs::File::create("ssh_server.log")?;
    let _ = simplelog::WriteLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        log_file,
    );

    let config = SshServerConfig::new()
        .bind_addr("0.0.0.0:2222")
        .load_or_generate_key("ssh_host_key");

    println!("=== SSH TUI Server ===");
    println!();
    println!("Server listening on port 2222");
    println!();
    println!("Connect with:");
    println!("  ssh -p 2222 user@localhost");
    println!();
    println!("Any password will work (demo only!)");
    println!();
    println!("Press Ctrl+C to stop the server");
    println!();

    let server = SshServer::new(config, || {
        Box::new(|backend| {
            run_tui_app(backend);
        })
    });

    server.run().await?;

    Ok(())
}
