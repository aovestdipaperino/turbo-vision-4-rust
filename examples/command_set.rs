// (C) 2025 - Enzo Lombardi
//! Command Set System Demo
//!
//! Demonstrates automatic button enable/disable based on application state.
//! Shows how the command set system works like Borland Turbo Vision.
//!
//! ## What This Demo Shows:
//!
//! 1. Buttons are automatically created in disabled state if their command is disabled
//! 2. The Application idle() loop broadcasts CM_COMMAND_SET_CHANGED when commands change
//! 3. Buttons automatically update their disabled state when they receive the broadcast
//!
//! ## Try It:
//!
//! - Notice that Copy, Cut, Paste, Undo, Redo buttons start DISABLED (gray)
//! - Press 'E' to enable edit commands - buttons turn GREEN
//! - Press 'D' to disable them again - buttons turn GRAY
//! - This happens automatically through the command set broadcast system!

use turbo_vision::app::Application;
use turbo_vision::core::command::{
    CommandId, CM_CANCEL, CM_COPY, CM_CUT, CM_PASTE, CM_REDO, CM_UNDO,
};
use turbo_vision::core::command_set;
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::{button::ButtonBuilder, dialog::DialogBuilder, static_text::StaticTextBuilder};

// Custom commands for this demo
const CMD_ENABLE_EDITS: CommandId = 200;
const CMD_DISABLE_EDITS: CommandId = 201;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Initially disable clipboard and undo commands
    // (Simulating empty clipboard and no history)
    command_set::disable_command(CM_COPY);
    command_set::disable_command(CM_CUT);
    command_set::disable_command(CM_PASTE);
    command_set::disable_command(CM_UNDO);
    command_set::disable_command(CM_REDO);

    // Create dialog
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(10, 4, 70, 21))
        .title("Command Set Demo - Automatic Button Enable/Disable")
        .build();

    // Instructions
    let instructions = StaticTextBuilder::new()
        .bounds(Rect::new(2, 1, 56, 6))
        .text("This demo shows AUTOMATIC button enable/disable!\n\
         \n\
         Edit commands start DISABLED (gray).\n\
         Press ~E~ to Enable edits - buttons turn GREEN!\n\
         Press ~D~ to Disable edits - buttons turn GRAY!")
        .build();
    dialog.add(Box::new(instructions));

    // Edit command buttons (will be initially disabled due to command set)
    let cut_button = ButtonBuilder::new()
        .bounds(Rect::new(2, 7, 14, 9))
        .title(" C~u~t ")
        .command(CM_CUT)
        .default(false)
        .build();
    dialog.add(Box::new(cut_button));

    let copy_button = ButtonBuilder::new()
        .bounds(Rect::new(15, 7, 27, 9))
        .title(" ~C~opy ")
        .command(CM_COPY)
        .default(false)
        .build();
    dialog.add(Box::new(copy_button));

    let paste_button = ButtonBuilder::new()
        .bounds(Rect::new(28, 7, 40, 9))
        .title(" ~P~aste ")
        .command(CM_PASTE)
        .default(false)
        .build();
    dialog.add(Box::new(paste_button));

    let undo_button = ButtonBuilder::new()
        .bounds(Rect::new(2, 10, 14, 12))
        .title(" ~U~ndo ")
        .command(CM_UNDO)
        .default(false)
        .build();
    dialog.add(Box::new(undo_button));

    let redo_button = ButtonBuilder::new()
        .bounds(Rect::new(15, 10, 27, 12))
        .title(" ~R~edo ")
        .command(CM_REDO)
        .default(false)
        .build();
    dialog.add(Box::new(redo_button));

    // Control buttons
    let enable_button = ButtonBuilder::new()
        .bounds(Rect::new(42, 7, 56, 9))
        .title("~E~nable Edits")
        .command(CMD_ENABLE_EDITS)
        .default(false)
        .build();
    dialog.add(Box::new(enable_button));

    let disable_button = ButtonBuilder::new()
        .bounds(Rect::new(42, 10, 56, 12))
        .title("~D~isable Edits")
        .command(CMD_DISABLE_EDITS)
        .default(false)
        .build();
    dialog.add(Box::new(disable_button));

    // Close button
    let close_button = ButtonBuilder::new()
        .bounds(Rect::new(22, 13, 38, 15))
        .title("  Close  ")
        .command(CM_CANCEL)
        .default(true)
        .build();
    dialog.add(Box::new(close_button));

    // Execute the dialog - use standard Application.run() with custom command handling
    // Desktop.add() automatically sets focus on the newly added dialog
    app.desktop.add(Box::new(dialog));

    // Custom event loop to handle Enable/Disable commands
    loop {
        // Use application's event handling
        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
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
                    }
                    CMD_DISABLE_EDITS => {
                        command_set::disable_command(CM_COPY);
                        command_set::disable_command(CM_CUT);
                        command_set::disable_command(CM_PASTE);
                        command_set::disable_command(CM_UNDO);
                        command_set::disable_command(CM_REDO);
                    }
                    CM_CANCEL => {
                        break;
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

    println!("\nDemo complete!");
    println!("Notice how buttons automatically updated when commands were enabled/disabled!");
    Ok(())
}
