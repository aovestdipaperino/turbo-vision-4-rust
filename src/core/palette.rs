// (C) 2025 - Enzo Lombardi

//! Color palette - 16-color palette definitions and attribute management.
//! Palette index constants for view color mapping
//!
//! These constants define the logical color indices used by each view type
//! when calling map_color(). These indices are mapped through the view's
//! palette to determine the actual color attribute.
// Color Palette
// Color definitions, attributes, and palette management matching Borland Turbo Vision
use crossterm::style::Color;

// Button palette indices (maps to CP_BUTTON)
pub const BUTTON_NORMAL: u8 = 1; // Normal button color
pub const BUTTON_DEFAULT: u8 = 2; // Default button (not focused)
pub const BUTTON_SELECTED: u8 = 3; // Selected/focused button
pub const BUTTON_DISABLED: u8 = 4; // Disabled button
pub const BUTTON_SHORTCUT: u8 = 7; // Shortcut letter color
pub const BUTTON_SHADOW: u8 = 8; // Shadow color

// InputLine palette indices (maps to CP_INPUT_LINE)
pub const INPUT_NORMAL: u8 = 1; // Normal input line
pub const INPUT_FOCUSED: u8 = 2; // Focused input line
pub const INPUT_SELECTED: u8 = 3; // Selected text
pub const INPUT_ARROWS: u8 = 4; // Arrow indicators

// ScrollBar palette indices (maps to CP_SCROLLBAR)
pub const SCROLLBAR_PAGE: u8 = 1; // Page/background area
pub const SCROLLBAR_ARROWS: u8 = 2; // Arrow buttons
pub const SCROLLBAR_INDICATOR: u8 = 3; // Scroll indicator

// ListBox palette indices (maps to CP_LISTBOX)
pub const LISTBOX_NORMAL: u8 = 1; // Normal item
pub const LISTBOX_FOCUSED: u8 = 2; // Focused list (active)
pub const LISTBOX_SELECTED: u8 = 3; // Selected item
pub const LISTBOX_DIVIDER: u8 = 4; // Divider line

// Cluster (CheckBox/RadioButton) palette indices (maps to CP_CLUSTER)
pub const CLUSTER_NORMAL: u8 = 1; // Normal item
pub const CLUSTER_FOCUSED: u8 = 2; // Focused cluster
pub const CLUSTER_SHORTCUT: u8 = 3; // Shortcut letter
pub const CLUSTER_DISABLED: u8 = 4; // Disabled item

// Label palette indices (maps to CP_LABEL)
pub const LABEL_NORMAL: u8 = 1; // Normal label text
pub const LABEL_SELECTED: u8 = 2; // Selected label
pub const LABEL_SHORTCUT: u8 = 3; // Shortcut letter

// StaticText palette indices (maps to CP_STATIC_TEXT)
pub const STATIC_TEXT_NORMAL: u8 = 1; // Normal static text

// ParamText palette indices (same as StaticText)
pub const PARAM_TEXT_NORMAL: u8 = 1; // Normal param text

// StatusLine palette indices (maps to CP_STATUSLINE)
pub const STATUSLINE_NORMAL: u8 = 1; // Normal text
pub const STATUSLINE_SHORTCUT: u8 = 2; // Shortcut letter
pub const STATUSLINE_SELECTED: u8 = 3; // Selected item
pub const STATUSLINE_SELECTED_SHORTCUT: u8 = 4; // Selected shortcut

// Frame palette indices (maps to Window/Dialog palette based on frame type)
// Borland: cFrame values use these palette indices
pub const FRAME_INACTIVE: u8 = 1; // Inactive frame (both fg and bg)
pub const FRAME_ACTIVE_BORDER: u8 = 2; // Active frame border (White on LightGray for dialog)
pub const FRAME_TITLE: u8 = 2; // Frame title (White on LightGray for dialog)
pub const FRAME_ICON: u8 = 3; // Close icon and dragging state (LightGreen on LightGray)

// Window interior palette indices
pub const WINDOW_BACKGROUND: u8 = 1; // Window interior/background color (maps differently per window type)
pub const BLUE_WINDOW_BACKGROUND: u8 = 5; // Blue window interior (Yellow on Blue)

// Editor palette indices (direct app palette indices for editor components inside windows)
// Note: Editor views return None for get_palette(), so these map directly to app palette
pub const EDITOR_NORMAL: u8 = 9; // Normal editor text (White on Blue) - app palette position 9 = 0x1F
pub const EDITOR_SELECTED: u8 = 57; // Selected text (Black on Cyan) - app palette position 57 = 0x30
pub const EDITOR_CURSOR: u8 = 57; // Cursor (Black on Cyan) - app palette position 57 = 0x30

/// 16-color palette matching Turbo Vision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TvColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

impl TvColor {
    /// Converts TvColor to crossterm Color with RGB values
    pub fn to_crossterm(self) -> Color {
        match self {
            TvColor::Black => Color::Rgb { r: 0, g: 0, b: 0 },
            TvColor::Blue => Color::Rgb { r: 0, g: 0, b: 170 },
            TvColor::Green => Color::Rgb { r: 0, g: 170, b: 0 },
            TvColor::Cyan => Color::Rgb {
                r: 0,
                g: 170,
                b: 170,
            },
            TvColor::Red => Color::Rgb { r: 170, g: 0, b: 0 },
            TvColor::Magenta => Color::Rgb {
                r: 170,
                g: 0,
                b: 170,
            },
            TvColor::Brown => Color::Rgb {
                r: 170,
                g: 85,
                b: 0,
            },
            TvColor::LightGray => Color::Rgb {
                r: 170,
                g: 170,
                b: 170,
            },
            TvColor::DarkGray => Color::Rgb {
                r: 85,
                g: 85,
                b: 85,
            },
            TvColor::LightBlue => Color::Rgb {
                r: 85,
                g: 85,
                b: 255,
            },
            TvColor::LightGreen => Color::Rgb {
                r: 85,
                g: 255,
                b: 85,
            },
            TvColor::LightCyan => Color::Rgb {
                r: 85,
                g: 255,
                b: 255,
            },
            TvColor::LightRed => Color::Rgb {
                r: 255,
                g: 85,
                b: 85,
            },
            TvColor::LightMagenta => Color::Rgb {
                r: 255,
                g: 85,
                b: 255,
            },
            TvColor::Yellow => Color::Rgb {
                r: 255,
                g: 255,
                b: 85,
            },
            TvColor::White => Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }

    /// Gets the RGB components of this color
    pub fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            TvColor::Black => (0, 0, 0),
            TvColor::Blue => (0, 0, 170),
            TvColor::Green => (0, 170, 0),
            TvColor::Cyan => (0, 170, 170),
            TvColor::Red => (170, 0, 0),
            TvColor::Magenta => (170, 0, 170),
            TvColor::Brown => (170, 85, 0),
            TvColor::LightGray => (170, 170, 170),
            TvColor::DarkGray => (85, 85, 85),
            TvColor::LightBlue => (85, 85, 255),
            TvColor::LightGreen => (85, 255, 85),
            TvColor::LightCyan => (85, 255, 255),
            TvColor::LightRed => (255, 85, 85),
            TvColor::LightMagenta => (255, 85, 255),
            TvColor::Yellow => (255, 255, 85),
            TvColor::White => (255, 255, 255),
        }
    }

    /// Creates a TvColor from RGB values by finding the closest match
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        // Find closest color in the palette
        let all_colors = [
            TvColor::Black,
            TvColor::Blue,
            TvColor::Green,
            TvColor::Cyan,
            TvColor::Red,
            TvColor::Magenta,
            TvColor::Brown,
            TvColor::LightGray,
            TvColor::DarkGray,
            TvColor::LightBlue,
            TvColor::LightGreen,
            TvColor::LightCyan,
            TvColor::LightRed,
            TvColor::LightMagenta,
            TvColor::Yellow,
            TvColor::White,
        ];

        let mut best_color = TvColor::Black;
        let mut best_distance = u32::MAX;

        for &color in &all_colors {
            let (cr, cg, cb) = color.to_rgb();
            let distance = (r as i32 - cr as i32).pow(2) as u32
                + (g as i32 - cg as i32).pow(2) as u32
                + (b as i32 - cb as i32).pow(2) as u32;
            if distance < best_distance {
                best_distance = distance;
                best_color = color;
            }
        }

        best_color
    }

    pub fn from_u8(n: u8) -> Self {
        match n & 0x0F {
            0 => TvColor::Black,
            1 => TvColor::Blue,
            2 => TvColor::Green,
            3 => TvColor::Cyan,
            4 => TvColor::Red,
            5 => TvColor::Magenta,
            6 => TvColor::Brown,
            7 => TvColor::LightGray,
            8 => TvColor::DarkGray,
            9 => TvColor::LightBlue,
            10 => TvColor::LightGreen,
            11 => TvColor::LightCyan,
            12 => TvColor::LightRed,
            13 => TvColor::LightMagenta,
            14 => TvColor::Yellow,
            15 => TvColor::White,
            _ => TvColor::LightGray,
        }
    }
}

/// Text attributes (foreground and background colors)
///
/// # Examples
///
/// ```
/// use turbo_vision::core::palette::{Attr, TvColor, colors};
///
/// // Create custom attribute
/// let attr = Attr::new(TvColor::White, TvColor::Blue);
/// assert_eq!(attr.fg, TvColor::White);
/// assert_eq!(attr.bg, TvColor::Blue);
///
/// // Use predefined colors from colors module
/// let button_attr = colors::BUTTON_NORMAL;
/// assert_eq!(button_attr.fg, TvColor::Black);
/// assert_eq!(button_attr.bg, TvColor::Green);
///
/// // Convert to/from byte representation
/// let byte = attr.to_u8();
/// let restored = Attr::from_u8(byte);
/// assert_eq!(attr, restored);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attr {
    pub fg: TvColor,
    pub bg: TvColor,
}

impl Attr {
    pub const fn new(fg: TvColor, bg: TvColor) -> Self {
        Self { fg, bg }
    }

    pub fn from_u8(byte: u8) -> Self {
        Self {
            fg: TvColor::from_u8(byte & 0x0F),
            bg: TvColor::from_u8((byte >> 4) & 0x0F),
        }
    }

    pub fn to_u8(self) -> u8 {
        (self.fg as u8) | ((self.bg as u8) << 4)
    }

    /// Swaps foreground and background colors
    /// Useful when using block characters instead of spaces for shadows
    pub fn swap(self) -> Self {
        Self {
            fg: self.bg,
            bg: self.fg,
        }
    }

    /// Creates a darkened version of this attribute (for semi-transparent shadows)
    /// Reduces RGB values by the given factor (0.0 = black, 1.0 = unchanged)
    /// Default shadow factor is 0.5 (50% darker)
    pub fn darken(&self, factor: f32) -> Self {
        let darken_color = |color: TvColor| -> TvColor {
            let (r, g, b) = color.to_rgb();
            let new_r = ((r as f32) * factor).min(255.0) as u8;
            let new_g = ((g as f32) * factor).min(255.0) as u8;
            let new_b = ((b as f32) * factor).min(255.0) as u8;
            TvColor::from_rgb(new_r, new_g, new_b)
        };

        Self {
            fg: darken_color(self.fg),
            bg: darken_color(self.bg),
        }
    }
}

/// Standard color pairs for UI elements
pub mod colors {
    use super::*;

    pub const NORMAL: Attr = Attr::new(TvColor::LightGray, TvColor::Blue);
    pub const HIGHLIGHTED: Attr = Attr::new(TvColor::Yellow, TvColor::Blue);
    pub const SELECTED: Attr = Attr::new(TvColor::White, TvColor::Cyan);
    pub const DISABLED: Attr = Attr::new(TvColor::DarkGray, TvColor::Blue);

    pub const MENU_NORMAL: Attr = Attr::new(TvColor::Black, TvColor::LightGray);
    pub const MENU_SELECTED: Attr = Attr::new(TvColor::White, TvColor::Green);
    pub const MENU_DISABLED: Attr = Attr::new(TvColor::DarkGray, TvColor::LightGray);
    pub const MENU_SHORTCUT: Attr = Attr::new(TvColor::Red, TvColor::LightGray);

    pub const DIALOG_NORMAL: Attr = Attr::new(TvColor::Black, TvColor::LightGray); // cpDialog[0] = 0x70 interior
    pub const DIALOG_FRAME: Attr = Attr::new(TvColor::White, TvColor::LightGray); // cpDialog[1] = 0x7F
    pub const DIALOG_FRAME_ACTIVE: Attr = Attr::new(TvColor::White, TvColor::LightGray); // cpDialog[1] = 0x7F
    pub const DIALOG_TITLE: Attr = Attr::new(TvColor::White, TvColor::LightGray); // cpDialog[1] = 0x7F
    pub const DIALOG_SHORTCUT: Attr = Attr::new(TvColor::Red, TvColor::LightGray); // Shortcut letters in dialogs

    pub const BUTTON_NORMAL: Attr = Attr::new(TvColor::Black, TvColor::Green); // Inactive but focusable
    pub const BUTTON_DEFAULT: Attr = Attr::new(TvColor::LightGreen, TvColor::Green); // Default but not focused
    pub const BUTTON_SELECTED: Attr = Attr::new(TvColor::White, TvColor::Green); // Focused
    pub const BUTTON_DISABLED: Attr = Attr::new(TvColor::DarkGray, TvColor::Green); // Disabled (not implemented yet)
    pub const BUTTON_SHORTCUT: Attr = Attr::new(TvColor::Yellow, TvColor::Green); // Shortcut letters
    pub const BUTTON_SHADOW: Attr = Attr::new(TvColor::LightGray, TvColor::DarkGray);

    pub const STATUS_NORMAL: Attr = Attr::new(TvColor::Black, TvColor::LightGray);
    pub const STATUS_SHORTCUT: Attr = Attr::new(TvColor::Red, TvColor::LightGray);
    pub const STATUS_SELECTED: Attr = Attr::new(TvColor::White, TvColor::Green);
    pub const STATUS_SELECTED_SHORTCUT: Attr = Attr::new(TvColor::Yellow, TvColor::Green);

    // InputLine colors - matching actual C++ rendering (see colors.png)
    // Focused state uses Yellow on Blue (clearly visible in screenshot)
    // Both states use same color per C++ cpInputLine behavior
    pub const INPUT_NORMAL: Attr = Attr::new(TvColor::Yellow, TvColor::Blue); // Same as focused
    pub const INPUT_FOCUSED: Attr = Attr::new(TvColor::Yellow, TvColor::Blue); // SAME as unfocused!
    pub const INPUT_SELECTED: Attr = Attr::new(TvColor::Cyan, TvColor::Cyan); // cpDialog[20] = 0x33
    pub const INPUT_ARROWS: Attr = Attr::new(TvColor::Red, TvColor::Cyan); // cpDialog[21] = 0x34

    // Editor colors (matching original Turbo Vision)
    pub const EDITOR_NORMAL: Attr = Attr::new(TvColor::White, TvColor::Blue);
    pub const EDITOR_SELECTED: Attr = Attr::new(TvColor::Black, TvColor::Cyan);

    pub const LISTBOX_NORMAL: Attr = Attr::new(TvColor::Black, TvColor::LightGray);
    pub const LISTBOX_FOCUSED: Attr = Attr::new(TvColor::Black, TvColor::White);
    pub const LISTBOX_SELECTED: Attr = Attr::new(TvColor::White, TvColor::Blue);
    pub const LISTBOX_SELECTED_FOCUSED: Attr = Attr::new(TvColor::White, TvColor::Cyan);

    pub const SCROLLBAR_PAGE: Attr = Attr::new(TvColor::DarkGray, TvColor::LightGray);
    pub const SCROLLBAR_INDICATOR: Attr = Attr::new(TvColor::Blue, TvColor::LightGray);
    pub const SCROLLBAR_ARROW: Attr = Attr::new(TvColor::Black, TvColor::LightGray);

    pub const SCROLLER_NORMAL: Attr = Attr::new(TvColor::Black, TvColor::LightGray);
    pub const SCROLLER_SELECTED: Attr = Attr::new(TvColor::White, TvColor::Blue);

    pub const DESKTOP: Attr = Attr::new(TvColor::LightGray, TvColor::DarkGray);

    // Syntax highlighting colors (editor-specific, all use Blue background)
    pub const SYNTAX_NORMAL: Attr = Attr::new(TvColor::LightGray, TvColor::Blue);
    pub const SYNTAX_KEYWORD: Attr = Attr::new(TvColor::Yellow, TvColor::Blue);
    pub const SYNTAX_STRING: Attr = Attr::new(TvColor::LightRed, TvColor::Blue);
    pub const SYNTAX_COMMENT: Attr = Attr::new(TvColor::LightCyan, TvColor::Blue);
    pub const SYNTAX_NUMBER: Attr = Attr::new(TvColor::LightMagenta, TvColor::Blue);
    pub const SYNTAX_OPERATOR: Attr = Attr::new(TvColor::White, TvColor::Blue);
    pub const SYNTAX_IDENTIFIER: Attr = Attr::new(TvColor::LightGray, TvColor::Blue);
    pub const SYNTAX_TYPE: Attr = Attr::new(TvColor::LightGreen, TvColor::Blue);
    pub const SYNTAX_PREPROCESSOR: Attr = Attr::new(TvColor::LightCyan, TvColor::Blue);
    pub const SYNTAX_FUNCTION: Attr = Attr::new(TvColor::Cyan, TvColor::Blue);
    pub const SYNTAX_SPECIAL: Attr = Attr::new(TvColor::White, TvColor::Blue);

    // Help system colors
    pub const HELP_NORMAL: Attr = Attr::new(TvColor::Black, TvColor::LightGray);
    pub const HELP_FOCUSED: Attr = Attr::new(TvColor::Black, TvColor::White);
}

/// Palette - array of color remappings for the Borland indirect palette system
///
/// Each view has an optional palette that maps logical color indices to parent color indices.
/// When resolving a color, the system walks up the owner chain, remapping through each palette
/// until reaching the Application which has the actual color attributes.
#[derive(Debug, Clone)]
pub struct Palette {
    data: Vec<u8>,
}

impl Palette {
    /// Create a new empty palette
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a palette from a slice of color indices
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Get a color index from the palette (1-based indexing like Borland)
    /// Returns 0 (error color) if index is out of bounds
    pub fn get(&self, index: usize) -> u8 {
        if index == 0 || index > self.data.len() {
            0
        } else {
            self.data[index - 1]
        }
    }

    /// Get the length of the palette
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the palette is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard palette definitions matching Borland Turbo Vision
pub mod palettes {
    use std::cell::RefCell;

    thread_local! {
        /// Custom application palette that overrides CP_APP_COLOR if set
        /// This allows runtime palette customization for theming
        static CUSTOM_APP_PALETTE: RefCell<Option<Vec<u8>>> = RefCell::new(None);
    }

    /// Set a custom application palette
    /// Pass None to reset to default CP_APP_COLOR
    pub fn set_custom_palette(palette: Option<Vec<u8>>) {
        CUSTOM_APP_PALETTE.with(|p| {
            *p.borrow_mut() = palette;
        });
    }

    /// Get the current application palette (custom or default)
    pub fn get_app_palette() -> Vec<u8> {
        CUSTOM_APP_PALETTE.with(|p| {
            if let Some(custom) = p.borrow().as_ref() {
                custom.clone()
            } else {
                CP_APP_COLOR.to_vec()
            }
        })
    }

    // Application color palette - contains actual color attributes (1-indexed)
    // This is the root palette that contains real Attr values encoded as u8
    // From Borland cpColor (program.h):
    //   Palette layout:
    //     1      = TBackground
    //     2-7    = TMenuView and TStatusLine
    //     8-15   = TWindow(Blue)
    //     16-23  = TWindow(Cyan)
    //     24-31  = TWindow(Gray)
    //     32-63  = TDialog
    #[rustfmt::skip]
    pub const CP_APP_COLOR: &[u8] = &[
        0x71, 0x70, 0x78, 0x74, 0x20, 0x28, 0x24, 0x17, // 1-8: Desktop colors
        0x1F, 0x1A, 0x31, 0x31, 0x1E, 0x71, 0x00,       // 9-15: Menu colors
        0x37, 0x3F, 0x3A, 0x13, 0x13, 0x3E, 0x21, 0x00, // 16-23: Cyan Window
        0x70, 0x7F, 0x7A, 0x13, 0x13, 0x70, 0x7F, 0x00, // 24-31: Gray Window
        0x70, 0x7F, 0x7A, 0x13, 0x13, 0x70, 0x70, 0x7F, // 32-39: Dialog (Frame, StaticText, Label, etc.)
        0x7E, 0x20, 0x2B, 0x2F, 0x78, 0x2E, 0x70, 0x30, // 40-47: Dialog (controls)
        0x3F, 0x3E, 0x1F, 0x2F, 0x1A, 0x20, 0x72, 0x31, // 48-55: Dialog (InputLine, Button, etc.)
        0x31, 0x30, 0x2F, 0x3E, 0x31, 0x13, 0x38, 0x00, // 56-63: Dialog (remaining)
    ];

    // Window palettes - map window color indices to app palette
    // BlueWindow: indices 8-15
    #[rustfmt::skip]
    pub const CP_BLUE_WINDOW: &[u8] = &[
        8, 9, 10, 11, 12, 13, 14, 15,  // Maps to app palette 8-15
    ];

    // CyanWindow: indices 16-23
    #[rustfmt::skip]
    pub const CP_CYAN_WINDOW: &[u8] = &[
        16, 17, 18, 19, 20, 21, 22, 23,  // Maps to app palette 16-23
    ];

    // GrayWindow: indices 24-31
    #[rustfmt::skip]
    pub const CP_GRAY_WINDOW: &[u8] = &[
        24, 25, 26, 27, 28, 29, 30, 31,  // Maps to app palette 24-31
    ];

    // Gray dialog palette - maps dialog color indices to app palette
    #[rustfmt::skip]
    pub const CP_GRAY_DIALOG: &[u8] = &[
        32, 33, 34, 35, 36, 37, 38, 39, 40, 41,  // 1-10
        42, 43, 44, 45, 46, 47, 48, 49, 50, 51,  // 11-20
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61,  // 21-30
        62, 63,                                   // 31-32
    ];

    // Blue dialog palette - maps dialog color indices to app palette
    #[rustfmt::skip]
    pub const CP_BLUE_DIALOG: &[u8] = &[
        16, 17, 18, 19, 20, 21, 22, 23, 24, 25,  // 1-10
        26, 27, 28, 29, 30, 31, 32, 33, 34, 35,  // 11-20
        36, 37, 38, 39, 40, 41, 42, 43, 44, 45,  // 21-30
        46, 47,                                   // 31-32
    ];

    // Button palette - from Borland cpButton "\x0A\x0B\x0C\x0D\x0E\x0E\x0E\x0F"
    #[rustfmt::skip]
    pub const CP_BUTTON: &[u8] = &[
        10, 11, 12, 13, 14, 14, 14, 15,  // 1-8: Matches Borland exactly
    ];

    // StaticText palette - from Borland cpStaticText "\x06"
    #[rustfmt::skip]
    pub const CP_STATIC_TEXT: &[u8] = &[
        6,  // 1: Normal text color
    ];

    // InputLine palette - from Borland cpInputLine "\x13\x13\x14\x15" (19, 19, 20, 21)
    // These are dialog-relative indices that should map to dialog palette positions
    #[rustfmt::skip]
    pub const CP_INPUT_LINE: &[u8] = &[
        19, 19, 20, 21,  // 1-4: Normal, focused, selected, arrows (from Borland)
    ];

    // Label palette - from Borland cpLabel "\x07\x08\x09\x09\x0D\x0D"
    // Used with getColor(0x0301) for normal, getColor(0x0402) for focused, getColor(0x0605) for disabled
    #[rustfmt::skip]
    pub const CP_LABEL: &[u8] = &[
        7, 8, 9, 9, 13, 13,  // 1-6: Normal fg/bg, Light fg/bg, Disabled fg/bg
    ];

    // ListBox palette
    #[rustfmt::skip]
    pub const CP_LISTBOX: &[u8] = &[
        26, 26, 27, 28,  // 1-4: Normal, focused, selected, divider
    ];

    // ScrollBar palette
    #[rustfmt::skip]
    pub const CP_SCROLLBAR: &[u8] = &[
        4, 5, 5,  // 1-3: Page, arrows, indicator
    ];

    // Scroller palette (base class for scrollable views)
    // Borland: cpScroller = "\x06\x07" (6, 7)
    // Used by TScroller and derived classes like TTextDevice/TTerminal
    #[rustfmt::skip]
    pub const CP_SCROLLER: &[u8] = &[
        6, 7,  // 1-2: Normal scrollable area, Selected item
    ];

    // Cluster palette (CheckBox, RadioButton)
    #[rustfmt::skip]
    pub const CP_CLUSTER: &[u8] = &[
        16, 17, 18, 19,  // 1-4: Normal, focused, shortcut, disabled
    ];

    // StatusLine palette
    #[rustfmt::skip]
    pub const CP_STATUSLINE: &[u8] = &[
        2, 4, 45, 41,  // 1-4: Normal, shortcut, selected, selected_shortcut
    ];

    // MenuBar palette (gray background, matching desktop colors)
    #[rustfmt::skip]
    pub const CP_MENU_BAR: &[u8] = &[
        2, 5, 3, 4,  // 1-4: Normal (Black/LightGray), Selected (Black/Green), Disabled (DarkGray/LightGray), Shortcut (Red/LightGray)
    ];

    // Memo palette (multi-line text editor)
    // Borland: cpMemo = "\x1A\x1B" (26, 27)
    // Maps to window interior colors for editor-like behavior
    #[rustfmt::skip]
    pub const CP_MEMO: &[u8] = &[
        26, 27,  // 1-2: Normal text, Selected text
    ];

    // Indicator palette (position/status indicator in editors)
    // Borland: cpIndicator = "\x02\x03" (2, 3)
    // Uses app-level status colors
    #[rustfmt::skip]
    pub const CP_INDICATOR: &[u8] = &[
        2, 3,  // 1-2: Normal indicator, Modified/active indicator
    ];

    // Help Viewer palette (help text display)
    // Borland: cHelpViewer = "\x06\x07\x08" (6, 7, 8)
    #[rustfmt::skip]
    pub const CP_HELP_VIEWER: &[u8] = &[
        6, 7, 8,  // 1-3: Normal text, Focused text, Cross-reference links
    ];
}
