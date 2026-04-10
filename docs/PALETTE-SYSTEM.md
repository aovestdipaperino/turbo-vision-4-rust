# Turbo Vision Palette System

## Overview

The Turbo Vision palette system provides **indirect color mapping** that allows views to define logical color indices that are remapped through a hierarchy of palettes until reaching actual terminal color attributes. This design enables consistent theming and color inheritance throughout the UI hierarchy.

## Borland's Original Implementation

### Concept

In Borland Turbo Vision (C++), each `TView` has:
- An `owner` pointer to its parent `TGroup`
- A `getPalette()` method that returns a palette for that view type
- A `mapColor(uchar index)` method that walks up the owner chain

### Color Mapping Process

When a view needs to draw with a color, it calls `mapColor(logicalIndex)`:

1. **View's Palette**: Remap logical index through the view's own palette
2. **Owner Chain Walk**: Walk up through `owner->owner->owner...`
3. **Parent Palettes**: At each level, remap through that parent's palette
4. **Application Root**: Reach the application, which has the final color attributes

### Example in Borland C++

```cpp
// Button wants to draw with color 3 (normal text)
Attr color = mapColor(3);

// Walk up the chain:
// 1. Button palette:     3 -> 14  (button's "normal text" maps to dialog color 14)
// 2. Dialog palette:     14 -> 45 (dialog color 14 maps to app color 45)
// 3. Application palette: 45 -> 0x2F (app color 45 is actual attribute: bright white on green)
```

### Borland's Owner Chain

```
Application (root)
  ├─ Desktop
  │   └─ Window
  │       └─ Dialog
  │           └─ Button
```

Each view stores a raw `owner` pointer to its parent, forming a linked list that `mapColor()` traverses.

## Rust Implementation

### QCell-Based Safe Palette Chain (v1.1.0+)

The Rust implementation faithfully reproduces Borland's owner-chain walk using
`qcell::QCell` for safe shared access, with zero `unsafe` code. Each view
stores an `Option<PaletteChainNode>` -- a reference-counted, QCell-protected
node that holds a palette and a link to its parent's node.

A single `QCellOwner` lives in a `static OnceLock` (accessed via
`palette_token()`), so `draw()` and `map_color()` keep their original
signatures with no token parameter threading.

For the full design rationale, thread-safety analysis, and architecture
diagram, see [PALETTE-SYSTEM-DESIGN.md](PALETTE-SYSTEM-DESIGN.md).

### Chain Setup During Draw

During `draw()`, each parent builds its palette chain node and sets it on
children before drawing them:

```rust
// Group::draw() -- Group is typically transparent (no palette)
let my_node = PaletteChainNode::new(
    self.get_palette(),          // None for Group
    self.palette_chain.clone(),  // link to parent's node (e.g. Window)
);
for child in &mut self.children {
    child.set_palette_chain(Some(my_node.clone()));
    child.draw(terminal);
}
```

Window::draw() does the same but provides a real palette (CP_BLUE_WINDOW,
CP_GRAY_DIALOG, etc.) as the chain root.

### Implementation in `View::map_color()`

```rust
fn map_color(&self, color_index: u8) -> Attr {
    let mut color = color_index;

    // Step 1: Remap through this view's own palette
    if let Some(palette) = self.get_palette() {
        color = palette.get(color as usize);
    }

    // Step 2: Walk up the QCell chain (safe, no unsafe)
    if let Some(chain_node) = self.get_palette_chain() {
        color = chain_node.remap_color(color);
    }

    // Step 3: Resolve through application palette
    let app_palette = palettes::get_app_palette();
    Attr::from_u8(app_palette[(color - 1) as usize])
}
```

### Per-View Storage

Each view stores a single optional chain node:

```rust
struct Button {
    // ... other fields
    palette_chain: Option<PaletteChainNode>,  // set by parent during draw
}
```

Benefits:
- No raw pointers or `unsafe` code anywhere in the view system
- Faithful reproduction of Borland's chain-walk semantics
- Original `draw()` and `map_color()` signatures preserved (no parameter changes)
- Thread-safe: static `QCellOwner` is `Sync`, nodes use `Rc` (`!Send`)
- Negligible runtime cost: one `Rc` clone per child per frame

## Palette Definitions

### Application Palette (CP_APP_COLOR)

The root palette containing **actual terminal color attributes** (foreground/background pairs). Matches Borland's cpColor exactly:

```rust
pub const CP_APP_COLOR: &[u8] = &[
    0x71, 0x70, 0x78, 0x74, 0x20, 0x28, 0x24, 0x17, // 1-8: Desktop colors
    0x1F, 0x1A, 0x31, 0x31, 0x1E, 0x71, 0x00,       // 9-15: Menu colors
    0x30, 0x3F, 0x3A, 0x13, 0x13, 0x3E, 0x21, 0x00, // 16-23: Cyan Window
    0x70, 0x7F, 0x7A, 0x13, 0x13, 0x70, 0x7F, 0x00, // 24-31: Gray Window
    0x70, 0x7F, 0x7A, 0x13, 0x13, 0x70, 0x70, 0x7F, // 32-39: Dialog
    0x7E, 0x20, 0x2B, 0x2F, 0x78, 0x2E, 0x70, 0x30, // 40-47: Dialog controls
    0x3F, 0x3E, 0x1F, 0x2F, 0x1A, 0x20, 0x72, 0x31, // 48-55: Dialog
    0x31, 0x30, 0x2F, 0x3E, 0x31, 0x13, 0x38, 0x00, // 56-63: Dialog
    // 64-74: Syntax highlighting (Blue background)
    0x17, 0x1E, 0x1C, 0x1F, 0x1D, 0x1F, 0x17, 0x1A, 0x1F, 0x1B, 0x1F,
    // 75-85: Syntax highlighting (Cyan background)
    0x37, 0x3E, 0x3C, 0x3F, 0x3D, 0x3F, 0x37, 0x3A, 0x3F, 0x3B, 0x3F,
    // 86-96: Syntax highlighting (Gray background)
    0x70, 0x7E, 0x7C, 0x78, 0x7D, 0x70, 0x70, 0x72, 0x78, 0x71, 0x74,
];
```

Palette layout:
- 1: TBackground
- 2-7: TMenuView and TStatusLine
- 8-15: TWindow (Blue)
- 16-23: TWindow (Cyan)
- 24-31: TWindow (Gray)
- 32-63: TDialog
- 64-74: Syntax highlighting (Blue background)
- 75-85: Syntax highlighting (Cyan background)
- 86-96: Syntax highlighting (Gray background)

Color attributes use format: `0xBF` where:
- `B` = background color (high nibble)
- `F` = foreground color (low nibble)

Example: `0x2F` = bright white (F) on green (2)

### Gray Dialog Palette (CP_GRAY_DIALOG)

Maps dialog-level color indices to application palette indices:

```rust
pub const CP_GRAY_DIALOG: &[u8] = &[
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41,  // 1-10: Dialog colors map to app 32-41
    42, 43, 44, 45, 46, 47, 48, 49, 50, 51,  // 11-20: More mappings
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61,  // 21-30
    62, 63,                                   // 31-32
];
```

This palette provides the "gray dialog" theme where dialogs have gray backgrounds.

### View-Specific Palettes

Each view type defines its own palette mapping its logical colors to parent (dialog) colors:

**Button Palette (CP_BUTTON)** - Matches Borland cpButton `"\x0A\x0B\x0C\x0D\x0E\x0E\x0E\x0F"`:
```rust
pub const CP_BUTTON: &[u8] = &[
    10, 11, 12, 13, 14, 14, 14, 15,  // Maps to dialog colors 10-15
];
```

Button color indices (when inside a Dialog):
- 1: Normal → Dialog[10]=41 → App[41]=0x20 (Black on Green)
- 2: Default → Dialog[11]=42 → App[42]=0x2B (LightGreen on Green)
- 3: Focused → Dialog[12]=43 → App[43]=0x2F (White on Green)
- 4: Disabled → Dialog[13]=44 → App[44]=0x78 (DarkGray on LightGray)
- 5-7: Shortcut → Dialog[14]=45 → App[45]=0x2E (Yellow on Green)
- 8: Shadow → Dialog[15]=46 → App[46]=0x70 (Black on LightGray)

**Label Palette (CP_LABEL)** - Matches Borland cpLabel `"\x07\x08\x09\x09\x0D\x0D"`:
```rust
pub const CP_LABEL: &[u8] = &[
    7, 8, 9, 9, 13, 13,  // 6 entries for normal fg/bg, light fg/bg, disabled fg/bg
];
```

Label colors (when inside a Dialog):
- 1: Normal fg → Dialog[7]=38 → App[38]=0x70 (Black on LightGray)
- 2: Normal bg → Dialog[8]=39 → App[39]=0x7F (White on LightGray)
- 3-4: Light → Dialog[9]=40 → App[40]=0x7E (Yellow on LightGray)
- 5-6: Disabled → Dialog[13]=44 → App[44]=0x78 (DarkGray on LightGray)

**StaticText Palette (CP_STATIC_TEXT)** - Matches Borland cpStaticText `"\x06"`:
```rust
pub const CP_STATIC_TEXT: &[u8] = &[
    6,  // Single color index
];
```

StaticText color (when inside a Dialog):
- 1: Normal → Dialog[6]=37 → App[37]=0x70 (Black on LightGray)

**MenuBar Palette (CP_MENU_BAR)** - Top-level view (top-level, no parent palette):
```rust
pub const CP_MENU_BAR: &[u8] = &[
    2, 5, 3, 4,  // Direct app palette indices (no dialog remapping)
];
```

MenuBar colors (NO dialog remapping, goes directly to app):
- 1: Normal → App[2]=0x70 (Black on LightGray)
- 2: Selected → App[5]=0x20 (Black on Green)
- 3: Disabled → App[3]=0x78 (DarkGray on LightGray)
- 4: Shortcut → App[4]=0x74 (Red on LightGray)

## Complete Color Mapping Example

Let's trace how a **Button's focused text** (logical color 3) becomes a terminal color when in a Dialog:

### Step 1: Button's Palette
```
Button logical color 3 → CP_BUTTON[3] = 12
```
Button's "focused text" maps to dialog color 12.

### Step 2: Check Owner Type
```
button has palette_chain → walk chain through Dialog palette → remap
```

### Step 3: Gray Dialog Palette
```
Dialog color 12 → CP_GRAY_DIALOG[12] = 43
```
Dialog color 12 maps to application color 43.

### Step 4: Application Palette
```
Application color 43 → CP_APP_COLOR[43] = 0x2F
```
Application color 43 is the actual terminal attribute: `0x2F` = **White on Green**.

### Final Result
```
Button.map_color(3) → 0x2F (White on Green)
```

### Example: MenuBar (Top-Level View)

Let's trace how a **MenuBar's selected item** (logical color 2) becomes a terminal color:

### Step 1: MenuBar's Palette
```
MenuBar logical color 2 → CP_MENU_BAR[2] = 5
```
MenuBar's "selected" maps to app color 5.

### Step 2: Check Owner Type
```
menubar has no palette_chain → skip chain walk, go direct to app palette
```

### Step 3: Application Palette (Direct)
```
Application color 5 → CP_APP_COLOR[5] = 0x20
```
Application color 5 is the actual terminal attribute: `0x20` = **Black on Green**.

### Final Result
```
MenuBar.map_color(2) → 0x20 (Black on Green)
```

## Comparison: Borland vs Rust

| Aspect | Borland C++ | Rust Implementation |
|--------|-------------|---------------------|
| **Owner Storage** | Raw `TView* owner` pointer | `PaletteChainNode` (Rc\<QCell\>) |
| **Chain Traversal** | Runtime walk via `owner->owner` | QCell chain walk via `remap_color()` |
| **Safety** | Unsafe raw pointers | 100% safe Rust (zero `unsafe` in views) |
| **Flexibility** | Dynamic, any hierarchy depth | Dynamic, any hierarchy depth (faithful reproduction) |
| **Performance** | Pointer dereferences + virtual calls | Direct palette lookups + enum check |
| **Visual Output** | Depends on actual hierarchy | Same colors via context-aware remapping |
| **Context Awareness** | Implicit (via owner chain) | Implicit (via QCell palette chain) |

## Advantages of the Rust Approach

### Safety
- ✅ No undefined behavior from invalid pointers
- ✅ No crashes from moved views
- ✅ Compiler-verified correctness

### Simplicity
- ✅ Easier to understand (no pointer chasing)
- ✅ Easier to debug (deterministic mapping)
- ✅ Less code complexity

### Performance
- ✅ No pointer dereferencing overhead
- ✅ No virtual function calls up the chain
- ✅ Direct array lookups

## Limitations

### Dynamic Palette Chain

The QCell-based palette chain supports arbitrary nesting depth, faithfully
reproducing Borland's dynamic owner chain traversal. Any view hierarchy
(Window, Dialog, nested Groups, custom containers) works automatically
because the chain is built from each view's actual `get_palette()` at draw time.
- Runtime-switchable palette chains

### When This Matters

The context limitation only affects advanced scenarios like:
- Custom container types with unique palettes (rare)
- Deeply nested groups with different themes (uncommon)
- Dynamic palette switching at runtime (unusual)

For standard Turbo Vision applications (Desktop → Window/Dialog → Controls), the context-aware remapping produces **identical visual results** to Borland's dynamic owner chain traversal.

### Testing and Validation

The palette system includes comprehensive regression tests:
- 9 palette regression tests in `tests/palette_regression_tests.rs`
- Tests verify Borland-accurate colors for all UI components
- Tests cover both Dialog-context and top-level views
- All tests ensure color stability across changes

## Runtime Palette Customization

The palette system supports runtime customization of the entire application palette, allowing you to create custom themes:

### Using `Application::set_palette()`

The `Application::set_palette()` method provides a convenient way to change the application palette with automatic redrawing:

```rust
use turbo_vision::app::Application;

let mut app = Application::new()?;

// Create a custom dark theme palette (63 bytes)
// Each byte encodes: (foreground << 4) | background
let dark_palette = vec![
    0x08, 0x0F, 0x08, 0x0E, 0x0B, 0x0A, 0x0C, 0x01, // Desktop
    0xF1, 0xE1, 0xF3, 0xF3, 0xF1, 0x08, 0x00,       // Menu
    // ... 63 bytes total
];

// Set the custom palette (redraw happens automatically!)
app.set_palette(Some(dark_palette));

// Reset to default Borland palette
app.set_palette(None);
```

### How It Works

1. **Automatic Redraw**: `set_palette()` automatically calls `needs_redraw()` when the palette changes
2. **Change Detection**: Only triggers redraw if the palette actually differs from the current one
3. **Thread-Local Storage**: Custom palette is stored in a thread-local `RefCell<Option<Vec<u8>>>`
4. **Transparent Remapping**: All views automatically use the new palette through `map_color()`

### Custom Palette Format

The application palette (`CP_APP_COLOR`) is a 63-byte array where each byte encodes a color attribute:

```
Byte format: 0xBF
  B = Background color (high nibble, 0-F)
  F = Foreground color (low nibble, 0-F)

Color values:
  0=Black, 1=Blue, 2=Green, 3=Cyan, 4=Red, 5=Magenta, 6=Brown, 7=LightGray
  8=DarkGray, 9=LightBlue, A=LightGreen, B=LightCyan, C=LightRed,
  D=LightMagenta, E=Yellow, F=White
```

### Palette Layout (indices 1-63)

```
 1-8:   Desktop colors
 9-15:  Menu and StatusLine
16-23:  Cyan Window theme
24-31:  Gray Window theme
32-63:  Dialog and control colors
```

### Example: Creating Themes

See `examples/palette_themes_demo.rs` for a complete example with multiple themes:

```rust
// Dark theme with dark backgrounds
let dark_palette = vec![
    0x08, 0x0F, 0x08, 0x0E, 0x0B, 0x0A, 0x0C, 0x01,
    0xF1, 0xE1, 0xF3, 0xF3, 0xF1, 0x08, 0x00,
    // ... rest of palette
];

// High-contrast theme (black on white, white on black)
let contrast_palette = vec![
    0x0F, 0xF0, 0x0F, 0xE0, 0xF0, 0xE0, 0xF0, 0xF0,
    0x0F, 0xE0, 0x0F, 0x0F, 0x0F, 0x0F, 0x00,
    // ... rest of palette
];

// Switch between themes
match theme_choice {
    ThemeChoice::Dark => app.set_palette(Some(dark_palette)),
    ThemeChoice::Contrast => app.set_palette(Some(contrast_palette)),
    ThemeChoice::Default => app.set_palette(None),
}
```

### Implementation Details

The `set_palette()` method in `Application`:

```rust
pub fn set_palette(&mut self, palette: Option<Vec<u8>>) {
    use crate::core::palette::palettes;

    // Get current palette to check if it's actually changing
    let current_palette = palettes::get_app_palette();
    let is_changing = match &palette {
        Some(new_palette) => new_palette != &current_palette,
        None => current_palette != palettes::CP_APP_COLOR,
    };

    // Set the new palette
    palettes::set_custom_palette(palette);

    // Trigger redraw only if the palette actually changed
    if is_changing {
        self.needs_redraw = true;
    }
}
```

### Low-Level API

For advanced use cases, you can use the low-level palette API:

```rust
use turbo_vision::core::palette::palettes;

// Set palette manually (no automatic redraw)
palettes::set_custom_palette(Some(custom_palette));

// Get current palette (custom or default)
let current = palettes::get_app_palette();

// Manually trigger redraw
app.needs_redraw();
```

### Testing and Validation

The palette system includes comprehensive regression tests:
- 9 palette regression tests in `tests/palette_regression_tests.rs`
- Tests verify Borland-accurate colors for all UI components
- Tests cover both Dialog-context and top-level views
- All tests ensure color stability across changes

## Future Enhancements

If dynamic palette chains are needed, safe alternatives include:

### Option 1: Palette Caching
When a view is added to a parent, compute and cache the full palette chain:
```rust
struct View {
    // Cache the resolved palette chain when added to parent
    cached_palette_chain: Option<Palette>,
}
```

### Option 2: Rc<RefCell<dyn View>>
Use reference-counted smart pointers instead of raw pointers:
```rust
struct View {
    owner: Option<Weak<RefCell<dyn View>>>,
}
```

### Option 3: Callback-Based Resolution
Pass a color resolver function during drawing:
```rust
fn draw(&mut self, terminal: &mut Terminal, color_resolver: &dyn Fn(u8) -> Attr)
```

## Conclusion

The current palette system eliminates unsafe code while maintaining visual compatibility with Borland Turbo Vision. By using QCell-based palette chain nodes instead of raw owner pointers, we achieve:

- **100% memory safety** (simple enum field, no raw pointers, no unsafe code)
- **Identical visual output** for standard UI layouts (verified by regression tests)
- **Simpler implementation** with better performance (direct lookups, no pointer chasing)
- **Context-aware remapping** that matches Borland's behavior
- **Maintained compatibility** with the Borland design philosophy

The context-aware palette system is a pragmatic design that prioritizes safety and simplicity while providing the flexibility needed for real-world Turbo Vision applications. The three context types (None, Window, Dialog) cover all standard use cases, and the comprehensive test suite ensures ongoing correctness.
