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

use turbo_vision::prelude::*;

use std::time::{Duration, Instant};
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::command_set;
use turbo_vision::core::event::KB_ALT_X;
use turbo_vision::core::palette::Attr;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::dialog::{Dialog, DialogBuilder};
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::terminal_widget::TerminalWidget;
use turbo_vision::views::view::ViewId;

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

/// Application state for log playback
struct LogState {
    playing: bool,
    index: usize,
    last_update: Instant,
}

impl LogState {
    fn new() -> Self {
        Self {
            playing: false,
            index: 0,
            last_update: Instant::now(),
        }
    }

    fn reset(&mut self) {
        self.playing = false;
        self.index = 0;
    }

    fn start(&mut self) {
        self.playing = true;
    }

    fn stop(&mut self) {
        self.playing = false;
    }

    fn should_update(&self) -> bool {
        self.playing && self.index < BUILD_LOG.len() && self.last_update.elapsed() >= Duration::from_millis(150)
    }
}

/// Helper function to get terminal widget from dialog
fn get_terminal_widget(app: &mut Application, term_vid: ViewId) -> Option<&mut TerminalWidget> {
    let dialog_view = app.desktop.child_at_mut(0);
    let dialog = dialog_view.as_any_mut().downcast_mut::<Dialog>()?;
    let terminal_view = dialog.child_by_id_mut(term_vid)?;
    terminal_view.as_any_mut().downcast_mut::<TerminalWidget>()
}

/// Create and setup the main dialog with terminal and buttons
fn create_dialog() -> (Box<Dialog>, ViewId) {
    let mut dialog = DialogBuilder::new().bounds(Rect::new(5, 2, 75, 22)).title("Build Output Terminal").build();

    // Add control buttons
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(54, 15, 66, 17)).title("~Q~uit").command(CM_QUIT).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(2, 15, 14, 17)).title("~S~tart Log").command(CM_START_LOG).build()));
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(16, 15, 28, 17)).title("S~t~op Log").command(CM_STOP_LOG).build()));
    command_set::disable_command(CM_STOP_LOG);
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(30, 15, 42, 17)).title("~C~lear Log").command(CM_CLEAR_LOG).build()));
    command_set::disable_command(CM_CLEAR_LOG);

    // Create terminal widget with welcome message
    let mut terminal = TerminalWidget::new(Rect::new(2, 1, 66, 14)).with_scrollbar();
    terminal.append_text("Build Output Viewer");
    terminal.append_text("==================");
    terminal.append_text("");
    terminal.append_text("Press 'Start Log' to simulate build output.");
    terminal.append_text("Use arrow keys or PgUp/PgDn to scroll.");
    terminal.append_text("");

    let term_vid = dialog.add(Box::new(terminal));

    (Box::new(dialog), term_vid)
}

/// Update log streaming - add next line to terminal
fn update_log_streaming(app: &mut Application, term_vid: ViewId, log_state: &mut LogState) {
    if !log_state.should_update() {
        return;
    }

    if let Some(terminal) = get_terminal_widget(app, term_vid) {
        let (text, color) = BUILD_LOG[log_state.index];
        terminal.append_line_colored(text.to_string(), Attr::from_u8(color));
        log_state.index += 1;
        log_state.last_update = Instant::now();
    }
}

/// Handle START_LOG command
fn handle_start_log(app: &mut Application, term_vid: ViewId, log_state: &mut LogState) {
    command_set::disable_command(CM_START_LOG);
    command_set::enable_command(CM_STOP_LOG);
    command_set::enable_command(CM_CLEAR_LOG);

    log_state.start();
    if log_state.index == 0 {
        if let Some(terminal) = get_terminal_widget(app, term_vid) {
            terminal.clear();
        }
    }
}

/// Handle STOP_LOG command
fn handle_stop_log(log_state: &mut LogState) {
    command_set::enable_command(CM_START_LOG);
    command_set::disable_command(CM_STOP_LOG);
    command_set::enable_command(CM_CLEAR_LOG);

    log_state.stop();
}

/// Handle CLEAR_LOG command
fn handle_clear_log(app: &mut Application, term_vid: ViewId, log_state: &mut LogState) {
    command_set::enable_command(CM_START_LOG);
    command_set::disable_command(CM_STOP_LOG);
    command_set::disable_command(CM_CLEAR_LOG);

    log_state.reset();
    if let Some(terminal) = get_terminal_widget(app, term_vid) {
        terminal.clear();
        terminal.append_text("Log cleared. Press 'Start Log' to replay.");
        terminal.append_text("");
    }
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create status line
    let (width, height) = app.terminal.size();
    let status_line = StatusLine::new(Rect::new(0, height - 1, width, height), vec![StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT)]);
    app.set_status_line(status_line);

    // Create and add dialog
    let (dialog, term_vid) = create_dialog();
    app.desktop.add(dialog);

    // Log playback state
    let mut log_state = LogState::new();

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

        // Update log streaming
        update_log_streaming(&mut app, term_vid, &mut log_state);

        // Handle events
        if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)).ok().flatten() {
            app.desktop.handle_event(&mut event);

            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            if event.what == EventType::Command {
                match event.command {
                    CM_START_LOG => handle_start_log(&mut app, term_vid, &mut log_state),
                    CM_STOP_LOG => handle_stop_log(&mut log_state),
                    CM_CLEAR_LOG => handle_clear_log(&mut app, term_vid, &mut log_state),
                    CM_QUIT => app.running = false,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
