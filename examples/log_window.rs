// (C) 2025 - Enzo Lombardi
// LogWindow Demo - demonstrates tracing integration with a scrollable log window
//
// Shows how tracing::info!(), debug!(), warn!(), error!() macros
// automatically route to a LogWindow with colored, timestamped output
// on a black background.

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{KB_ALT_X, KB_F5};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::log_window::LogWindowBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};

use std::time::{Duration, Instant};

const CM_BURST: u16 = 100;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let (width, height) = app.terminal.size();

    // Status line
    let status_line = StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~F5~ Log burst", KB_F5, CM_BURST),
        ],
    );
    app.set_status_line(status_line);

    // Create the LogWindow — this installs the tracing subscriber
    let log_window = LogWindowBuilder::new()
        .bounds(Rect::new(2, 2, width - 2, height - 2))
        .title("Application Log")
        .min_level(tracing::Level::TRACE)
        .build();
    app.desktop.add(Box::new(log_window));

    // These go straight to the log window
    tracing::info!("Application started");
    tracing::info!("Terminal size: {}x{}", width, height);
    tracing::debug!("LogWindow created with tracing subscriber");

    // Simulate some activity
    tracing::info!("Loading configuration...");
    tracing::debug!("Config path: ./config.toml");
    tracing::info!("Configuration loaded successfully");
    tracing::warn!("Config key 'theme' not found, using default");
    tracing::trace!("Theme resolved to: classic-blue");

    tracing::info!("Initializing subsystems...");
    tracing::debug!("Subsystem 'network' initialized");
    tracing::debug!("Subsystem 'storage' initialized");
    tracing::debug!("Subsystem 'auth' initialized");
    tracing::info!("All subsystems ready");

    tracing::info!("Press F5 for a log burst, Alt-X to exit");

    // Timed log generation
    let mut burst_count = 0u32;
    let mut last_periodic = Instant::now();

    app.running = true;
    while app.running {
        app.draw();

        let event = app.get_event();
        if let Some(mut event) = event {
            if event.what == turbo_vision::core::event::EventType::Command {
                match event.command {
                    CM_QUIT => app.running = false,
                    CM_BURST => {
                        burst_count += 1;
                        tracing::info!("--- Log burst #{burst_count} ---");
                        tracing::error!("Simulated error: connection refused (10.0.0.1:5432)");
                        tracing::warn!("Retry attempt 1/3...");
                        tracing::warn!("Retry attempt 2/3...");
                        tracing::error!("Simulated error: timeout after 30s");
                        tracing::warn!("Retry attempt 3/3...");
                        tracing::info!("Connection established on retry");
                        tracing::debug!("Latency: 42ms, pool size: 5");
                        tracing::trace!("TCP handshake completed in 12ms");
                        tracing::info!("--- End burst #{burst_count} ---");
                        event.clear();
                    }
                    _ => {}
                }
            }
            app.handle_event(&mut event);
        }

        // Periodic log every 5 seconds
        if last_periodic.elapsed() >= Duration::from_secs(5) {
            tracing::trace!("Heartbeat: system OK");
            last_periodic = Instant::now();
        }
    }

    Ok(())
}
