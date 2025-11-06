// (C) 2025 - Enzo Lombardi

//! View state flags - constants for tracking view visibility, focus, and behavior.

/// View state flags
pub type StateFlags = u16;

// TView State masks (matching C++ Turbo Vision)
pub const SF_VISIBLE: StateFlags = 0x001;
pub const SF_CURSOR_VIS: StateFlags = 0x002;
pub const SF_CURSOR_INS: StateFlags = 0x004;
pub const SF_SHADOW: StateFlags = 0x008;
pub const SF_ACTIVE: StateFlags = 0x010;
pub const SF_SELECTED: StateFlags = 0x020;
pub const SF_FOCUSED: StateFlags = 0x040;
pub const SF_DRAGGING: StateFlags = 0x080;
pub const SF_DISABLED: StateFlags = 0x100;
pub const SF_MODAL: StateFlags = 0x200;
pub const SF_DEFAULT: StateFlags = 0x400;
pub const SF_EXPOSED: StateFlags = 0x800;
pub const SF_CLOSED: StateFlags = 0x1000;  // Window marked for removal (Rust-specific)
pub const SF_RESIZING: StateFlags = 0x2000;  // Window is being resized (Rust-specific)

// TView Option masks
pub const OF_SELECTABLE: u16 = 0x001;
pub const OF_TOP_SELECT: u16 = 0x002;
pub const OF_FIRST_CLICK: u16 = 0x004;
pub const OF_FRAMED: u16 = 0x008;
pub const OF_PRE_PROCESS: u16 = 0x010;
pub const OF_POST_PROCESS: u16 = 0x020;
pub const OF_BUFFERED: u16 = 0x040;
pub const OF_TILEABLE: u16 = 0x080;
pub const OF_CENTER_X: u16 = 0x100;
pub const OF_CENTER_Y: u16 = 0x200;
pub const OF_CENTERED: u16 = 0x300;
pub const OF_VALIDATE: u16 = 0x400;  // View should be validated on focus release (Borland: ofValidate)

/// Shadow size (width, height)
pub const SHADOW_SIZE: (i16, i16) = (2, 1);

/// Shadow attribute (darkened color)
pub const SHADOW_ATTR: u8 = 0x08;

/// Shadow characters for buttons (CP437 equivalents in Unicode)
/// Original: "\xDC\xDB\xDF" = bottom edge, solid block, top edge
pub const SHADOW_BOTTOM: char = '▄';  // Lower half block
pub const SHADOW_SOLID: char = '█';   // Full block
pub const SHADOW_TOP: char = '▀';     // Upper half block
