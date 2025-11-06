// (C) 2025 - Enzo Lombardi

//! Command identifiers - constants for all UI commands and messages.

/// Command identifiers
pub type CommandId = u16;

// Standard commands
pub const CM_QUIT: CommandId = 24;
pub const CM_CLOSE: CommandId = 25;
pub const CM_ZOOM: CommandId = 26;
pub const CM_NEXT: CommandId = 27;  // Cycle to next window (Borland: cmNext)
pub const CM_PREV: CommandId = 28;  // Cycle to previous window (Borland: cmPrev)
pub const CM_TILE: CommandId = 29;  // Tile windows (Borland: cmTile)
pub const CM_CASCADE: CommandId = 30;  // Cascade windows (Borland: cmCascade)
pub const CM_OK: CommandId = 10;
pub const CM_CANCEL: CommandId = 11;
pub const CM_YES: CommandId = 12;
pub const CM_NO: CommandId = 13;
pub const CM_DEFAULT: CommandId = 14;

// Broadcast commands
pub const CM_COMMAND_SET_CHANGED: CommandId = 52;  // Borland: cmCommandSetChanged
pub const CM_RECEIVED_FOCUS: CommandId = 50;       // Borland: cmReceivedFocus
pub const CM_RELEASED_FOCUS: CommandId = 51;       // Borland: cmReleasedFocus
pub const CM_GRAB_DEFAULT: CommandId = 62;         // Borland: cmGrabDefault
pub const CM_RELEASE_DEFAULT: CommandId = 63;      // Borland: cmReleaseDefault
pub const CM_FILE_FOCUSED: CommandId = 64;         // Borland: cmFileFocused - file dialog selection changed
pub const CM_FILE_DOUBLE_CLICKED: CommandId = 65;  // Borland: cmFileDoubleClicked - file double-clicked in list

// Custom commands (user defined)
pub const CM_ABOUT: CommandId = 100;
pub const CM_BIRTHDATE: CommandId = 101;
pub const CM_TEXT_VIEWER: CommandId = 108;
pub const CM_CONTROLS_DEMO: CommandId = 109;

// File menu commands
pub const CM_NEW: CommandId = 102;
pub const CM_OPEN: CommandId = 103;
pub const CM_SAVE: CommandId = 104;
pub const CM_SAVE_AS: CommandId = 105;
pub const CM_SAVE_ALL: CommandId = 106;
pub const CM_CLOSE_FILE: CommandId = 107;

// Edit menu commands
pub const CM_UNDO: CommandId = 110;
pub const CM_REDO: CommandId = 111;
pub const CM_CUT: CommandId = 112;
pub const CM_COPY: CommandId = 113;
pub const CM_PASTE: CommandId = 114;
pub const CM_SELECT_ALL: CommandId = 115;
pub const CM_FIND: CommandId = 116;
pub const CM_REPLACE: CommandId = 117;
pub const CM_SEARCH_AGAIN: CommandId = 118;  // Borland: cmSearchAgain (F3) - find next

// Search menu commands
pub const CM_FIND_IN_FILES: CommandId = 120;
pub const CM_GOTO_LINE: CommandId = 121;

// View menu commands
pub const CM_ZOOM_IN: CommandId = 130;
pub const CM_ZOOM_OUT: CommandId = 131;
pub const CM_TOGGLE_SIDEBAR: CommandId = 132;
pub const CM_TOGGLE_STATUSBAR: CommandId = 133;

// Help menu commands
pub const CM_HELP_INDEX: CommandId = 140;
pub const CM_KEYBOARD_REF: CommandId = 141;

// Demo commands
pub const CM_LISTBOX_DEMO: CommandId = 150;
pub const CM_LISTBOX_SELECT: CommandId = 151;
pub const CM_MEMO_DEMO: CommandId = 152;
