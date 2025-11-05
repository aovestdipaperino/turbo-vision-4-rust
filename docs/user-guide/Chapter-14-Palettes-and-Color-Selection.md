# Chapter 14: Palettes and Color Selection

No one ever seems to agree on what colors are "best" for any computer screen. Rather than dictating the colors of screen items, Turbo Vision enables both programmers and users to vary the colors of views. This chapter covers the features of Turbo Vision you need to understand to work with colors: color palettes and color attributes.

## Using Color Palettes

Instead of making you specify the color of every view in your application, Turbo Vision uses a centralized color system to manage all the colors of all the views. For example, when you create a menu bar, you don't have to tell it what color you want it to be. It gets that information from predefined color constants. You can change colors by modifying these constants, which will change the color of every menu in the application. If you want to have a single menu that's a different color from all the other menus, you can use a different color constant for it.

The only time you have to concern yourself with colors is when writing draw methods. Draw is the only method that puts information on the screen.

The remainder of this section covers the following topics:

- Understanding color attributes
- Using default colors
- Defining new colors
- Color customization strategies

## Understanding Color Attributes

The Rust implementation of Turbo Vision uses a type-safe color system defined in `src/core/palette.rs`. Colors are represented using two primary types:

### The TvColor Enum

The `TvColor` enum represents the 16 standard colors available in Turbo Vision:

```rust
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
```

Each color has a numeric value from 0 to 15, matching the original Turbo Vision color scheme. The `TvColor` enum provides methods for conversion:

```rust
// Convert to crossterm Color for terminal rendering
let term_color = TvColor::Yellow.to_crossterm();

// Create from a u8 value
let color = TvColor::from_u8(14);  // Yellow
```

### The Attr Structure

The `Attr` structure combines foreground and background colors into a single attribute:

```rust
pub struct Attr {
    pub fg: TvColor,  // Foreground color
    pub bg: TvColor,  // Background color
}
```

Creating color attributes is straightforward:

```rust
use turbo_vision::core::palette::{Attr, TvColor};

// Create yellow text on blue background
let attr = Attr::new(TvColor::Yellow, TvColor::Blue);

// Create from a single byte (for compatibility)
let attr = Attr::from_u8(0x1E);  // Same as above
```

The attribute can be converted to/from a single byte for efficient storage:

```rust
// Convert to byte
let byte = attr.to_u8();  // Returns 0x1E

// Convert from byte
let attr = Attr::from_u8(0x1E);
```

The byte format matches the original Turbo Vision format:
- Lower 4 bits (0-3): Foreground color (0-15)
- Upper 4 bits (4-7): Background color (0-15)

For example, `0x1E` means:
- `0xE` (14) = Yellow foreground
- `0x1` (1) = Blue background

## Using Default Colors

The Rust implementation provides a comprehensive set of predefined color constants in the `colors` module (see `src/core/palette.rs:94-150`). These constants define the standard appearance of all UI elements:

### General UI Colors

```rust
use turbo_vision::core::palette::colors;

// Basic UI colors
colors::NORMAL          // Light gray on blue
colors::HIGHLIGHTED     // Yellow on blue
colors::SELECTED        // White on cyan
colors::DISABLED        // Dark gray on blue
```

### Menu Colors

```rust
// Menu bar and menu items
colors::MENU_NORMAL     // Black on light gray
colors::MENU_SELECTED   // White on green
colors::MENU_DISABLED   // Dark gray on light gray
colors::MENU_SHORTCUT   // Red on light gray
```

### Dialog Colors

```rust
// Dialog boxes
colors::DIALOG_NORMAL        // Black on light gray
colors::DIALOG_FRAME         // White on light gray
colors::DIALOG_FRAME_ACTIVE  // White on light gray
colors::DIALOG_TITLE         // White on light gray
colors::DIALOG_SHORTCUT      // Red on light gray
```

### Button Colors

```rust
// Buttons
colors::BUTTON_NORMAL    // Black on green
colors::BUTTON_DEFAULT   // Light green on green
colors::BUTTON_SELECTED  // White on green
colors::BUTTON_DISABLED  // Dark gray on green
colors::BUTTON_SHORTCUT  // Yellow on green
colors::BUTTON_SHADOW    // Light gray on dark gray
```

### Input Line Colors

```rust
// Input fields
colors::INPUT_NORMAL   // Black on light gray
colors::INPUT_FOCUSED  // Yellow on blue
```

### List Box Colors

```rust
// List boxes
colors::LISTBOX_NORMAL            // Black on light gray
colors::LISTBOX_FOCUSED           // Black on white
colors::LISTBOX_SELECTED          // White on blue
colors::LISTBOX_SELECTED_FOCUSED  // White on cyan
```

### Scroll Bar Colors

```rust
// Scroll bars
colors::SCROLLBAR_PAGE      // Dark gray on light gray
colors::SCROLLBAR_INDICATOR // Blue on light gray
colors::SCROLLBAR_ARROW     // Black on light gray
```

### Editor Colors

```rust
// Text editors
colors::EDITOR_NORMAL    // White on blue
colors::EDITOR_SELECTED  // Black on cyan
```

### Other UI Elements

```rust
// Status line
colors::STATUS_NORMAL             // Black on light gray
colors::STATUS_SHORTCUT           // Red on light gray
colors::STATUS_SELECTED           // White on green
colors::STATUS_SELECTED_SHORTCUT  // Yellow on green

// Desktop background
colors::DESKTOP  // Light gray on dark gray

// Scroller views
colors::SCROLLER_NORMAL    // Black on light gray
colors::SCROLLER_SELECTED  // White on blue

// Help system
colors::HELP_NORMAL   // Black on light gray
colors::HELP_FOCUSED  // Black on white
```

## Using Colors in Draw Methods

When you write a draw method for a view, you use these predefined color constants to specify how elements should appear:

```rust
use turbo_vision::views::view::View;
use turbo_vision::terminal::Terminal;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::palette::colors;

impl View for MyView {
    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds().width() as usize;
        let mut buffer = DrawBuffer::new(width);

        // Use predefined colors based on focus state
        let color = if self.is_focused() {
            colors::INPUT_FOCUSED
        } else {
            colors::INPUT_NORMAL
        };

        // Draw text with the selected color
        buffer.move_str(0, "Hello, World!", color);

        // Write to terminal
        write_line_to_terminal(
            terminal,
            self.bounds().a.x,
            self.bounds().a.y,
            &buffer
        );
    }
}
```

## Defining New Colors

To define new colors for custom views, you can create your own color constants using the same pattern:

```rust
use turbo_vision::core::palette::{Attr, TvColor};

// Define custom colors for your application
pub mod my_colors {
    use super::*;

    // Custom application colors
    pub const HEADER: Attr = Attr::new(TvColor::Yellow, TvColor::Red);
    pub const FOOTER: Attr = Attr::new(TvColor::White, TvColor::Blue);
    pub const SPECIAL: Attr = Attr::new(TvColor::LightCyan, TvColor::Magenta);
}
```

Then use them in your draw methods:

```rust
fn draw(&mut self, terminal: &mut Terminal) {
    let mut buffer = DrawBuffer::new(width);

    // Use custom colors
    buffer.move_str(0, "Header", my_colors::HEADER);
    buffer.move_str(10, "Content", colors::NORMAL);
    buffer.move_str(20, "Footer", my_colors::FOOTER);

    // ...
}
```

## Color Selection Based on State

Views often need to change colors based on their state (focused, selected, disabled, etc.). The standard pattern is to select colors conditionally:

```rust
fn draw(&mut self, terminal: &mut Terminal) {
    let mut buffer = DrawBuffer::new(width);

    // Select color based on view state
    let color = if !self.is_enabled() {
        colors::DISABLED
    } else if self.is_focused() {
        colors::HIGHLIGHTED
    } else if self.is_selected() {
        colors::SELECTED
    } else {
        colors::NORMAL
    };

    buffer.move_str(0, "Text", color);
    // ...
}
```

For buttons, the pattern might be:

```rust
fn draw(&mut self, terminal: &mut Terminal) {
    let (text_color, shortcut_color) = if self.is_focused() {
        (colors::BUTTON_SELECTED, colors::BUTTON_SHORTCUT)
    } else if self.is_default() {
        (colors::BUTTON_DEFAULT, colors::BUTTON_SHORTCUT)
    } else {
        (colors::BUTTON_NORMAL, colors::BUTTON_SHORTCUT)
    };

    // Draw button text with appropriate colors
    buffer.move_str(0, "[ ", text_color);
    buffer.move_char(2, 'O', shortcut_color, 1);  // Hotkey
    buffer.move_str(3, "K ]", text_color);
}
```

## Understanding Color Consistency

The centralized color system in Turbo Vision ensures consistency across your application:

### Consistent UI Elements

All instances of the same UI element type use the same colors by default. All normal buttons use `colors::BUTTON_NORMAL`, all input lines use `colors::INPUT_NORMAL` when unfocused, etc.

### Focus Indication

Views automatically change colors when they gain or lose focus. Input lines change from `INPUT_NORMAL` to `INPUT_FOCUSED`, list boxes highlight the selected item differently based on focus state, etc.

### Thematic Consistency

Related elements use related colors. All dialog elements (frames, titles, shortcuts) use colors from the `DIALOG_*` family, all menu elements use colors from the `MENU_*` family, etc.

## Creating Color Schemes

To create an alternative color scheme for your application, you can define a complete set of replacement colors:

```rust
pub mod dark_scheme {
    use turbo_vision::core::palette::{Attr, TvColor};

    // Dark theme colors
    pub const NORMAL: Attr = Attr::new(TvColor::LightGray, TvColor::Black);
    pub const HIGHLIGHTED: Attr = Attr::new(TvColor::Yellow, TvColor::Black);
    pub const SELECTED: Attr = Attr::new(TvColor::Black, TvColor::LightGray);

    pub const MENU_NORMAL: Attr = Attr::new(TvColor::LightGray, TvColor::DarkGray);
    pub const MENU_SELECTED: Attr = Attr::new(TvColor::Yellow, TvColor::Black);

    pub const DIALOG_NORMAL: Attr = Attr::new(TvColor::LightGray, TvColor::DarkGray);
    pub const DIALOG_FRAME: Attr = Attr::new(TvColor::White, TvColor::DarkGray);

    pub const BUTTON_NORMAL: Attr = Attr::new(TvColor::LightGray, TvColor::DarkGray);
    pub const BUTTON_SELECTED: Attr = Attr::new(TvColor::Yellow, TvColor::Black);

    // ... etc.
}
```

Then modify your views to use the alternative scheme:

```rust
// Option 1: Use conditional compilation
#[cfg(feature = "dark_theme")]
use dark_scheme as colors;
#[cfg(not(feature = "dark_theme"))]
use turbo_vision::core::palette::colors;

// Option 2: Make it runtime configurable
fn get_color_scheme() -> &'static ColorScheme {
    if user_preferences.dark_mode {
        &DARK_SCHEME
    } else {
        &DEFAULT_SCHEME
    }
}
```

## Working with Individual Color Components

You can work with foreground and background colors separately:

```rust
use turbo_vision::core::palette::{Attr, TvColor};

// Create an attribute
let attr = Attr::new(TvColor::Yellow, TvColor::Blue);

// Access components
let fg = attr.fg;  // TvColor::Yellow
let bg = attr.bg;  // TvColor::Blue

// Create variations
let inverted = Attr::new(attr.bg, attr.fg);  // Blue on yellow
let same_bg = Attr::new(TvColor::Red, attr.bg);  // Red on blue
```

## Color Conversion and Compatibility

The Rust implementation provides compatibility with the original byte-based color format:

```rust
// Legacy color byte (from file or configuration)
let byte: u8 = 0x1E;

// Convert to Attr
let attr = Attr::from_u8(byte);
assert_eq!(attr.fg, TvColor::Yellow);
assert_eq!(attr.bg, TvColor::Blue);

// Convert back to byte
let byte2 = attr.to_u8();
assert_eq!(byte, byte2);
```

This is useful for:
- Loading color schemes from configuration files
- Maintaining compatibility with original Turbo Vision palettes
- Compact storage of color information

## Best Practices for Color Usage

### Use Semantic Names

Rather than thinking in terms of specific colors like "yellow on blue," think in terms of semantic purposes like `colors::HIGHLIGHTED` or `colors::MENU_SELECTED`. This makes it easier to change color schemes later.

**Good:**
```rust
let color = colors::BUTTON_SELECTED;
```

**Avoid:**
```rust
let color = Attr::new(TvColor::White, TvColor::Green);  // Less maintainable
```

### Respect Focus States

Always use different colors for focused and unfocused states:

```rust
let color = if self.is_focused() {
    colors::INPUT_FOCUSED
} else {
    colors::INPUT_NORMAL
};
```

### Group Related Elements

Keep related UI elements in the same color family. All button states should use the `BUTTON_*` colors, all menu items should use `MENU_*` colors, etc.

### Maintain Contrast

Ensure sufficient contrast between foreground and background colors for readability. The predefined colors are designed with good contrast ratios.

### Test in Different Environments

Terminal color rendering can vary across different terminal emulators. Test your color choices in the terminals your users are likely to use.

## Color Constants Reference

Here's a complete reference of available color constants:

| Constant | Foreground | Background | Usage |
|----------|------------|------------|-------|
| `NORMAL` | LightGray | Blue | Default text |
| `HIGHLIGHTED` | Yellow | Blue | Important text |
| `SELECTED` | White | Cyan | Selected items |
| `DISABLED` | DarkGray | Blue | Disabled elements |
| `MENU_NORMAL` | Black | LightGray | Menu items |
| `MENU_SELECTED` | White | Green | Selected menu item |
| `MENU_DISABLED` | DarkGray | LightGray | Disabled menu item |
| `MENU_SHORTCUT` | Red | LightGray | Menu shortcuts |
| `DIALOG_NORMAL` | Black | LightGray | Dialog interior |
| `DIALOG_FRAME` | White | LightGray | Dialog frame |
| `BUTTON_NORMAL` | Black | Green | Unfocused button |
| `BUTTON_DEFAULT` | LightGreen | Green | Default button |
| `BUTTON_SELECTED` | White | Green | Focused button |
| `BUTTON_SHORTCUT` | Yellow | Green | Button hotkey |
| `INPUT_NORMAL` | Black | LightGray | Unfocused input |
| `INPUT_FOCUSED` | Yellow | Blue | Focused input |
| `LISTBOX_NORMAL` | Black | LightGray | List items |
| `LISTBOX_SELECTED` | White | Blue | Selected list item |
| `EDITOR_NORMAL` | White | Blue | Editor text |
| `EDITOR_SELECTED` | Black | Cyan | Selected editor text |
| `DESKTOP` | LightGray | DarkGray | Desktop background |

## Example: Custom View with Colors

Here's a complete example of a custom view using the color system:

```rust
use turbo_vision::views::view::View;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::colors;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::views::view::write_line_to_terminal;

pub struct StatusIndicator {
    bounds: Rect,
    status: String,
    is_error: bool,
    state: StateFlags,
}

impl StatusIndicator {
    pub fn new(bounds: Rect, status: String, is_error: bool) -> Self {
        Self {
            bounds,
            status,
            is_error,
            state: 0,
        }
    }
}

impl View for StatusIndicator {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let mut buffer = DrawBuffer::new(width);

        // Select color based on status and focus
        let color = if self.is_error {
            // Error: red text on light gray
            Attr::new(TvColor::Red, TvColor::LightGray)
        } else if self.is_focused() {
            colors::HIGHLIGHTED
        } else {
            colors::NORMAL
        };

        // Draw status text
        buffer.move_str(0, &self.status, color);

        // Fill rest with spaces
        if self.status.len() < width {
            buffer.move_char(
                self.status.len(),
                ' ',
                color,
                width - self.status.len()
            );
        }

        write_line_to_terminal(
            terminal,
            self.bounds.a.x,
            self.bounds.a.y,
            &buffer
        );
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }
}
```

## Summary

The Rust implementation of Turbo Vision provides a type-safe, centralized color system:

- **`TvColor` enum**: Represents 16 standard colors with type safety
- **`Attr` structure**: Combines foreground and background colors
- **Predefined constants**: Complete set of colors for all UI elements in `colors` module
- **Byte compatibility**: Conversion to/from byte format for storage and compatibility

Key principles:
- Use semantic color names rather than specific color values
- Maintain consistency across similar UI elements
- Respect focus and state changes
- Create custom color schemes by defining new constant sets
- Test colors in different terminal environments

The color system is defined in `src/core/palette.rs` and used throughout the view implementations for consistent, attractive, and accessible user interfaces.
