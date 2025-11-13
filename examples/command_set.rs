// (C) 2025 - Enzo Lombardi
//! Command Set System Demo
//!
//! Demonstrates button enable/disable based on application state.
//! Shows how the command set system works like Borland Turbo Vision.
//!
//! ## What This Demo Shows:
//!
//! 1. Buttons are automatically created in disabled state if their command is disabled
//! 2. The Application `idle()` loop broadcasts `CM_COMMAND_SET_CHANGED` when commands change
//! 3. Buttons automatically update their disabled state when they receive the broadcast
//!
//! ## Try It:
//!
//! - Notice that Copy, Cut, Paste, Undo, Redo buttons start disabled (grayed)
//! - Press 'E' to enable edit commands - buttons turn green
//! - Press 'D' to disable them again - buttons turn gray
//! - This happens automatically through the command set broadcast system (`app::idle()`)

use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_COPY, CM_CUT, CM_PASTE, CM_QUIT, CM_REDO, CM_UNDO, CommandId};
use turbo_vision::core::command_set;
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::{button::ButtonBuilder, dialog::DialogBuilder, static_text::StaticTextBuilder};

// Custom commands for this demo
const CMD_ENABLE_EDITS: CommandId = 200;
const CMD_DISABLE_EDITS: CommandId = 201;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create and add the dialog
    let dialog = create_dialog();
    app.desktop.add(Box::new(dialog)); // non modal dialog
    app.running = true; // set application running state

    while app.running {
        // Application's event handling
        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            app.handle_event(&mut event);

            // Check for our custom commands
            if event.what == EventType::Command {
                match event.command {
                    CMD_ENABLE_EDITS => {
                        command_set::enable_command(CM_COPY);
                        command_set::enable_command(CM_CUT);
                        command_set::enable_command(CM_PASTE);
                        command_set::enable_command(CM_UNDO);
                        command_set::enable_command(CM_REDO);

                        command_set::enable_command(CMD_DISABLE_EDITS);
                        command_set::disable_command(CMD_ENABLE_EDITS);
                    }
                    CMD_DISABLE_EDITS => {
                        command_set::disable_command(CM_COPY);
                        command_set::disable_command(CM_CUT);
                        command_set::disable_command(CM_PASTE);
                        command_set::disable_command(CM_UNDO);
                        command_set::disable_command(CM_REDO);

                        command_set::enable_command(CMD_ENABLE_EDITS);
                        command_set::disable_command(CMD_DISABLE_EDITS);
                    }
                    _ => {}
                }
            }
        }

        // CRITICAL: Call idle() to broadcast command set changes, then draw
        app.idle();
        app.draw();
        let _ = app.terminal.flush();
    }

    Ok(())
}

/// Creates and configures the command set demo dialog
fn create_dialog() -> turbo_vision::views::dialog::Dialog {
    let mut dialog = DialogBuilder::new().bounds(Rect::new(10, 4, 72, 21)).title("Button State Management Demo").build();

    // Instructions
    let instructions = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, 56, 6))
        .text(
            r"This demo shows button state management.

Edit commands are initially disabled (grayed out).
Press ~E~ to enable edit commands - buttons turn green.
Press ~D~ to disable edit commands - buttons turn gray.",
        )
        .build();
    dialog.add(Box::new(instructions));

    // Close button - add it FIRST so it gets initial focus
    let close_button = ButtonBuilder::new().bounds(Rect::new(22, 13, 38, 15)).title(" Close ").command(CM_QUIT).default(true).build();
    dialog.add(Box::new(close_button));

    // Edit command buttons (will be initially be disabled due to command set)
    let cut_button = ButtonBuilder::new().bounds(Rect::new(2, 7, 14, 9)).title(" C~u~t ").command(CM_CUT).build();
    dialog.add(Box::new(cut_button));

    let copy_button = ButtonBuilder::new().bounds(Rect::new(15, 7, 27, 9)).title(" ~C~opy ").command(CM_COPY).build();
    dialog.add(Box::new(copy_button));

    let paste_button = ButtonBuilder::new().bounds(Rect::new(28, 7, 40, 9)).title(" ~P~aste ").command(CM_PASTE).build();
    dialog.add(Box::new(paste_button));

    let undo_button = ButtonBuilder::new().bounds(Rect::new(2, 10, 14, 12)).title(" ~U~ndo ").command(CM_UNDO).build();
    dialog.add(Box::new(undo_button));

    let redo_button = ButtonBuilder::new().bounds(Rect::new(15, 10, 27, 12)).title(" ~R~edo ").command(CM_REDO).build();
    dialog.add(Box::new(redo_button));

    // Control buttons
    let enable_button = ButtonBuilder::new().bounds(Rect::new(42, 7, 58, 9)).title("~E~nable Edits").command(CMD_ENABLE_EDITS).build();
    dialog.add(Box::new(enable_button));

    let disable_button = ButtonBuilder::new().bounds(Rect::new(42, 10, 58, 12)).title("~D~isable Edits").command(CMD_DISABLE_EDITS).build();
    dialog.add(Box::new(disable_button));

    // Set the UI in a known state
    command_set::disable_command(CM_COPY);
    command_set::disable_command(CM_CUT);
    command_set::disable_command(CM_PASTE);
    command_set::disable_command(CM_UNDO);
    command_set::disable_command(CM_REDO);

    command_set::enable_command(CMD_ENABLE_EDITS);
    command_set::disable_command(CMD_DISABLE_EDITS);

    // Note: Close button gets initial focus because it's the first focusable child added
    // (StaticText at index 0 is not focusable, Close at index 1 is focusable)
    // When Desktop.add() is called, it will call set_focus() which calls set_initial_focus()
    // This will give focus to the first focusable child (Close button)

    dialog
}
