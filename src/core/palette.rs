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
pub const CLUSTER_SHORTCUT: u8 = 3; // Shortcut letter (normal)
pub const CLUSTER_SHORTCUT_SELECTED: u8 = 4; // Shortcut letter (selected)
pub const CLUSTER_DISABLED: u8 = 5; // Disabled item (Borland cpCluster position 5)

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
pub const STATUSLINE_DISABLED: u8 = 5; // Disabled item (command_set::command_enabled == false)

// Frame palette indices (maps to Window/Dialog palette based on frame type)
// Borland: cFrame values use these palette indices
pub const FRAME_INACTIVE: u8 = 1; // Inactive frame (both fg and bg)
pub const FRAME_ACTIVE_BORDER: u8 = 2; // Active frame border (White on LightGray for dialog)
pub const FRAME_TITLE: u8 = 2; // Frame title (White on LightGray for dialog)
pub const FRAME_ICON: u8 = 3; // Close icon and dragging state (LightGreen on LightGray)

// Window interior palette indices
pub const WINDOW_BACKGROUND: u8 = 1; // Window interior/background color (maps differently per window type)
pub const BLUE_WINDOW_BACKGROUND: u8 = 5; // Blue window interior (Yellow on Blue)

// EditorWindow palette indices (cpEditor palette-relative indices)
// EditorWindow now uses CP_EDITOR palette [6, 7] with proper parent-child hierarchy
// Index 1 → CP_EDITOR[0] = 6 → App palette position 6 (Normal text)
// Index 2 → CP_EDITOR[1] = 7 → App palette position 7 (Selected text)
pub const EDITOR_NORMAL: u8 = 1; // Normal editor text - cpEditor position 1 → app palette 6
pub const EDITOR_SELECTED: u8 = 2; // Selected text - cpEditor position 2 → app palette 7
pub const EDITOR_CURSOR: u8 = 2; // Cursor - same as selected

// Syntax highlighting palette indices (editor-relative, map through CP_EDITOR → window palette → app)
// Index 1-2 are normal/selected text. Indices 3-13 are syntax token colors.
pub const SYNTAX_NORMAL_IDX: u8 = 3;
pub const SYNTAX_KEYWORD_IDX: u8 = 4;
pub const SYNTAX_STRING_IDX: u8 = 5;
pub const SYNTAX_COMMENT_IDX: u8 = 6;
pub const SYNTAX_NUMBER_IDX: u8 = 7;
pub const SYNTAX_OPERATOR_IDX: u8 = 8;
pub const SYNTAX_IDENTIFIER_IDX: u8 = 9;
pub const SYNTAX_TYPE_IDX: u8 = 10;
pub const SYNTAX_PREPROCESSOR_IDX: u8 = 11;
pub const SYNTAX_FUNCTION_IDX: u8 = 12;
pub const SYNTAX_SPECIAL_IDX: u8 = 13;

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
    /// True-color RGB value, bypasses CGA palette mapping.
    Rgb {
        r: u8,
        g: u8,
        b: u8,
    } = 16,
}

impl TvColor {
    /// Returns the CGA palette index (0-15). Rgb returns 0.
    pub fn to_index(self) -> u8 {
        match self {
            TvColor::Black => 0,
            TvColor::Blue => 1,
            TvColor::Green => 2,
            TvColor::Cyan => 3,
            TvColor::Red => 4,
            TvColor::Magenta => 5,
            TvColor::Brown => 6,
            TvColor::LightGray => 7,
            TvColor::DarkGray => 8,
            TvColor::LightBlue => 9,
            TvColor::LightGreen => 10,
            TvColor::LightCyan => 11,
            TvColor::LightRed => 12,
            TvColor::LightMagenta => 13,
            TvColor::Yellow => 14,
            TvColor::White => 15,
            TvColor::Rgb { .. } => 0,
        }
    }

    /// Converts TvColor to ANSI 256-color code.
    ///
    /// Turbo Vision uses CGA/DOS color ordering which differs from ANSI:
    /// - TV: 0=Black, 1=Blue, 2=Green, 3=Cyan, 4=Red, 5=Magenta, 6=Brown, 7=LightGray
    /// - ANSI: 0=Black, 1=Red, 2=Green, 3=Yellow, 4=Blue, 5=Magenta, 6=Cyan, 7=White
    ///
    /// This method maps from TV color indices to ANSI color codes.
    pub fn to_ansi_code(self) -> u8 {
        // Map Turbo Vision color order to ANSI color order
        match self {
            TvColor::Black => 0,
            TvColor::Blue => 4, // TV 1 -> ANSI 4
            TvColor::Green => 2,
            TvColor::Cyan => 6, // TV 3 -> ANSI 6
            TvColor::Red => 1,  // TV 4 -> ANSI 1
            TvColor::Magenta => 5,
            TvColor::Brown => 3, // TV 6 -> ANSI 3 (yellow/brown)
            TvColor::LightGray => 7,
            TvColor::DarkGray => 8,
            TvColor::LightBlue => 12, // TV 9 -> ANSI 12
            TvColor::LightGreen => 10,
            TvColor::LightCyan => 14, // TV 11 -> ANSI 14
            TvColor::LightRed => 9,   // TV 12 -> ANSI 9
            TvColor::LightMagenta => 13,
            TvColor::Yellow => 11, // TV 14 -> ANSI 11
            TvColor::White => 15,
            TvColor::Rgb { .. } => 0, // RGB colors don't map to ANSI indices
        }
    }

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
            TvColor::Rgb { r, g, b } => Color::Rgb { r, g, b },
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
            TvColor::Rgb { r, g, b } => (r, g, b),
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

/// Text style flags (rendered as SGR attributes). Independent of color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Style(u8);

impl Style {
    pub const BOLD: Style = Style(1 << 0);
    pub const ITALIC: Style = Style(1 << 1);
    pub const UNDERLINE: Style = Style(1 << 2);
    pub const REVERSE: Style = Style(1 << 3);
    pub const DIM: Style = Style(1 << 4);
    pub const STRIKETHROUGH: Style = Style(1 << 5);

    /// The empty style (no flags set).
    pub const fn empty() -> Style {
        Style(0)
    }

    /// Raw bit representation.
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// True when no flags are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// True when every flag in `other` is set in `self`.
    pub const fn contains(self, other: Style) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Set every flag in `other`.
    pub fn insert(&mut self, other: Style) {
        self.0 |= other.0;
    }

    /// Clear every flag in `other`.
    pub fn remove(&mut self, other: Style) {
        self.0 &= !other.0;
    }
}

impl core::ops::BitOr for Style {
    type Output = Style;
    fn bitor(self, rhs: Style) -> Style {
        Style(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for Style {
    fn bitor_assign(&mut self, rhs: Style) {
        self.0 |= rhs.0;
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
    pub style: Style,
}

impl Attr {
    pub const fn new(fg: TvColor, bg: TvColor) -> Self {
        Self {
            fg,
            bg,
            style: Style::empty(),
        }
    }

    /// Returns a copy with the given style flags set (replacing any existing flags).
    pub const fn with_style(self, style: Style) -> Self {
        Self {
            fg: self.fg,
            bg: self.bg,
            style,
        }
    }

    /// Returns a copy with an additional style flag set.
    const fn add_style(self, flag: Style) -> Self {
        Self {
            fg: self.fg,
            bg: self.bg,
            style: Style(self.style.bits() | flag.bits()),
        }
    }

    pub const fn bold(self) -> Self {
        self.add_style(Style::BOLD)
    }
    pub const fn italic(self) -> Self {
        self.add_style(Style::ITALIC)
    }
    pub const fn underline(self) -> Self {
        self.add_style(Style::UNDERLINE)
    }
    pub const fn reverse(self) -> Self {
        self.add_style(Style::REVERSE)
    }
    pub const fn dim(self) -> Self {
        self.add_style(Style::DIM)
    }
    pub const fn strikethrough(self) -> Self {
        self.add_style(Style::STRIKETHROUGH)
    }

    pub fn from_u8(byte: u8) -> Self {
        Self {
            fg: TvColor::from_u8(byte & 0x0F),
            bg: TvColor::from_u8((byte >> 4) & 0x0F),
            style: Style::empty(),
        }
    }

    pub fn to_u8(self) -> u8 {
        self.fg.to_index() | (self.bg.to_index() << 4)
    }

    /// Swaps foreground and background colors
    /// Useful when using block characters instead of spaces for shadows
    pub fn swap(self) -> Self {
        Self {
            fg: self.bg,
            bg: self.fg,
            style: self.style,
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
            style: self.style,
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

    // EditorWindow colors (matching original Turbo Vision)
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
    pub const SYNTAX_COMMENT: Attr = Attr::new(TvColor::White, TvColor::Blue);
    pub const SYNTAX_NUMBER: Attr = Attr::new(TvColor::LightMagenta, TvColor::Blue);
    pub const SYNTAX_OPERATOR: Attr = Attr::new(TvColor::White, TvColor::Blue);
    pub const SYNTAX_IDENTIFIER: Attr = Attr::new(TvColor::LightGray, TvColor::Blue);
    pub const SYNTAX_TYPE: Attr = Attr::new(TvColor::LightGreen, TvColor::Blue);
    pub const SYNTAX_PREPROCESSOR: Attr = Attr::new(TvColor::White, TvColor::Blue);
    pub const SYNTAX_FUNCTION: Attr = Attr::new(TvColor::White, TvColor::Blue);
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
    //     64-74  = Syntax highlighting (Blue bg)
    //     75-85  = Syntax highlighting (Cyan bg)
    //     86-96  = Syntax highlighting (Gray bg)
    //     97-104 = Black Window (LogWindow)
    #[rustfmt::skip]
    pub const CP_APP_COLOR: &[u8] = &[
        0x71, 0x70, 0x78, 0x74, 0x20, 0x28, 0x24, 0x17, // 1-8: Desktop colors
        0x1F, 0x1A, 0x31, 0x31, 0x1E, 0x71, 0x1F,       // 9-15: Menu colors (15: Borland 0x1F)
        // 16-23: Cyan Window
        // Note: Index 16 changed from Borland's 0x37 (light gray on cyan) to 0x30 (black on cyan)
        // for better readability on modern terminals where light gray on cyan has poor contrast
        // Index 23 restored to Borland's 0x3F (was accidentally zeroed).
        0x30, 0x3F, 0x3A, 0x13, 0x13, 0x3E, 0x21, 0x3F,
        // 24-31: Gray Window (31 restored to Borland's 0x7E; was accidentally zeroed)
        0x70, 0x7F, 0x7A, 0x13, 0x13, 0x70, 0x7F, 0x7E,
        0x70, 0x7F, 0x7A, 0x13, 0x13, 0x70, 0x70, 0x7F, // 32-39: Dialog (Frame, StaticText, Label, etc.)
        0x7E, 0x20, 0x2B, 0x2F, 0x78, 0x2E, 0x70, 0x30, // 40-47: Dialog (controls)
        0x3F, 0x3E, 0x1F, 0x2F, 0x1A, 0x20, 0x72, 0x31, // 48-55: Dialog (InputLine, Button, etc.)
        0x31, 0x30, 0x2F, 0x3E, 0x31, 0x13, 0x38, 0x00, // 56-63: Dialog (remaining)
        // 64-74: Syntax highlighting - Blue background
        // Normal, Keyword, String, Comment, Number, Operator, Identifier, Type, Preprocessor, Function, Special
        0x17, 0x1E, 0x1C, 0x1F, 0x1D, 0x1F, 0x17, 0x1A, 0x1F, 0x1B, 0x1F,
        // 75-85: Syntax highlighting - Cyan background
        0x37, 0x3E, 0x3C, 0x3F, 0x3D, 0x3F, 0x37, 0x3A, 0x3F, 0x3B, 0x3F,
        // 86-96: Syntax highlighting - Gray (LightGray) background
        0x70, 0x7E, 0x7C, 0x78, 0x7D, 0x70, 0x70, 0x72, 0x78, 0x71, 0x74,
        // 97-104: Black Window (for LogWindow and similar dark-themed windows)
        // Same layout as Blue/Cyan/Gray window entries 1-8:
        // 1: Frame passive, 2: Frame active, 3: Frame icon, 4: ScrollBar page
        // 5: ScrollBar arrows, 6: Normal text, 7: Selected text, 8: Reserved
        0x07, 0x0F, 0x0A, 0x08, 0x08, 0x0F, 0x07, 0x00,
        // 105-136: Blue Dialog (Borland cpAppColor 64-95, moved here because this
        // port repurposed app entries 64-96 for syntax-highlighting colors).
        // CP_BLUE_DIALOG maps its 32 entries into this region.
        // Note: entry 136 (Borland 95) is 0x00 in Borland too (reserved).
        0x17, 0x1F, 0x1A, 0x71, 0x71, 0x1E, 0x17, 0x1F, // 105-112: Frame/StaticText/Label
        0x1E, 0x20, 0x2B, 0x2F, 0x78, 0x2E, 0x10, 0x30, // 113-120: Label/Button/Cluster
        0x3F, 0x3E, 0x70, 0x2F, 0x7A, 0x20, 0x12, 0x31, // 121-128: InputLine/History/List
        0x31, 0x30, 0x2F, 0x3E, 0x31, 0x13, 0x38, 0x00, // 129-136: InfoPane etc. (136 reserved)
    ];

    // Window palettes - map window color indices to app palette
    // BlueWindow: indices 8-15
    #[rustfmt::skip]
    pub const CP_BLUE_WINDOW: &[u8] = &[
        8, 9, 10, 11, 12, 13, 14, 15,                   // 1-8: Original window entries
        64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74,     // 9-19: Syntax colors (blue bg)
    ];

    // CyanWindow: indices 16-23
    #[rustfmt::skip]
    pub const CP_CYAN_WINDOW: &[u8] = &[
        16, 17, 18, 19, 20, 21, 22, 23,                 // 1-8: Original window entries
        75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85,     // 9-19: Syntax colors (cyan bg)
    ];

    // GrayWindow: indices 24-31
    #[rustfmt::skip]
    pub const CP_GRAY_WINDOW: &[u8] = &[
        24, 25, 26, 27, 28, 29, 30, 31,                 // 1-8: Original window entries
        86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96,     // 9-19: Syntax colors (gray bg)
    ];

    // Gray dialog palette - maps dialog color indices to app palette
    #[rustfmt::skip]
    pub const CP_GRAY_DIALOG: &[u8] = &[
        32, 33, 34, 35, 36, 37, 38, 39, 40, 41,  // 1-10
        42, 43, 44, 45, 46, 47, 48, 49, 50, 51,  // 11-20
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61,  // 21-30
        62, 63,                                   // 31-32
    ];

    // Blue dialog palette - maps dialog color indices to app palette.
    // Borland cpBlueDialog maps to app entries 64-95, but this port uses 64-96
    // for syntax-highlighting colors, so the Borland blue-dialog attribute block
    // was appended to CP_APP_COLOR at positions 105-136 instead (same values,
    // relocated). This keeps blue dialogs rendering with blue backgrounds without
    // disturbing the syntax-color region.
    #[rustfmt::skip]
    pub const CP_BLUE_DIALOG: &[u8] = &[
        105, 106, 107, 108, 109, 110, 111, 112, 113, 114,  // 1-10
        115, 116, 117, 118, 119, 120, 121, 122, 123, 124,  // 11-20
        125, 126, 127, 128, 129, 130, 131, 132, 133, 134,  // 21-30
        135, 136,                                           // 31-32
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
    // Borland cpCluster = "\x10\x11\x12\x12\x1f" (16, 17, 18, 18, 31)
    #[rustfmt::skip]
    pub const CP_CLUSTER: &[u8] = &[
        16, 17, 18, 18, 31,  // 1-5: Normal, selected, shortcut, shortcut selected, disabled
    ];

    // StatusLine palette
    #[rustfmt::skip]
    pub const CP_STATUSLINE: &[u8] = &[
        2, 4, 45, 41,  // 1-4: Normal, shortcut, selected, selected_shortcut
        // 5: Disabled — DarkGray on LightGray. Mirrors Borland's
        // cpStatusLine[1] (cNormDisabled), which points at app-palette
        // position 3 (`0x78`) so disabled status items grey out against
        // the same light-gray bar background.
        3,
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

    // HelpViewer palette - uses cyan window colors for classic help appearance
    // Borland: cHelpViewer used cyan window background
    // These indices are remapped through CP_CYAN_WINDOW to get final app palette colors
    // Extended for rich text: normal, links, selected, bold, italic, code
    #[rustfmt::skip]
    pub const CP_HELP_VIEWER: &[u8] = &[
        1,  // 1: Normal text -> CP_CYAN[0] = 16 -> 0x30 (cyan bg, black fg)
        2,  // 2: Link/keyword text -> CP_CYAN[1] = 17 -> 0x3F (cyan bg, bright white)
        6,  // 3: Selected link -> CP_CYAN[5] = 21 -> 0x3E (cyan bg, yellow)
        2,  // 4: Bold text -> CP_CYAN[1] = 17 -> 0x3F (cyan bg, bright white)
        3,  // 5: Italic text -> CP_CYAN[2] = 18 -> 0x3A (cyan bg, bright green)
        4,  // 6: Code text -> CP_CYAN[3] = 19 -> 0x13 (blue bg, cyan fg)
    ];

    // EditorWindow palette (TEditor view)
    // Borland: cpEditor = "\x06\x07" (6, 7)
    // Same as CP_SCROLLER - editors use window background colors
    #[rustfmt::skip]
    pub const CP_EDITOR: &[u8] = &[
        6, 7,                                          // 1-2: Normal text, Selected text
        9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,   // 3-13: Syntax colors (window-relative)
    ];

    // History Viewer palette (THistoryViewer)
    // Borland: cpHistoryViewer = "\x06\x06\x07\x06\x06" (6, 6, 7, 6, 6)
    #[rustfmt::skip]
    pub const CP_HISTORY_VIEWER: &[u8] = &[
        6, 6, 7, 6, 6,  // 1-5: Normal, Normal bg, Selected, Divider bg, Arrows
    ];

    // History dropdown button palette (THistory)
    // Borland: cpHistory = "\x16\x17" (22, 23)
    #[rustfmt::skip]
    pub const CP_HISTORY: &[u8] = &[
        22, 23,  // 1-2: Normal button, Arrow icon
    ];

    // Background palette (TBackground)
    // Borland: cpBackground = "\x01" (1)
    #[rustfmt::skip]
    pub const CP_BACKGROUND: &[u8] = &[
        1,  // 1: Background color (maps to app palette position 1)
    ];
}

#[cfg(test)]
mod tests {
    use super::palettes::*;
    use super::*;

    /// Resolves a 1-based app palette index to its attribute byte.
    fn app(index: u8) -> u8 {
        CP_APP_COLOR[index as usize - 1]
    }

    #[test]
    fn app_palette_restored_borland_bytes() {
        // Positions 15, 23, 31 were accidentally zeroed; Borland cpAppColor
        // has 0x1F, 0x3F, 0x7E at those positions.
        assert_eq!(app(15), 0x1F);
        assert_eq!(app(23), 0x3F);
        assert_eq!(app(31), 0x7E);
    }

    #[test]
    fn blue_dialog_maps_to_blue_backgrounds() {
        // CP_BLUE_DIALOG entry 1 (frame passive) must resolve to a blue
        // background (Borland 0x17: light gray on blue).
        let frame = app(CP_BLUE_DIALOG[0]);
        assert_eq!(frame, 0x17);
        assert_eq!(Attr::from_u8(frame).bg, TvColor::Blue);

        // Entry 2 (frame active) is 0x1F: white on blue.
        let active = app(CP_BLUE_DIALOG[1]);
        assert_eq!(active, 0x1F);
        assert_eq!(Attr::from_u8(active).bg, TvColor::Blue);

        // All CP_BLUE_DIALOG targets must be within the app palette.
        for &idx in CP_BLUE_DIALOG {
            assert!((idx as usize) <= CP_APP_COLOR.len());
        }
        // And they must not collide with the syntax-color region (64-96).
        for &idx in CP_BLUE_DIALOG {
            assert!(idx < 64 || idx > 96);
        }
    }

    #[test]
    fn blue_dialog_matches_borland_values() {
        // The relocated region must carry Borland cpBlueDialog's attribute
        // bytes (Borland app entries 64-95).
        let borland: [u8; 32] = [
            0x17, 0x1F, 0x1A, 0x71, 0x71, 0x1E, 0x17, 0x1F, 0x1E, 0x20, 0x2B, 0x2F, 0x78, 0x2E,
            0x10, 0x30, 0x3F, 0x3E, 0x70, 0x2F, 0x7A, 0x20, 0x12, 0x31, 0x31, 0x30, 0x2F, 0x3E,
            0x31, 0x13, 0x38, 0x00,
        ];
        for (i, &idx) in CP_BLUE_DIALOG.iter().enumerate() {
            assert_eq!(app(idx), borland[i], "blue dialog entry {}", i + 1);
        }
    }

    #[test]
    fn cluster_palette_matches_borland() {
        // Borland cpCluster = "\x10\x11\x12\x12\x1f"
        assert_eq!(CP_CLUSTER, &[16, 17, 18, 18, 31]);
        // The disabled entry resolves through the (restored) app entry 31.
        assert_eq!(app(CP_CLUSTER[CLUSTER_DISABLED as usize - 1]), 0x7E);
        // Out-of-range indices still hit the Palette::get() fallback.
        let pal = Palette::from_slice(CP_CLUSTER);
        assert_eq!(pal.get(6), 0);
    }

    #[test]
    fn test_style_bitset() {
        let s = Style::BOLD | Style::ITALIC;
        assert!(s.contains(Style::BOLD));
        assert!(s.contains(Style::ITALIC));
        assert!(!s.contains(Style::UNDERLINE));
        assert!(Style::empty().is_empty());
        assert!(!s.is_empty());

        let mut m = Style::empty();
        m.insert(Style::UNDERLINE);
        assert!(m.contains(Style::UNDERLINE));
        m.remove(Style::UNDERLINE);
        assert!(!m.contains(Style::UNDERLINE));
        assert!(m.is_empty());
    }

    #[test]
    fn test_attr_style_builders() {
        let a = Attr::new(TvColor::White, TvColor::Blue);
        assert!(a.style.is_empty());

        let b = a.bold().italic();
        assert!(b.style.contains(Style::BOLD));
        assert!(b.style.contains(Style::ITALIC));
        assert_eq!(b.fg, TvColor::White);
        assert_eq!(b.bg, TvColor::Blue);

        // with_style replaces the style set
        let c = a.with_style(Style::UNDERLINE | Style::REVERSE);
        assert!(c.style.contains(Style::UNDERLINE));
        assert!(c.style.contains(Style::REVERSE));
        assert!(!c.style.contains(Style::BOLD));
    }

    #[test]
    fn test_attr_u8_drops_style() {
        let a = Attr::new(TvColor::White, TvColor::Blue).bold();
        let round = Attr::from_u8(a.to_u8());
        assert_eq!(round.fg, TvColor::White);
        assert_eq!(round.bg, TvColor::Blue);
        assert!(round.style.is_empty(), "style is not representable in the color byte");
    }

    #[test]
    fn test_attr_swap_darken_preserve_style() {
        let a = Attr::new(TvColor::White, TvColor::Blue).underline();
        assert!(a.swap().style.contains(Style::UNDERLINE));
        assert!(a.darken(0.5).style.contains(Style::UNDERLINE));
    }
}
