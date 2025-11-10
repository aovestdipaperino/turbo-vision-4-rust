# Palette Architecture: CP_* Palettes vs Direct Colors

This document explains which components use Borland's CP_* palette system (dynamic palette mapping) versus direct color constants.

## Two Color Systems

### System 1: CP_* Palette Mapping (Borland-Compatible)

Components return a `CP_*` palette in `get_palette()` and use palette index constants with `map_color()`.

**How it works:**
1. Component calls `self.map_color(INDEX)` with a palette index (u8)
2. Index is remapped through component's palette (from `get_palette()`)
3. Then remapped through owner's palette (Dialog/Window)
4. Finally mapped to application palette (CP_APP_COLOR)
5. **Result:** Colors automatically adapt to palette changes

**Benefits:**
- ✅ Full Borland compatibility
- ✅ Supports runtime palette customization
- ✅ Context-aware color remapping (Dialog vs Window)
- ✅ Colors change when Application::set_palette() is called

### System 2: Direct Colors (Hardcoded Attr)

Components use direct `Attr` constants from the `colors` module, bypassing palette mapping.

**How it works:**
1. Component uses `colors::COMPONENT_STATE` directly
2. No palette remapping occurs
3. **Result:** Fixed colors that don't change with palette

**Benefits:**
- ✅ Simpler to use
- ✅ Predictable colors
- ❌ **Does NOT support runtime palette customization**
- ❌ **Does NOT adapt to theme changes**

---

## Components Using CP_* Palette System ✅

### Dialog Components (Proper Palette Mapping)

These components implement full Borland palette mapping:

| Component | CP_* Palette | Palette Indices | Notes |
|-----------|-------------|-----------------|-------|
| **Button** | `CP_BUTTON` | 1-8 | Normal, Default, Selected, Disabled, Shortcut, Shadow |
| **Label** | `CP_LABEL` | 1-6 | Normal fg/bg, Light fg/bg, Disabled fg/bg |
| **StaticText** | `CP_STATIC_TEXT` | 1 | Single normal color |
| **InputLine** | `CP_INPUT_LINE` | 1-4 | Normal, Focused, Selected, Arrows |
| **ListBox** | `CP_LISTBOX` | 1-4 | Normal, Focused, Selected, Divider |
| **CheckBox** | `CP_CLUSTER` | 1-4 | Normal, Focused, Shortcut, Disabled |
| **RadioButton** | `CP_CLUSTER` | 1-4 | Normal, Focused, Shortcut, Disabled |
| **ScrollBar** | `CP_SCROLLBAR` | 1-3 | Page, Arrows, Indicator |

**Color Remapping Flow:**
```
Button.map_color(1)
  → CP_BUTTON[1] = 10
  → CP_GRAY_DIALOG[10] = 41  (if OwnerType::Dialog)
  → CP_APP_COLOR[41] = 0x20 (Black on Green)
```

### Window/Frame Components

| Component | CP_* Palette | Notes |
|-----------|-------------|-------|
| **Frame** | `CP_GRAY_DIALOG` or `CP_BLUE_WINDOW` | Depending on FramePaletteType |
| **Window** | None | Uses Frame's palette |
| **Dialog** | None | Uses Window's palette |

### Top-Level Components

| Component | CP_* Palette | Palette Indices | Notes |
|-----------|-------------|-----------------|-------|
| **MenuBar** | `CP_MENU_BAR` | 1-4 | Normal, Selected, Disabled, Shortcut |
| **StatusLine** | `CP_STATUSLINE` | 1-4 | Normal, Shortcut, Selected, Selected Shortcut |

**Color Mapping (No Dialog Remapping):**
```
MenuBar.map_color(1)
  → CP_MENU_BAR[1] = 2
  → CP_APP_COLOR[2] = 0x70 (Black on LightGray)
  (No intermediate dialog palette)
```

### List-Based Components

All list viewers use CP_LISTBOX palette:

| Component | CP_* Palette | Notes |
|-----------|-------------|-------|
| **SortedListBox** | `CP_LISTBOX` | Sorted list implementation |
| **DirListBox** | `CP_LISTBOX` | Directory listing |
| **FileList** | `CP_LISTBOX` | File listing |
| **HistoryViewer** | `CP_LISTBOX` | History list |
| **Outline** | `CP_LISTBOX` | Tree/outline view |

---

## Components Using Direct Colors ❌

### Components That Do NOT Use CP_* Palettes

These components use direct `Attr` constants and **do NOT support runtime palette customization**:

| Component | Color Source | Constants Used | Themeable? |
|-----------|-------------|----------------|-----------|
| **Editor** | Palette indices (not CP_*) | `EDITOR_NORMAL`, `EDITOR_SELECTED`, `EDITOR_CURSOR` | ⚠️ Partially |
| **Memo** | Direct colors | `colors::EDITOR_NORMAL`, `colors::EDITOR_SELECTED` | ❌ No |
| **TextView** | Direct colors | Hardcoded in draw() | ❌ No |
| **Indicator** | Direct colors | Hardcoded in draw() | ❌ No |
| **Desktop** | App palette directly | `colors::DESKTOP` | ⚠️ Partially |
| **HelpViewer** | Direct colors | `colors::HELP_NORMAL`, `colors::HELP_FOCUSED` | ❌ No |
| **Scroller** | Direct colors | `colors::SCROLLER_NORMAL`, `colors::SCROLLER_SELECTED` | ❌ No |
| **ParamText** | Direct colors | Hardcoded in draw() | ❌ No |

### Special Case: Editor

**Editor uses a hybrid approach:**
```rust
// Editor.get_palette() returns None
fn get_palette(&self) -> Option<Palette> {
    None  // Editor uses hardcoded blue window colors
}

// But uses palette INDEX constants (not CP_* palette)
let default_color = self.map_color(EDITOR_NORMAL);  // EDITOR_NORMAL = 9 (u8)
```

**What happens:**
1. `EDITOR_NORMAL = 9` is a palette index (u8)
2. Since `get_palette()` returns None, no component palette remapping
3. Index 9 maps directly to `CP_APP_COLOR[9]`
4. **Result:** Editor colors CAN change if you modify CP_APP_COLOR via `Application::set_palette()`

**Why this works:**
- Editor is typically in a blue window context
- It uses direct app palette indices (9, 57) for editor-specific colors
- These indices point to blue window colors in CP_APP_COLOR
- When you call `app.set_palette()`, CP_APP_COLOR changes
- Editor colors update automatically! ✅

---

## Palette Constants Summary

### CP_* Palettes (Defined in src/core/palette.rs)

```rust
// Application-level palette (63 colors)
pub const CP_APP_COLOR: &[u8]        // Root palette with actual colors

// Window/Dialog container palettes
pub const CP_BLUE_WINDOW: &[u8]      // Maps to app palette 8-15
pub const CP_CYAN_WINDOW: &[u8]      // Maps to app palette 16-23
pub const CP_GRAY_WINDOW: &[u8]      // Maps to app palette 24-31
pub const CP_GRAY_DIALOG: &[u8]      // Maps to app palette 32-63
pub const CP_BLUE_DIALOG: &[u8]      // Maps to app palette 16-31

// Component-level palettes
pub const CP_BUTTON: &[u8]           // 8 indices for button states
pub const CP_STATIC_TEXT: &[u8]      // 1 index for static text
pub const CP_INPUT_LINE: &[u8]       // 4 indices for input states
pub const CP_LABEL: &[u8]            // 6 indices for label states
pub const CP_LISTBOX: &[u8]          // 4 indices for list states
pub const CP_SCROLLBAR: &[u8]        // 3 indices for scrollbar parts
pub const CP_CLUSTER: &[u8]          // 4 indices for checkbox/radio
pub const CP_STATUSLINE: &[u8]       // 4 indices for status states
pub const CP_MENU_BAR: &[u8]         // 4 indices for menu states
```

### Palette Index Constants (u8 values)

```rust
// Frame indices (lines 63-66)
pub const FRAME_INACTIVE: u8 = 1;
pub const FRAME_ACTIVE_BORDER: u8 = 2;
pub const FRAME_TITLE: u8 = 2;
pub const FRAME_ICON: u8 = 3;

// Window indices (lines 68-70)
pub const WINDOW_BACKGROUND: u8 = 1;
pub const BLUE_WINDOW_BACKGROUND: u8 = 5;

// Editor indices (lines 74-76) - Direct app palette indices
pub const EDITOR_NORMAL: u8 = 9;     // App palette 9 = 0x1F
pub const EDITOR_SELECTED: u8 = 57;  // App palette 57 = 0x30
pub const EDITOR_CURSOR: u8 = 57;    // App palette 57 = 0x30
```

### Direct Color Constants (colors module)

```rust
// colors module (lines 330+)
pub mod colors {
    pub const NORMAL: Attr = ...
    pub const MENU_NORMAL: Attr = ...
    pub const BUTTON_NORMAL: Attr = ...
    pub const EDITOR_NORMAL: Attr = ...  // ⚠️ Same name, different type!
    pub const LISTBOX_NORMAL: Attr = ...
    pub const SYNTAX_KEYWORD: Attr = ...
    // ... 61 total constants
}
```

**⚠️ Important:** `EDITOR_NORMAL` exists in TWO forms:
- `pub const EDITOR_NORMAL: u8 = 9;` - Palette index (themeable)
- `colors::EDITOR_NORMAL: Attr` - Direct color (not themeable)

---

## Runtime Palette Customization

### What Changes With `Application::set_palette()`

When you call `app.set_palette(Some(custom_palette))`, it updates `CP_APP_COLOR`.

**✅ Components that update automatically:**

All components using CP_* palettes + Editor (uses direct app palette indices):
- Button, Label, StaticText, InputLine
- ListBox, CheckBox, RadioButton, ScrollBar
- MenuBar, StatusLine
- Frame, Window, Dialog
- Editor ✅ (special case - uses app palette indices)

**❌ Components that DO NOT update:**

Components using direct colors from `colors` module:
- Memo (uses `colors::EDITOR_NORMAL` not palette index)
- TextView (hardcoded colors)
- Indicator (hardcoded colors)
- HelpViewer (uses `colors::HELP_NORMAL`)
- Scroller (uses `colors::SCROLLER_NORMAL`)
- ParamText (hardcoded colors)

### Example: Palette Customization

```rust
// Create custom dark palette (63 bytes)
let dark_palette = vec![
    0x08, 0x0F, 0x08, 0x0E, 0x0B, 0x0A, 0x0C, 0x01, // Desktop
    0xF1, 0xE1, 0xF3, 0xF3, 0xF1, 0x08, 0x00,       // Menu
    // ... rest of 63 bytes
];

app.set_palette(Some(dark_palette));

// ✅ Button colors change (uses CP_BUTTON → CP_GRAY_DIALOG → CP_APP_COLOR)
// ✅ Menu colors change (uses CP_MENU_BAR → CP_APP_COLOR)
// ✅ Editor colors change (uses app palette indices 9, 57)
// ❌ Memo colors stay the same (uses colors::EDITOR_NORMAL directly)
```

---

## Recommendations

### For Themeable Components ✅

**Use CP_* palette system:**
1. Implement `get_palette()` to return a CP_* palette
2. Use palette index constants (u8) with `map_color()`
3. Set appropriate `owner_type` (Dialog/Window/None)

### For Non-Themeable Components (OK for specialized cases)

**Use direct colors when:**
- Component is debug/utility (ANSI dump, test viewer)
- Colors are intentionally fixed (help system, indicators)
- Component doesn't need theme support

### Converting to Themeable

To convert a direct-color component to use CP_* palettes:

```rust
// Before: Direct colors
let color = colors::COMPONENT_NORMAL;

// After: Palette mapping
// 1. Define CP_COMPONENT palette
pub const CP_COMPONENT: &[u8] = &[1, 2, 3, 4];

// 2. Implement get_palette()
fn get_palette(&self) -> Option<Palette> {
    Some(Palette::from_slice(palettes::CP_COMPONENT))
}

// 3. Use map_color()
let color = self.map_color(1);  // Maps through CP_COMPONENT → Dialog → App
```

---

## Summary Table

| Component Type | Palette System | Themeable? | Notes |
|---------------|---------------|------------|-------|
| Dialog controls | CP_* palettes | ✅ Yes | Button, Label, InputLine, CheckBox, etc. |
| Frame/Window | CP_* palettes | ✅ Yes | Uses CP_GRAY_DIALOG or CP_BLUE_WINDOW |
| MenuBar | CP_MENU_BAR | ✅ Yes | Top-level component |
| StatusLine | CP_STATUSLINE | ✅ Yes | Top-level component |
| ListBox & variants | CP_LISTBOX | ✅ Yes | All list components |
| Editor | App palette indices | ✅ Yes | Special case - no CP_* but uses palette indices |
| Memo | Direct colors | ❌ No | Uses colors::EDITOR_NORMAL |
| TextView | Direct colors | ❌ No | Hardcoded in draw() |
| HelpViewer | Direct colors | ❌ No | Uses colors::HELP_* |
| Scroller | Direct colors | ❌ No | Uses colors::SCROLLER_* |
| Indicator | Direct colors | ❌ No | Hardcoded in draw() |

**Bottom Line:**
- **Most UI components (85%)** use CP_* palette system → ✅ Fully themeable
- **Specialized components (15%)** use direct colors → ❌ Not themeable
- **Editor is special** → ✅ Themeable via app palette indices

---

*Document created: 2025-11-10*
*Reflects codebase as of v0.10.1*
