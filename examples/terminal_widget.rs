// (C) 2025 - Enzo Lombardi
// Terminal Widget Demo - demonstrates scrolling output viewer with simulated build log
//
// This matches Borland's terminal.cc example structure:
// - Terminal widget inside a Dialog/Window
// - With horizontal and vertical scrollbars
// - Buttons inside the same Dialog/Window
//
// Enhanced with:
// - Simulated streaming build log output
// - Color-coded messages (warnings, errors, success)
// - Interactive buttons to control log playback

use std::time::{Duration, Instant};
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::command_set;
use turbo_vision::core::event::{EventType, KB_ALT_X};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Attr;
use turbo_vision::views::View;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::terminal_widget::TerminalWidget;

const CM_START_LOG: u16 = 100;
const CM_STOP_LOG: u16 = 101;
const CM_CLEAR_LOG: u16 = 102;

// Simulated build log entries
const BUILD_LOG: &[(&str, u8)] = &[
    ("========================================", 0x07),
    ("Starting build process...", 0x0F),
    ("", 0x07),
    ("[00:00.000] Checking dependencies...", 0x07),
    ("[00:00.123] Found 156 packages", 0x0A),
    ("[00:00.234] Resolving package versions...", 0x07),
    ("[00:00.456] Resolution complete", 0x0A),
    ("", 0x07),
    ("[00:01.000] Compiling core v0.1.0", 0x0E),
    ("[00:01.234] Compiling utils v0.2.3", 0x0E),
    ("[00:01.567] Compiling config v1.0.0", 0x0E),
    ("[00:02.100] warning: unused import `std::fs`", 0x0C),
    ("[00:02.101]  --> src/config.rs:12:5", 0x08),
    ("[00:02.345] Compiling network v0.5.0", 0x0E),
    ("[00:03.123] Compiling database v2.1.0", 0x0E),
    ("[00:03.456] warning: field is never read: `timestamp`", 0x0C),
    ("[00:03.457]  --> src/database/model.rs:45:5", 0x08),
    ("[00:04.000] Compiling api v1.0.0", 0x0E),
    ("[00:05.234] Compiling server v0.8.0", 0x0E),
    ("", 0x07),
    ("[00:06.000] Building static assets...", 0x07),
    ("[00:06.500] Bundling JavaScript modules...", 0x07),
    ("[00:07.000] Minifying CSS files...", 0x07),
    ("[00:07.500] Optimizing images...", 0x07),
    ("[00:08.000] Assets ready", 0x0A),
    ("", 0x07),
    ("[00:08.500] Running post-build tasks...", 0x07),
    ("[00:09.000] Generating documentation...", 0x07),
    ("[00:09.500] Creating distribution package...", 0x07),
    ("", 0x07),
    ("[00:10.000] Build Summary:", 0x0F),
    ("[00:10.000] ================", 0x0F),
    ("[00:10.000] Compiled: 8 crates", 0x0A),
    ("[00:10.000] Warnings: 2", 0x0C),
    ("[00:10.000] Errors: 0", 0x0A),
    ("[00:10.000] Time: 10.00s", 0x0A),
    ("", 0x07),
    ("    Finished dev [unoptimized + debuginfo]", 0x0A),
    ("", 0x07),
    ("Build completed successfully!", 0x0A),
    ("========================================", 0x07),
];

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create status line
    let status_line = StatusLine::new(Rect::new(0, height as i16 - 1, width as i16, height as i16), vec![StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT)]);
    app.set_status_line(status_line);

    // Create dialog (matches Borland structure but larger for more content)
    let mut dialog = DialogBuilder::new().bounds(Rect::new(5, 2, 75, 22)).title("Build Output Terminal").build();

    // Add control buttons
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(54, 15, 66, 17)).title("~Q~uit").command(CM_QUIT).build()));

    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(2, 15, 14, 17)).title("~S~tart Log").command(CM_START_LOG).build()));

    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(16, 15, 28, 17)).title("S~t~op Log").command(CM_STOP_LOG).build()));
    command_set::disable_command(CM_STOP_LOG);

    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(30, 15, 42, 17)).title("~C~lear Log").command(CM_CLEAR_LOG).build()));
    command_set::disable_command(CM_CLEAR_LOG);

    // Create terminal widget inside dialog
    let mut terminal = TerminalWidget::new(Rect::new(2, 1, 66, 14)).with_scrollbar();

    // Add initial welcome message
    terminal.append_text("Build Output Viewer");
    terminal.append_text("==================");
    terminal.append_text("");
    terminal.append_text("Press 'Start Log' to simulate build output.");
    terminal.append_text("Use arrow keys or PgUp/PgDn to scroll.");
    terminal.append_text("");

    dialog.add(Box::new(terminal));

    // Terminal widget index in dialog (after 4 buttons: Quit, Start, Stop, Clear)
    const TERMINAL_INDEX: usize = 4;

    app.desktop.add(Box::new(dialog));
    // Log playback state
    let mut log_playing = false;
    let mut log_index = 0;
    let mut last_log_time = Instant::now();

    // Main event loop
    app.running = true;
    while app.running {
        // Call idle() to broadcast command set changes, then draw
        app.idle();
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Simulate log streaming
        if log_playing && log_index < BUILD_LOG.len() {
            if last_log_time.elapsed() >= Duration::from_millis(150) {
                // Access terminal widget
                let dialog_view = app.desktop.child_at_mut(0);
                if let Some(dialog) = dialog_view.as_any_mut().downcast_mut::<turbo_vision::views::dialog::Dialog>() {
                    let terminal_view = dialog.child_at_mut(TERMINAL_INDEX);
                    if let Some(terminal) = terminal_view.as_any_mut().downcast_mut::<TerminalWidget>() {
                        let (text, color) = BUILD_LOG[log_index];
                        terminal.append_line_colored(text.to_string(), Attr::from_u8(color));
                        log_index += 1;
                        last_log_time = Instant::now();
                    }
                }
            }
        }

        if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)).ok().flatten() {
            app.desktop.handle_event(&mut event);

            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            if event.what == EventType::Command {
                match event.command {
                    CM_START_LOG => {
                        command_set::disable_command(CM_START_LOG);
                        command_set::enable_command(CM_STOP_LOG);
                        command_set::enable_command(CM_CLEAR_LOG);

                        log_playing = true;
                        if log_index == 0 {
                            // Clear welcome message before starting
                            let dialog_view = app.desktop.child_at_mut(0);
                            if let Some(dialog) = dialog_view.as_any_mut().downcast_mut::<turbo_vision::views::dialog::Dialog>() {
                                let terminal_view = dialog.child_at_mut(TERMINAL_INDEX);
                                if let Some(terminal) = terminal_view.as_any_mut().downcast_mut::<TerminalWidget>() {
                                    terminal.clear();
                                }
                            }
                        }
                    }
                    CM_STOP_LOG => {
                        command_set::enable_command(CM_START_LOG);
                        command_set::disable_command(CM_STOP_LOG);
                        command_set::enable_command(CM_CLEAR_LOG);

                        log_playing = false;
                    }
                    CM_CLEAR_LOG => {
                        command_set::enable_command(CM_START_LOG);
                        command_set::disable_command(CM_STOP_LOG);
                        command_set::disable_command(CM_CLEAR_LOG);

                        log_playing = false;
                        log_index = 0;
                        let dialog_view = app.desktop.child_at_mut(0);
                        if let Some(dialog) = dialog_view.as_any_mut().downcast_mut::<turbo_vision::views::dialog::Dialog>() {
                            let terminal_view = dialog.child_at_mut(TERMINAL_INDEX);
                            if let Some(terminal) = terminal_view.as_any_mut().downcast_mut::<TerminalWidget>() {
                                terminal.clear();
                                terminal.append_text("Log cleared. Press 'Start Log' to replay.");
                                terminal.append_text("");
                            }
                        }
                    }
                    CM_QUIT => {
                        app.running = false;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
