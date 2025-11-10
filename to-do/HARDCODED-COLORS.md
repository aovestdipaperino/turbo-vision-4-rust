# Hardcoded Colors Report

This document identifies all instances where hardcoded attributes are used instead of proper palette indexing in the turbo-vision codebase.

## Summary

- **Total instances found:** 69
- **Files affected:** 16
- **Primary issue:** Direct use of `Attr::new()` with hardcoded `TvColor` values instead of palette lookups
- **Correct usage found:** 1 instance using `colors::EDITOR_NORMAL` in `terminal_widget.rs:319`

## Severity Classification

### Critical (Production Code)
Files that directly impact user-visible UI and should be fixed for proper theming support.

### Test/Debug Code
Files that are part of testing infrastructure - lower priority.

---

## Detailed Findings by File

### Critical Issues

#### 1. src/views/window.rs (3 instances)
Window and dialog interior colors are hardcoded in constructors:

- **Line 55:** `Attr::new(TvColor::Yellow, TvColor::Blue)` - Window title color in `new()`
- **Line 68:** `Attr::new(TvColor::Black, TvColor::LightGray)` - Dialog title color in `new_for_dialog()`
- **Line 640:** `Attr::new(TvColor::Yellow, TvColor::Blue)` - Unknown context

**Impact:** Windows and dialogs cannot be themed properly.

**Recommended Fix:** Use palette-based colors from `colors::` module based on window type.

---

#### 2. src/views/frame.rs (10 instances)
Frame drawing with hardcoded colors for different states:

**Dialog Frame Colors:**
- **Line 75:** `Attr::new(TvColor::DarkGray, TvColor::LightGray)` - Inactive frame
- **Line 80:** `Attr::new(TvColor::LightGreen, TvColor::LightGray)` - Dragging state
- **Line 87:** `Attr::new(TvColor::White, TvColor::LightGray)` - Active frame
- **Line 88:** `Attr::new(TvColor::LightGreen, TvColor::LightGray)` - Close icon
- **Line 89:** `Attr::new(TvColor::White, TvColor::LightGray)` - Title

**Editor Frame Colors:**
- **Line 100:** `Attr::new(TvColor::LightGreen, TvColor::Blue)` - Inactive frame
- **Line 104:** `Attr::new(TvColor::LightGreen, TvColor::LightGray)` - Dragging state
- **Line 108:** `Attr::new(TvColor::White, TvColor::Blue)` - Active frame
- **Line 109:** `Attr::new(TvColor::LightGreen, TvColor::Blue)` - Close icon
- **Line 110:** `Attr::new(TvColor::Yellow, TvColor::Blue)` - Title

**Impact:** Frame colors in `get_frame_colors()` method cannot be themed.

**Recommended Fix:** Define palette constants for frame states:
- `FRAME_DIALOG_ACTIVE`, `FRAME_DIALOG_INACTIVE`, `FRAME_DIALOG_DRAGGING`
- `FRAME_EDITOR_ACTIVE`, `FRAME_EDITOR_INACTIVE`, `FRAME_EDITOR_DRAGGING`
- `FRAME_CLOSE_ICON`, `FRAME_TITLE`

---

#### 3. src/views/editor.rs (3 instances)
Editor text rendering with hardcoded colors:

- **Line 1090:** `Attr::new(TvColor::White, TvColor::Blue)` - Default editor color
- **Line 1176:** `Attr::new(TvColor::Black, TvColor::Cyan)` - Selection highlight
- **Line 1206:** `Attr::new(TvColor::Black, TvColor::Cyan)` - Cursor color

**Impact:** Editor cannot be themed.

**Recommended Fix:** Use existing `colors::EDITOR_NORMAL` and add `colors::EDITOR_SELECTED` and `colors::EDITOR_CURSOR`.

---

#### 4. src/views/syntax.rs (13 instances)
Syntax highlighting token colors all hardcoded:

- **Line 48:** `Attr::new(TvColor::LightGray, TvColor::Blue)` - Normal text
- **Line 49:** `Attr::new(TvColor::Yellow, TvColor::Blue)` - Keywords
- **Line 50:** `Attr::new(TvColor::LightRed, TvColor::Blue)` - Strings
- **Line 51:** `Attr::new(TvColor::LightCyan, TvColor::Blue)` - Comments
- **Line 52:** `Attr::new(TvColor::LightMagenta, TvColor::Blue)` - Numbers
- **Line 53:** `Attr::new(TvColor::White, TvColor::Blue)` - Operators
- **Line 54:** `Attr::new(TvColor::LightGray, TvColor::Blue)` - Identifiers
- **Line 55:** `Attr::new(TvColor::LightGreen, TvColor::Blue)` - Type names
- **Line 56:** `Attr::new(TvColor::LightCyan, TvColor::Blue)` - Preprocessor
- **Line 57:** `Attr::new(TvColor::Cyan, TvColor::Blue)` - Functions
- **Line 58:** `Attr::new(TvColor::White, TvColor::Blue)` - Special
- **Line 322-323:** `Attr::new(TvColor::Black, TvColor::Black)` - Test assertions

**Impact:** Syntax highlighting theme cannot be customized.

**Recommended Fix:** Define palette constants for each token type:
- `SYNTAX_KEYWORD`, `SYNTAX_STRING`, `SYNTAX_COMMENT`, `SYNTAX_NUMBER`, etc.

---

#### 5. src/views/desktop.rs (1 instance)

- **Line 29:** `Attr::new(TvColor::LightGray, TvColor::DarkGray)` - Desktop background

**Impact:** Desktop background cannot be themed.

**Recommended Fix:** Add `colors::DESKTOP_BACKGROUND` constant.

---

#### 6. src/views/text_viewer.rs (3 instances)

- **Line 246:** `Attr::new(TvColor::Black, TvColor::LightGray)` - Background fill
- **Line 255:** `Attr::new(TvColor::White, TvColor::LightGray)` - Line numbers
- **Line 265:** `Attr::new(TvColor::Black, TvColor::LightGray)` - Text content

**Impact:** Text viewer cannot be themed.

**Recommended Fix:** Add palette constants for text viewer components.

---

#### 7. src/views/memo.rs (2 instances)

- **Line 570:** `Attr::new(TvColor::White, TvColor::Blue)` - Default memo color
- **Line 624:** `Attr::new(TvColor::Black, TvColor::Cyan)` - Cursor highlight

**Impact:** Memo widget cannot be themed.

**Recommended Fix:** Use `colors::EDITOR_NORMAL` and `colors::EDITOR_CURSOR`.

---

#### 8. src/views/indicator.rs (2 instances)

- **Line 61:** `Attr::new(TvColor::White, TvColor::LightGray)` - Indicator background
- **Line 62:** `Attr::new(TvColor::White, TvColor::LightGray)` - Indicator text

**Impact:** Position indicator cannot be themed.

**Recommended Fix:** Add `colors::INDICATOR_NORMAL` constant.

---

### List Component Pattern (Repeated Across Multiple Files)

The following files all use the same hardcoded color pattern for list items:
- Focused: `Black on White`
- Normal: `Black on LightGray`
- Selected: `White on Cyan`
- Selected unfocused: `White on Blue`

#### 9. src/views/dir_listbox.rs (3 instances)
- **Line 300:** Focused item
- **Line 302:** Normal item
- **Line 306:** Empty line

#### 10. src/views/file_list.rs (3 instances)
- **Line 288:** Focused item
- **Line 290:** Normal item
- **Line 294:** Empty line

#### 11. src/views/sorted_listbox.rs (4 instances)
- **Line 257:** Focused item
- **Line 259:** Normal item
- **Line 262:** Selected item
- **Line 264:** Selected unfocused

#### 12. src/views/history_viewer.rs (4 instances)
- **Line 88:** Focused item
- **Line 90:** Normal item
- **Line 93:** Selected item
- **Line 95:** Selected unfocused

#### 13. src/views/help_viewer.rs (2 instances)
- **Line 155:** Focused item
- **Line 157:** Normal item

#### 14. src/views/history.rs (2 instances)
- **Line 92:** Focused history entry (uses Green background)
- **Line 94:** Normal history entry (uses Green background)

#### 15. src/views/outline.rs (4 instances)
- **Line 301:** Focused outline item
- **Line 303:** Normal outline item
- **Line 306:** Selected item
- **Line 308:** Selected unfocused

**Impact:** All list components use identical hardcoded colors, preventing theming.

**Recommended Fix:** Consolidate into shared palette constants:
- `colors::LISTBOX_NORMAL`
- `colors::LISTBOX_FOCUSED`
- `colors::LISTBOX_SELECTED`
- `colors::LISTBOX_SELECTED_UNFOCUSED`

---

### Test/Debug Code (Lower Priority)

#### 16. src/core/ansi_dump.rs (2 instances)
- **Line 206-207:** Test cells with `White on Blue`

**Context:** ANSI dump utility for testing/debugging.

---

#### 17. src/core/draw.rs (2 instances)
- **Line 125, 137:** Unit test attributes with `White on Black`

**Context:** DrawBuffer test methods.

---

#### 18. src/test_util.rs (5 instances)
- **Line 45:** Documentation comment example
- **Line 64, 211:** Test utilities with `LightGray on Black`
- **Line 270, 333:** Test functions with `White on Blue`

**Context:** Test helper utilities.

---

## Correct Usage Example

#### src/views/terminal_widget.rs:319
```rust
let default_color = colors::EDITOR_NORMAL;
```

**This is the correct approach** - using palette constants from the `colors::` module.

---

## Common Patterns Identified

### Pattern 1: List Item States (12+ files)
Multiple components duplicate this exact pattern:
```rust
let attr = if focused {
    Attr::new(TvColor::Black, TvColor::White)
} else {
    Attr::new(TvColor::Black, TvColor::LightGray)
};
```

### Pattern 2: Editor/Text Components (3 files)
```rust
let default_attr = Attr::new(TvColor::White, TvColor::Blue);
let selection_attr = Attr::new(TvColor::Black, TvColor::Cyan);
```

### Pattern 3: Frame State Colors (frame.rs)
Different colors for active/inactive/dragging states, hardcoded per palette type.

---

## Recommendations

### Priority 1: List Component Consolidation (High Impact)
**Files:** 8 list-related files
**Instances:** 28
**Action:** Define shared constants:
```rust
pub const LISTBOX_NORMAL: Attr = Attr::new(...);
pub const LISTBOX_FOCUSED: Attr = Attr::new(...);
pub const LISTBOX_SELECTED: Attr = Attr::new(...);
pub const LISTBOX_SELECTED_UNFOCUSED: Attr = Attr::new(...);
```

### Priority 2: Editor/Text Components (High Impact)
**Files:** editor.rs, memo.rs, text_viewer.rs
**Instances:** 8
**Action:** Extend existing `colors::EDITOR_NORMAL` with:
```rust
pub const EDITOR_SELECTED: Attr = Attr::new(...);
pub const EDITOR_CURSOR: Attr = Attr::new(...);
pub const TEXT_VIEWER_NORMAL: Attr = Attr::new(...);
pub const TEXT_VIEWER_LINE_NUMBERS: Attr = Attr::new(...);
```

### Priority 3: Frame Colors (Medium Impact)
**Files:** frame.rs
**Instances:** 10
**Action:** Replace `get_frame_colors()` hardcoded values with palette lookups.

### Priority 4: Syntax Highlighting (Medium Impact)
**Files:** syntax.rs
**Instances:** 13
**Action:** Define syntax color constants in palette module to enable syntax theme customization.

### Priority 5: Miscellaneous Components (Medium Impact)
**Files:** window.rs, desktop.rs, indicator.rs
**Instances:** 6
**Action:** Define component-specific palette constants.

### Priority 6: Test Code (Low Impact)
**Files:** ansi_dump.rs, draw.rs, test_util.rs
**Instances:** 9
**Action:** Can remain hardcoded or be updated last for consistency.

---

## Benefits of Fixing

1. **True Theme Support:** Users can customize all colors through the palette system
2. **Consistency:** Eliminate duplicate hardcoded values across components
3. **Maintainability:** Centralized color definitions easier to update
4. **Accessibility:** Easier to create high-contrast or color-blind friendly themes
5. **Flexibility:** Components automatically adapt to palette changes

---

## Next Steps

1. Review and approve this report
2. Create palette constants for each identified use case
3. Systematically replace hardcoded attributes with palette lookups
4. Test theme switching across all affected components
5. Update documentation on proper palette usage

---

*Report generated: 2025-11-10*
*Total hardcoded attributes: 69 instances across 16 files*
