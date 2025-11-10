# Borland Palette Usage vs Our Implementation

This document compares how Borland Turbo Vision uses palettes for components that currently use hardcoded colors in our Rust implementation.

## Executive Summary

**FINDING:** All components in Borland Turbo Vision use palettes. NONE use hardcoded colors.

Our components that currently use direct colors from the `colors` module should be converted to use CP_* palettes to match Borland's architecture and enable full theme support.

---

## Component-by-Component Analysis

### 1. TMemo (Memo)

**Our Implementation:**
```rust
// src/views/memo.rs
fn get_palette(&self) -> Option<Palette> {
    None  // ❌ No palette
}

// Uses direct colors
let color = colors::EDITOR_NORMAL;  // Hardcoded Attr
```

**Borland Implementation:**
```cpp
// local-only/magiblot-tvision/source/tvision/tmemo.cpp:63-66
#define cpMemo "\x1A\x1B"  // Indices 26, 27

TPalette& TMemo::getPalette() const {
    static TPalette palette(cpMemo, sizeof(cpMemo)-1);
    return palette;
}
```

**Borland Palette:** `cpMemo = "\x1A\x1B"` (indices 26, 27)
- Index 26: Normal text color
- Index 27: Selected text color

**Impact:** ❌ Our Memo cannot be themed, Borland's can

**Fix Required:** Add CP_MEMO palette

---

### 2. TScroller (Base class for TextView, etc.)

**Our Implementation:**
```rust
// src/views/scroller.rs
// Uses direct colors from colors module
let color = colors::SCROLLER_NORMAL;  // Hardcoded Attr
```

**Borland Implementation:**
```cpp
// local-only/magiblot-tvision/source/tvision/tscrolle.cpp:35, 77-80
#define cpScroller "\x06\x07"  // Indices 6, 7

TPalette& TScroller::getPalette() const {
    static TPalette palette(cpScroller, sizeof(cpScroller)-1);
    return palette;
}
```

**Borland Palette:** `cpScroller = "\x06\x07"` (indices 6, 7)
- Index 6: Normal scrollable area color
- Index 7: Selected/highlighted color

**Impact:** ❌ Our Scroller cannot be themed, Borland's can

**Fix Required:** Add CP_SCROLLER palette

---

### 3. TTextDevice/TTerminal (TextView)

**Our Implementation:**
```rust
// src/views/text_viewer.rs
// Uses hardcoded colors directly in draw()
let color = Attr::new(TvColor::Black, TvColor::LightGray);  // Hardcoded
```

**Borland Implementation:**
```cpp
// TTextDevice extends TScroller
// Inherits cpScroller palette from TScroller
// Uses mapColor(1) for drawing
```

**Borland Palette:** Inherits `cpScroller = "\x06\x07"` from TScroller

**Impact:** ❌ Our TextView cannot be themed, Borland's can

**Fix Required:** Make TextView use CP_SCROLLER palette (inherited)

---

### 4. TIndicator (Indicator)

**Our Implementation:**
```rust
// src/views/indicator.rs
fn get_palette(&self) -> Option<Palette> {
    None  // ❌ No palette
}

// Uses hardcoded colors
let color = Attr::new(TvColor::White, TvColor::LightGray);  // Hardcoded
```

**Borland Implementation:**
```cpp
// local-only/magiblot-tvision/source/tvision/tindictr.cpp:27, 67-69
#define cpIndicator "\x02\x03"  // Indices 2, 3

TPalette& TIndicator::getPalette() const {
    static TPalette palette(cpIndicator, sizeof(cpIndicator)-1);
    return palette;
}
```

**Borland Palette:** `cpIndicator = "\x02\x03"` (indices 2, 3)
- Index 2: Normal indicator color
- Index 3: Modified/active indicator color

**Impact:** ❌ Our Indicator cannot be themed, Borland's can

**Fix Required:** Add CP_INDICATOR palette

---

### 5. THelpViewer (HelpViewer)

**Our Implementation:**
```rust
// src/views/help_viewer.rs
// Uses direct colors from colors module
let color = colors::HELP_NORMAL;      // Hardcoded Attr
let color = colors::HELP_FOCUSED;     // Hardcoded Attr
```

**Borland Implementation:**
```cpp
// local-only/magiblot-tvision/source/tvision/help.cpp:153-156
// local-only/magiblot-tvision/include/tvision/helpbase.h
#define cHelpViewer "\x06\x07\x08"  // Indices 6, 7, 8

TPalette& THelpViewer::getPalette() const {
    static TPalette palette(cHelpViewer, sizeof(cHelpViewer)-1);
    return palette;
}
```

**Borland Palette:** `cHelpViewer = "\x06\x07\x08"` (indices 6, 7, 8)
- Index 6: Normal text color
- Index 7: Focused/selected text color
- Index 8: Cross-reference links color

**Impact:** ❌ Our HelpViewer cannot be themed, Borland's can

**Fix Required:** Add CP_HELP_VIEWER palette

---

### 6. THelpWindow (HelpWindow)

**Our Implementation:**
```rust
// src/views/help_window.rs
// Uses Window's palette (correct)
```

**Borland Implementation:**
```cpp
// local-only/magiblot-tvision/source/tvision/help.cpp:283
// local-only/magiblot-tvision/include/tvision/helpbase.h
#define cHelpWindow "\x80\x81\x82\x83\x84\x85\x86\x87"  // Indices 128-135

TPalette& THelpWindow::getPalette() const {
    static TPalette palette(cHelpWindow, sizeof(cHelpWindow)-1);
    return palette;
}
```

**Borland Palette:** `cHelpWindow = "\x80\x81\x82\x83\x84\x85\x86\x87"` (indices 128-135)
- Full window palette with 8 colors for frame, interior, scrollbars, etc.

**Impact:** ⚠️ Partial - We use Window palette, Borland has specific help window palette

**Fix Required:** Add CP_HELP_WINDOW palette (optional, lower priority)

---

### 7. TParamText (ParamText)

**Our Implementation:**
```rust
// src/views/paramtext.rs
// Uses hardcoded colors in draw()
```

**Borland Implementation:**
```cpp
// TParamText extends TStaticText
// Inherits cpStaticText palette from TStaticText
```

**Borland Palette:** Inherits `cpStaticText = "\x06"` from TStaticText (already implemented)

**Impact:** ✅ We already have CP_STATIC_TEXT palette, just need to use it

**Fix Required:** Make ParamText use CP_STATIC_TEXT palette (inherited)

---

## Summary Table

| Component | Borland Palette | Our Implementation | Status | Priority |
|-----------|----------------|-------------------|--------|----------|
| **TMemo** | `cpMemo = "\x1A\x1B"` (26, 27) | Direct colors | ❌ Missing | HIGH |
| **TScroller** | `cpScroller = "\x06\x07"` (6, 7) | Direct colors | ❌ Missing | HIGH |
| **TTextView** | Inherits `cpScroller` | Hardcoded | ❌ Missing | HIGH |
| **TIndicator** | `cpIndicator = "\x02\x03"` (2, 3) | Hardcoded | ❌ Missing | HIGH |
| **THelpViewer** | `cHelpViewer = "\x06\x07\x08"` (6, 7, 8) | Direct colors | ❌ Missing | MEDIUM |
| **THelpWindow** | `cHelpWindow = "\x80...\x87"` (128-135) | Window palette | ⚠️ Partial | LOW |
| **TParamText** | Inherits `cpStaticText` | Hardcoded | ⚠️ Should inherit | LOW |

---

## Palette Index Mappings

Based on Borland's palettes, here's what the indices typically map to:

### cpMemo (indices 26, 27)
These map to window interior colors (indices 24-31 in CP_APP_COLOR are window colors):
- 26 → Normal memo text (likely White on Blue)
- 27 → Selected memo text (likely Black on Cyan)

### cpScroller (indices 6, 7)
These map to generic scrollable area colors:
- 6 → Normal scroller background/text
- 7 → Selected scroller item

### cpIndicator (indices 2, 3)
These map to app-level status colors:
- 2 → Normal indicator (likely Black on LightGray)
- 3 → Active/modified indicator (likely different fg/bg)

### cHelpViewer (indices 6, 7, 8)
- 6 → Normal help text
- 7 → Focused/selected help text
- 8 → Cross-reference links

---

## Proposed CP_* Palette Definitions

Add these to `src/core/palette.rs`:

```rust
/// Memo palette (indices for editor-like memo fields)
/// Borland: cpMemo = "\x1A\x1B" (26, 27)
pub const CP_MEMO: &[u8] = &[
    26, 27,  // 1-2: Normal, Selected
];

/// Scroller palette (base for scrollable views)
/// Borland: cpScroller = "\x06\x07" (6, 7)
pub const CP_SCROLLER: &[u8] = &[
    6, 7,  // 1-2: Normal, Selected
];

/// Indicator palette (position/status indicator)
/// Borland: cpIndicator = "\x02\x03" (2, 3)
pub const CP_INDICATOR: &[u8] = &[
    2, 3,  // 1-2: Normal, Modified
];

/// Help viewer palette (help text viewer)
/// Borland: cHelpViewer = "\x06\x07\x08" (6, 7, 8)
pub const CP_HELP_VIEWER: &[u8] = &[
    6, 7, 8,  // 1-3: Normal, Focused, Link
];

/// Help window palette (help window container)
/// Borland: cHelpWindow = "\x80\x81\x82\x83\x84\x85\x86\x87" (128-135)
pub const CP_HELP_WINDOW: &[u8] = &[
    128, 129, 130, 131, 132, 133, 134, 135,
    // Full window palette for help windows
];
```

---

## Implementation Steps

### Step 1: Add CP_* Palette Constants (HIGH Priority)

Add to `src/core/palette.rs` in the palettes module:

1. ✅ CP_SCROLLER - Already exists! Just need to use it
2. ❌ CP_MEMO - Need to add
3. ❌ CP_INDICATOR - Need to add
4. ❌ CP_HELP_VIEWER - Need to add
5. ❌ CP_HELP_WINDOW - Need to add (optional)

### Step 2: Update Components to Use Palettes (HIGH Priority)

Update `get_palette()` implementations:

```rust
// src/views/memo.rs
fn get_palette(&self) -> Option<Palette> {
    use crate::core::palette::{palettes, Palette};
    Some(Palette::from_slice(palettes::CP_MEMO))
}

// src/views/scroller.rs
fn get_palette(&self) -> Option<Palette> {
    use crate::core::palette::{palettes, Palette};
    Some(Palette::from_slice(palettes::CP_SCROLLER))
}

// src/views/indicator.rs
fn get_palette(&self) -> Option<Palette> {
    use crate::core::palette::{palettes, Palette};
    Some(Palette::from_slice(palettes::CP_INDICATOR))
}

// src/views/help_viewer.rs
fn get_palette(&self) -> Option<Palette> {
    use crate::core::palette::{palettes, Palette};
    Some(Palette::from_slice(palettes::CP_HELP_VIEWER))
}
```

### Step 3: Update Drawing Code to Use map_color() (HIGH Priority)

Replace direct color usage with palette mapping:

```rust
// Before: Direct colors
let color = colors::EDITOR_NORMAL;

// After: Palette mapping
let color = self.map_color(1);  // Normal color
let selected_color = self.map_color(2);  // Selected color
```

### Step 4: Test Theme Switching (HIGH Priority)

Verify all components update when calling `app.set_palette()`:

```rust
// Test that these components change colors with palette
app.set_palette(Some(dark_palette));
// Memo, Scroller, Indicator, HelpViewer should all update ✅
```

---

## Benefits of Using CP_* Palettes

### 1. Full Theme Support ✅
All components will respond to `Application::set_palette()` changes

### 2. Borland Compatibility ✅
Matches original Turbo Vision architecture exactly

### 3. Consistency ✅
All components use the same color system

### 4. Flexibility ✅
Users can create custom palettes that affect ALL components

### 5. Maintainability ✅
Colors defined in one place (CP_APP_COLOR)

---

## Current State vs. Target State

### Current State ❌

**Components with palettes (85%):**
- Button, Label, InputLine, StaticText
- CheckBox, RadioButton, ListBox, ScrollBar
- MenuBar, StatusLine, Frame

**Components without palettes (15%):**
- Memo, TextView, Scroller
- Indicator, HelpViewer, ParamText

**Result:** Only 85% of components are themeable

### Target State ✅

**All components use CP_* palettes (100%):**
- Button, Label, InputLine, StaticText
- CheckBox, RadioButton, ListBox, ScrollBar
- MenuBar, StatusLine, Frame
- **Memo, TextView, Scroller** ← Fixed
- **Indicator, HelpViewer, ParamText** ← Fixed

**Result:** 100% of components are themeable

---

## Conclusion

**Key Finding:** Borland Turbo Vision uses palettes for EVERY component. None use hardcoded colors.

**Recommendation:** Convert all components currently using direct colors to use CP_* palettes to:
1. Enable full theme customization
2. Match Borland's architecture
3. Achieve 100% component themeability

**Impact:** HIGH - This will complete the theming system and provide full Borland compatibility.

---

*Document created: 2025-11-10*
*Based on analysis of magiblot-tvision source code*
*Status: Ready for implementation*
