// (C) 2025 - Enzo Lombardi

//! View state flags - constants for tracking view visibility, focus, and behavior.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

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
pub const SF_CLOSED: StateFlags = 0x1000; // Window marked for removal (Rust-specific)
pub const SF_RESIZING: StateFlags = 0x2000; // Window is being resized (Rust-specific)

// TView grow mode masks (matching Borland's gfGrowXxx flags)
// When a parent Group is resized by (dw, dh), each edge of a child whose
// corresponding grow bit is set moves by the size delta; edges without the
// bit keep their position relative to the parent's origin.

/// Grow mode flags (Borland: uchar growMode)
pub type GrowFlags = u8;

/// Left edge follows the parent's width change (Borland: gfGrowLoX)
pub const GF_GROW_LO_X: GrowFlags = 0x01;
/// Top edge follows the parent's height change (Borland: gfGrowLoY)
pub const GF_GROW_LO_Y: GrowFlags = 0x02;
/// Right edge follows the parent's width change (Borland: gfGrowHiX)
pub const GF_GROW_HI_X: GrowFlags = 0x04;
/// Bottom edge follows the parent's height change (Borland: gfGrowHiY)
pub const GF_GROW_HI_Y: GrowFlags = 0x08;
/// All edges follow the parent's size change (Borland: gfGrowAll)
pub const GF_GROW_ALL: GrowFlags = 0x0F;

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
pub const OF_VALIDATE: u16 = 0x400; // View should be validated on focus release (Borland: ofValidate)

/// Shadow size storage - initialized once at startup based on terminal cell aspect ratio
static SHADOW_SIZE_CELL: OnceLock<(i16, i16)> = OnceLock::new();

/// Get shadow size (width, height) - dynamically determined from terminal cell aspect ratio
///
/// Terminal characters are typically taller than wide (e.g., 10x16 pixels = 1.6:1 ratio).
/// This function queries the terminal for pixel dimensions and calculates the appropriate
/// shadow proportions. Falls back to (2, 1) if pixel info is unavailable.
///
/// The value is cached after first call for consistency throughout the session.
#[inline]
pub fn shadow_size() -> (i16, i16) {
    *SHADOW_SIZE_CELL.get_or_init(|| crate::terminal::Terminal::query_cell_aspect_ratio())
}

/// Legacy constant for backwards compatibility - prefer shadow_size() function
/// This is kept for code that needs a const value at compile time
pub const SHADOW_SIZE: (i16, i16) = (2, 1);

/// Shadow attribute (darkened color)
pub const SHADOW_ATTR: u8 = 0x08;

/// Shadow characters for buttons (CP437 equivalents in Unicode)
/// Original: "\xDC\xDB\xDF" = bottom edge, solid block, top edge
pub const SHADOW_BOTTOM: char = '▄'; // Lower half block
pub const SHADOW_SOLID: char = '█'; // Full block
pub const SHADOW_TOP: char = '▀'; // Upper half block

/// Global block-edit mode flag.
///
/// When on, a selection started in an editor is a rectangular (block)
/// selection instead of a stream selection. This is a global mode rather than
/// a keyboard modifier because terminals disagree on whether they deliver
/// Alt/Option with cursor keys and mouse drags.
static BLOCK_EDIT_MODE: AtomicBool = AtomicBool::new(false);

/// Is block-edit mode currently on?
#[inline]
pub fn block_edit_mode() -> bool {
    BLOCK_EDIT_MODE.load(Ordering::Relaxed)
}

/// Turn block-edit mode on or off.
#[inline]
pub fn set_block_edit_mode(on: bool) {
    BLOCK_EDIT_MODE.store(on, Ordering::Relaxed);
}

/// Flip block-edit mode and return the new value.
#[inline]
pub fn toggle_block_edit_mode() -> bool {
    !BLOCK_EDIT_MODE.fetch_xor(true, Ordering::Relaxed)
}
