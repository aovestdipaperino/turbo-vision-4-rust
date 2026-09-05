// (C) 2025 - Enzo Lombardi

//! View state flags - constants for tracking view visibility, focus, and behavior.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

/// Declare a bit-flag newtype: a `Copy` wrapper over an integer with `const`
/// flags, set operations and the bit operators, so a state cannot be handed
/// to an options parameter by mistake. Used for [`State`], [`Options`],
/// [`Grow`], `MsgBox` and `ValidatorOptions`.
macro_rules! flags {
    ($(#[$outer:meta])* $vis:vis struct $name:ident: $repr:ty { $($(#[$m:meta])* const $flag:ident = $val:expr;)* }) => {
        $(#[$outer])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
        $vis struct $name($repr);
        impl $name {
            $( $(#[$m])* pub const $flag: $name = $name($val); )*
            /// No flags set.
            pub const fn empty() -> Self {
                Self(0)
            }
            /// The raw bits.
            pub const fn bits(self) -> $repr {
                self.0
            }
            /// Wrap raw bits.
            pub const fn from_bits(bits: $repr) -> Self {
                Self(bits)
            }
            /// Whether all of `other`'s flags are set.
            pub const fn contains(self, other: Self) -> bool {
                (self.0 & other.0) == other.0
            }
            /// Whether any of `other`'s flags is set.
            pub const fn intersects(self, other: Self) -> bool {
                (self.0 & other.0) != 0
            }
            /// Whether no flag is set.
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }
            pub fn insert(&mut self, other: Self) {
                self.0 |= other.0;
            }
            pub fn remove(&mut self, other: Self) {
                self.0 &= !other.0;
            }
            /// Set or clear `other`.
            pub fn set(&mut self, other: Self, on: bool) {
                if on {
                    self.insert(other)
                } else {
                    self.remove(other)
                }
            }
        }
        impl core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, r: Self) -> Self {
                Self(self.0 | r.0)
            }
        }
        impl core::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, r: Self) -> Self {
                Self(self.0 & r.0)
            }
        }
        impl core::ops::BitXor for $name {
            type Output = Self;
            fn bitxor(self, r: Self) -> Self {
                Self(self.0 ^ r.0)
            }
        }
        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, r: Self) {
                self.0 |= r.0;
            }
        }
        impl core::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, r: Self) {
                self.0 &= r.0;
            }
        }
        impl core::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self {
                Self(!self.0)
            }
        }
        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}({:#x})", stringify!($name), self.0)
            }
        }
    };
}
pub(crate) use flags;

flags! {
    /// View state flags (Borland: the `sfXxxx` masks of `TView::state`).
    pub struct State: u16 {
        const VISIBLE = 0x001;
        const CURSOR_VIS = 0x002;
        const CURSOR_INS = 0x004;
        const SHADOW = 0x008;
        const ACTIVE = 0x010;
        const SELECTED = 0x020;
        const FOCUSED = 0x040;
        const DRAGGING = 0x080;
        const DISABLED = 0x100;
        const MODAL = 0x200;
        const DEFAULT = 0x400;
        const EXPOSED = 0x800;
        /// Window marked for removal (Rust-specific)
        const CLOSED = 0x1000;
        /// Window is being resized (Rust-specific)
        const RESIZING = 0x2000;
    }
}

/// View state flags; `State` is the type, this alias keeps the 2.x name.
pub type StateFlags = State;

flags! {
    /// View option flags (Borland: the `ofXxxx` masks of `TView::options`).
    pub struct Options: u16 {
        const SELECTABLE = 0x001;
        const TOP_SELECT = 0x002;
        const FIRST_CLICK = 0x004;
        const FRAMED = 0x008;
        const PRE_PROCESS = 0x010;
        const POST_PROCESS = 0x020;
        const BUFFERED = 0x040;
        const TILEABLE = 0x080;
        const CENTER_X = 0x100;
        const CENTER_Y = 0x200;
        const CENTERED = 0x300;
        /// View should be validated on focus release (Borland: ofValidate)
        const VALIDATE = 0x400;
    }
}

flags! {
    /// Grow mode flags (Borland: `gfGrowXxx` in `TView::growMode`).
    ///
    /// When a parent Group is resized by (dw, dh), each edge of a child whose
    /// corresponding grow bit is set moves by the size delta; edges without
    /// the bit keep their position relative to the parent's origin.
    pub struct Grow: u8 {
        /// Left edge follows the parent's width change (Borland: gfGrowLoX)
        const LO_X = 0x01;
        /// Top edge follows the parent's height change (Borland: gfGrowLoY)
        const LO_Y = 0x02;
        /// Right edge follows the parent's width change (Borland: gfGrowHiX)
        const HI_X = 0x04;
        /// Bottom edge follows the parent's height change (Borland: gfGrowHiY)
        const HI_Y = 0x08;
        /// All edges follow the parent's size change (Borland: gfGrowAll)
        const ALL = 0x0F;
    }
}

/// Grow mode flags; `Grow` is the type, this alias keeps the 2.x name.
pub type GrowFlags = Grow;

// 2.x names, kept for one release.
macro_rules! deprecated_aliases {
    ($ty:ident: $($old:ident => $new:ident),* $(,)?) => {
        $(
            #[deprecated(since = "3.0.0", note = "use the typed flag constant (e.g. `State::MODAL`)")]
            pub const $old: $ty = $ty::$new;
        )*
    };
}
pub(crate) use deprecated_aliases;

deprecated_aliases! { State:
    SF_VISIBLE => VISIBLE, SF_CURSOR_VIS => CURSOR_VIS, SF_CURSOR_INS => CURSOR_INS,
    SF_SHADOW => SHADOW, SF_ACTIVE => ACTIVE, SF_SELECTED => SELECTED, SF_FOCUSED => FOCUSED,
    SF_DRAGGING => DRAGGING, SF_DISABLED => DISABLED, SF_MODAL => MODAL, SF_DEFAULT => DEFAULT,
    SF_EXPOSED => EXPOSED, SF_CLOSED => CLOSED, SF_RESIZING => RESIZING,
}
deprecated_aliases! { Options:
    OF_SELECTABLE => SELECTABLE, OF_TOP_SELECT => TOP_SELECT, OF_FIRST_CLICK => FIRST_CLICK,
    OF_FRAMED => FRAMED, OF_PRE_PROCESS => PRE_PROCESS, OF_POST_PROCESS => POST_PROCESS,
    OF_BUFFERED => BUFFERED, OF_TILEABLE => TILEABLE, OF_CENTER_X => CENTER_X,
    OF_CENTER_Y => CENTER_Y, OF_CENTERED => CENTERED, OF_VALIDATE => VALIDATE,
}
deprecated_aliases! { Grow:
    GF_GROW_LO_X => LO_X, GF_GROW_LO_Y => LO_Y, GF_GROW_HI_X => HI_X, GF_GROW_HI_Y => HI_Y,
    GF_GROW_ALL => ALL,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_flags_are_typed_and_composable() {
        let s = State::VISIBLE | State::FOCUSED;
        assert!(s.contains(State::FOCUSED));
        assert!(!s.contains(State::MODAL));
        assert_eq!(s & !State::FOCUSED, State::VISIBLE);
        assert_eq!(State::default(), State::empty());
        let mut g = Grow::empty();
        g.set(Grow::HI_X, true);
        assert!(g.intersects(Grow::ALL));
        assert_eq!(Options::CENTERED, Options::CENTER_X | Options::CENTER_Y);
    }
}
