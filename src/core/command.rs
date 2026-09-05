// (C) 2025 - Enzo Lombardi

//! Command identifiers - constants for all UI commands and messages.
//!
//! Standard command values match Borland Turbo Vision (views.h / dialogs.h /
//! stddlg.h). Always refer to commands by these constants, never by numeric
//! value — port-specific commands occupy gaps in Borland's numbering and may
//! be renumbered.

/// Command identifiers
pub type CommandId = u16;

/// Command number ownership:
///
/// * `0..=99`: Borland's standard commands, including the `editors.h` file
///   set (`CM_NEW`, `CM_OPEN`, `CM_SAVE`, `CM_SAVE_AS`, `CM_SAVE_ALL`,
///   `CM_CLOSE_FILE`).
/// * `100..=199`: reserved for this crate's internal views and broadcasts
///   (history, file dialog, redraw, editor commands).
/// * `200..`: free for applications. Start your own numbering at `CM_USER`.
pub const CM_USER: CommandId = 200;

// Modal dialog control
pub const CM_CONTINUE: CommandId = 0; // Modal dialog continues (returned by get_end_state when no end command received)

// Standard commands (Borland views.h values)
pub const CM_QUIT: CommandId = 1; // Borland: cmQuit
pub const CM_CLOSE: CommandId = 4; // Borland: cmClose
pub const CM_ZOOM: CommandId = 5; // Borland: cmZoom
pub const CM_RESIZE: CommandId = 6; // Borland: cmResize (keyboard move/resize mode)
pub const CM_NEXT: CommandId = 7; // Cycle to next window (Borland: cmNext)
pub const CM_PREV: CommandId = 8; // Cycle to previous window (Borland: cmPrev)
pub const CM_OK: CommandId = 10; // Borland: cmOK
pub const CM_CANCEL: CommandId = 11; // Borland: cmCancel
pub const CM_YES: CommandId = 12; // Borland: cmYes
pub const CM_NO: CommandId = 13; // Borland: cmNo
pub const CM_DEFAULT: CommandId = 14; // Borland: cmDefault

// Standard edit/window commands (Borland views.h values)
pub const CM_CUT: CommandId = 20; // Borland: cmCut
pub const CM_COPY: CommandId = 21; // Borland: cmCopy
pub const CM_PASTE: CommandId = 22; // Borland: cmPaste
pub const CM_UNDO: CommandId = 23; // Borland: cmUndo
pub const CM_CLEAR: CommandId = 24; // Borland: cmClear
pub const CM_TILE: CommandId = 25; // Tile windows (Borland: cmTile)
pub const CM_CASCADE: CommandId = 26; // Cascade windows (Borland: cmCascade)

// Broadcast commands (Borland views.h values)
pub const CM_RECEIVED_FOCUS: CommandId = 50; // Borland: cmReceivedFocus
pub const CM_RELEASED_FOCUS: CommandId = 51; // Borland: cmReleasedFocus
pub const CM_COMMAND_SET_CHANGED: CommandId = 52; // Borland: cmCommandSetChanged
pub const CM_RECORD_HISTORY: CommandId = 60; // Broadcast on dialog OK — History views record their linked data (Borland: cmRecordHistory)
pub const CM_GRAB_DEFAULT: CommandId = 61; // Borland: cmGrabDefault
pub const CM_RELEASE_DEFAULT: CommandId = 62; // Borland: cmReleaseDefault
pub const CM_SCROLLBAR_CHANGED: CommandId = 57; // Borland: cmScrollBarChanged (new value in event.info, clamped to u16)
pub const CM_SELECT_WINDOW_NUM: CommandId = 55; // Borland: cmSelectWindowNum (window number in event.info)

// Standard dialog broadcast commands (Borland stddlg.h values)
pub const CM_FILE_FOCUSED: CommandId = 102; // Borland: cmFileFocused - file dialog selection changed
pub const CM_FILE_DOUBLE_CLICKED: CommandId = 103; // Borland: cmFileDoubleClicked - file double-clicked in list

// Port-specific commands (no Borland equivalent; live in the 30-49 and 63-99 gaps)
pub const CM_SCREENSHOT: CommandId = 104; // Save a PNG screenshot (also bound to Ctrl+F12)
pub const CM_REDRAW: CommandId = 105; // Full screen redraw needed (terminal resize, palette change, etc.)
pub const CM_FOCUS_LINK: CommandId = 106; // Label hotkey: focus the linked control (ViewId stored in key_code)
pub const CM_RADIO_SELECTED: CommandId = 107; // Broadcast: radio button selected (group id in event.info)
pub const CM_SHOW_HISTORY: CommandId = 108; // Command: open the history popup for a History button (history id in event.info)
pub const CM_HISTORY_SELECTED: CommandId = 109; // Broadcast: a history item was selected (history id in event.info; item is at front of HistoryManager list)
pub const CM_SHOW_DROPDOWN: CommandId = 110; // Command: open a ComboBox drop-down list (combo id in event.info)
pub const CM_MOUSE_AUTO_REPEAT: CommandId = 122; // Broadcast while a mouse button is held, so views can auto-repeat
pub const CM_IDLE_TICK: CommandId = 123; // Broadcast from Application::idle so views can run timers (hover delays, animation)

// Custom commands (user defined)

// File menu commands (moved out of 102-107 to avoid Borland's cmFileFocused range)
pub const CM_NEW: CommandId = 30;
pub const CM_OPEN: CommandId = 31;
pub const CM_SAVE: CommandId = 32;
pub const CM_SAVE_AS: CommandId = 33;
pub const CM_SAVE_ALL: CommandId = 34;
pub const CM_CLOSE_FILE: CommandId = 35;

// Edit menu commands (CUT/COPY/PASTE/UNDO are the Borland standards above)
pub const CM_REDO: CommandId = 111;
pub const CM_SELECT_ALL: CommandId = 115;
pub const CM_FIND: CommandId = 116;
pub const CM_REPLACE: CommandId = 117;
pub const CM_SEARCH_AGAIN: CommandId = 118; // Borland: cmSearchAgain (F3) - find next
pub const CM_TOGGLE_BLOCK_MODE: CommandId = 119; // Toggle the global block-edit mode (rectangular selections)

// Search menu commands
pub const CM_GOTO_LINE: CommandId = 121;

// View menu commands

// Help menu commands
pub const CM_HELP_INDEX: CommandId = 140;

// Demo commands

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_commands_stay_inside_the_reserved_band() {
        for c in [
            CM_REDRAW,
            CM_FOCUS_LINK,
            CM_RADIO_SELECTED,
            CM_SHOW_HISTORY,
            CM_HISTORY_SELECTED,
            CM_SHOW_DROPDOWN,
            CM_SCREENSHOT,
            CM_FILE_FOCUSED,
            CM_FILE_DOUBLE_CLICKED,
            CM_MOUSE_AUTO_REPEAT,
            CM_IDLE_TICK,
            CM_HELP_INDEX,
        ] {
            assert!((100..CM_USER).contains(&c), "{c} must be in 100..200");
        }
        for c in [
            CM_NEW,
            CM_OPEN,
            CM_SAVE,
            CM_SAVE_AS,
            CM_SAVE_ALL,
            CM_CLOSE_FILE,
        ] {
            assert!(c < 100, "{c} is a Borland standard command");
        }
    }
}
