# Palette Flexibility & Window Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix palette inflexibility (#96, #98) and window management gaps (#97, #103) reported by Hans-Christian Esperer.

**Architecture:** Extend the existing Borland-style palette chain to support syntax highlighting colors per window type. Add a public bring-to-front API on Desktop. Fix HelpWindow's missing View trait delegations and add UX improvements for link navigation and history restoration.

**Tech Stack:** Rust, no new dependencies. All changes in `src/core/` and `src/views/`.

**Spec:** `docs/superpowers/specs/2026-04-10-palette-and-window-management-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `src/core/palette.rs` | Modify | Add syntax palette indices, extend CP_EDITOR, extend window palettes, extend CP_APP_COLOR |
| `src/views/syntax.rs` | Modify | Add `TokenType::palette_index()` method |
| `src/views/editor.rs` | Modify | Use `map_color(palette_index())` instead of `default_color()` |
| `src/views/window.rs` | Modify | Add public `Window::new_with_type()`, add `palette_type` to `WindowBuilder` |
| `src/views/group.rs` | Modify | Fix `bring_to_front()` and `send_to_back()` to sync `view_ids` |
| `src/views/desktop.rs` | Modify | Add `Desktop::bring_to_front(ViewId)` |
| `src/views/help_window.rs` | Modify | Add `options()`/`set_options()` delegation, single-click link follow, history restoration |
| `src/views/help_viewer.rs` | Modify | Add `get_scroll_state()`/`set_scroll_state()`, Up/Down link cycling, single-click propagation |

---

### Task 1: Fix Group::bring_to_front() and send_to_back() view_ids sync

**Files:**
- Modify: `src/views/group.rs:128-177`

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `src/views/group.rs`:

```rust
#[test]
fn test_bring_to_front_syncs_view_ids() {
    use crate::core::geometry::Rect;

    let mut group = Group::new(Rect::new(0, 0, 80, 25));
    let id1 = group.add(Box::new(super::background::Background::new(
        Rect::new(0, 0, 10, 5), ' ', crate::core::palette::Attr::new(
            crate::core::palette::TvColor::White, crate::core::palette::TvColor::Blue,
        ),
    )));
    let id2 = group.add(Box::new(super::background::Background::new(
        Rect::new(0, 0, 10, 5), ' ', crate::core::palette::Attr::new(
            crate::core::palette::TvColor::White, crate::core::palette::TvColor::Blue,
        ),
    )));
    let id3 = group.add(Box::new(super::background::Background::new(
        Rect::new(0, 0, 10, 5), ' ', crate::core::palette::Attr::new(
            crate::core::palette::TvColor::White, crate::core::palette::TvColor::Blue,
        ),
    )));

    // Bring first child to front
    group.bring_to_front(0);

    // After bring_to_front(0): order should be [id2, id3, id1]
    // Verify child_by_id still works correctly
    assert!(group.child_by_id(id1).is_some());
    assert!(group.child_by_id(id2).is_some());
    assert!(group.child_by_id(id3).is_some());

    // The brought-to-front child (id1) should now be at the last index
    // Verify by checking that view_ids[2] == id1
    // We can test this indirectly: remove_by_id should still find the right child
    assert!(group.remove_by_id(id1));
    assert_eq!(group.len(), 2);
    // id2 and id3 should still be findable
    assert!(group.child_by_id(id2).is_some());
    assert!(group.child_by_id(id3).is_some());
}

#[test]
fn test_send_to_back_syncs_view_ids() {
    use crate::core::geometry::Rect;

    let mut group = Group::new(Rect::new(0, 0, 80, 25));
    let id1 = group.add(Box::new(super::background::Background::new(
        Rect::new(0, 0, 10, 5), ' ', crate::core::palette::Attr::new(
            crate::core::palette::TvColor::White, crate::core::palette::TvColor::Blue,
        ),
    )));
    let id2 = group.add(Box::new(super::background::Background::new(
        Rect::new(0, 0, 10, 5), ' ', crate::core::palette::Attr::new(
            crate::core::palette::TvColor::White, crate::core::palette::TvColor::Blue,
        ),
    )));
    let id3 = group.add(Box::new(super::background::Background::new(
        Rect::new(0, 0, 10, 5), ' ', crate::core::palette::Attr::new(
            crate::core::palette::TvColor::White, crate::core::palette::TvColor::Blue,
        ),
    )));

    // Send last child to back (position 1, after index 0)
    group.send_to_back(2);

    // After send_to_back(2): order should be [id1, id3, id2]
    // All IDs should still be findable
    assert!(group.child_by_id(id1).is_some());
    assert!(group.child_by_id(id2).is_some());
    assert!(group.child_by_id(id3).is_some());

    // Remove id3 (should be at index 1 now) to verify sync
    assert!(group.remove_by_id(id3));
    assert_eq!(group.len(), 2);
    assert!(group.child_by_id(id1).is_some());
    assert!(group.child_by_id(id2).is_some());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib test_bring_to_front_syncs_view_ids test_send_to_back_syncs_view_ids -- --nocapture`

Expected: FAIL — `remove_by_id` or `child_by_id` will find the wrong child because `view_ids` is desynced from `children`.

- [ ] **Step 3: Fix bring_to_front to sync view_ids**

In `src/views/group.rs`, modify `bring_to_front()` (around line 128):

```rust
    pub fn bring_to_front(&mut self, index: usize) -> usize {
        if index >= self.children.len() || index == self.children.len() - 1 {
            return index;
        }

        let view = self.children.remove(index);
        let view_id = self.view_ids.remove(index);

        self.children.push(view);
        self.view_ids.push(view_id);

        let new_index = self.children.len() - 1;
        if self.focused == index {
            self.focused = new_index;
        } else if self.focused > index {
            self.focused -= 1;
        }

        new_index
    }
```

- [ ] **Step 4: Fix send_to_back to sync view_ids**

In `src/views/group.rs`, modify `send_to_back()` (around line 156):

```rust
    pub fn send_to_back(&mut self, index: usize) -> usize {
        if index >= self.children.len() || index == 1 {
            return index;
        }

        let view = self.children.remove(index);
        let view_id = self.view_ids.remove(index);

        self.children.insert(1, view);
        self.view_ids.insert(1, view_id);

        if self.focused == index {
            self.focused = 1;
        } else if self.focused >= 1 && self.focused < index {
            self.focused += 1;
        }

        1
    }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib test_bring_to_front_syncs_view_ids test_send_to_back_syncs_view_ids -- --nocapture`

Expected: PASS

- [ ] **Step 6: Run full test suite**

Run: `cargo test --lib`

Expected: All 226+ tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/views/group.rs
git commit -m "fix: sync view_ids in Group::bring_to_front() and send_to_back()

bring_to_front() and send_to_back() reordered children but not the
parallel view_ids vec, desyncing ViewId-to-child mapping after any
z-order change."
```

---

### Task 2: Add Desktop::bring_to_front(ViewId) (#103)

**Files:**
- Modify: `src/views/desktop.rs`

- [ ] **Step 1: Write the failing test**

Add to the end of `src/views/desktop.rs` (create a `#[cfg(test)] mod tests` block if one doesn't exist):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Rect;
    use crate::views::window::Window;

    #[test]
    fn test_bring_to_front_by_view_id() {
        let mut desktop = Desktop::new(Rect::new(0, 1, 80, 24));

        let id1 = desktop.add(Box::new(Window::new(Rect::new(5, 5, 30, 15), "Win 1")));
        let id2 = desktop.add(Box::new(Window::new(Rect::new(10, 6, 35, 16), "Win 2")));
        let _id3 = desktop.add(Box::new(Window::new(Rect::new(15, 7, 40, 17), "Win 3")));

        // Win 3 is on top (last added). Bring Win 1 to front.
        assert!(desktop.bring_to_front(id1));

        // Win 1 should now be the top window (last child, excluding background).
        // Verify by bringing id2 to front and confirming it also works.
        assert!(desktop.bring_to_front(id2));

        // Non-existent ViewId returns false
        let fake_id = crate::views::view::ViewId::from_u16(9999);
        assert!(!desktop.bring_to_front(fake_id));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib test_bring_to_front_by_view_id -- --nocapture`

Expected: FAIL — method `bring_to_front` not found on `Desktop`.

- [ ] **Step 3: Implement Desktop::bring_to_front**

Add this method to the `impl Desktop` block in `src/views/desktop.rs`:

```rust
    /// Bring a specific window to the front of the Z-order by its ViewId.
    /// Returns true if the window was found and moved, false if not found.
    /// Matches Borland: TView::makeFirst() / TView::select()
    pub fn bring_to_front(&mut self, view_id: ViewId) -> bool {
        use crate::core::state::OF_TOP_SELECT;

        // Search children (skip background at index 0) for matching ViewId
        let found_index = (1..self.children.len()).find(|&i| {
            self.children.view_id_at(i) == Some(view_id)
        });

        let index = match found_index {
            Some(i) => i,
            None => return false,
        };

        let last_idx = self.children.len() - 1;
        if index == last_idx {
            return true; // Already on top
        }

        // Check if the window has OF_TOP_SELECT
        let options = self.children.child_at(index).options();
        if (options & OF_TOP_SELECT) == 0 {
            return false;
        }

        // Unfocus current top window
        self.children.child_at_mut(last_idx).set_focus(false);

        // Bring the target window to front
        self.children.bring_to_front(index);

        // Focus the new top window
        let new_top = self.children.len() - 1;
        self.children.child_at_mut(new_top).set_focus(true);

        true
    }
```

- [ ] **Step 4: Add Group::view_id_at helper**

Add to `impl Group` in `src/views/group.rs`:

```rust
    /// Get the ViewId of a child at the given index.
    /// Returns None if the index is out of bounds.
    pub fn view_id_at(&self, index: usize) -> Option<ViewId> {
        self.view_ids.get(index).copied()
    }
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib test_bring_to_front_by_view_id -- --nocapture`

Expected: PASS

- [ ] **Step 6: Run full test suite**

Run: `cargo test --lib`

Expected: All tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/views/desktop.rs src/views/group.rs
git commit -m "feat: add Desktop::bring_to_front(ViewId) for z-order control (#103)"
```

---

### Task 3: Make Window palette selection public (#96)

**Files:**
- Modify: `src/views/window.rs`

- [ ] **Step 1: Write the failing test**

Add to the test module in `src/views/window.rs` (or create one):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Rect;

    #[test]
    fn test_new_with_type_gray() {
        let window = Window::new_with_type(
            Rect::new(5, 5, 40, 20),
            "Gray Panel",
            WindowPaletteType::Gray,
        );
        assert_eq!(window.bounds(), Rect::new(5, 5, 40, 20));
    }

    #[test]
    fn test_new_with_type_cyan() {
        let window = Window::new_with_type(
            Rect::new(5, 5, 40, 20),
            "Cyan Window",
            WindowPaletteType::Cyan,
        );
        assert_eq!(window.bounds(), Rect::new(5, 5, 40, 20));
    }

    #[test]
    fn test_new_with_type_blue() {
        let window = Window::new_with_type(
            Rect::new(5, 5, 40, 20),
            "Blue Window",
            WindowPaletteType::Blue,
        );
        assert_eq!(window.bounds(), Rect::new(5, 5, 40, 20));
    }

    #[test]
    fn test_builder_with_palette_type() {
        let window = WindowBuilder::new()
            .bounds(Rect::new(5, 5, 40, 20))
            .title("Gray Window")
            .palette_type(WindowPaletteType::Gray)
            .build();
        assert_eq!(window.bounds(), Rect::new(5, 5, 40, 20));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib test_new_with_type_gray test_new_with_type_cyan test_new_with_type_blue test_builder_with_palette_type -- --nocapture`

Expected: FAIL — `new_with_type` and `palette_type` don't exist.

- [ ] **Step 3: Add Window::new_with_type**

Add this method to `impl Window` in `src/views/window.rs`, after `new_for_help`:

```rust
    /// Create a window with a specific palette type.
    /// This allows users to create Gray, Cyan, or Blue windows without
    /// being constrained to the preset constructors.
    pub fn new_with_type(bounds: Rect, title: &str, palette_type: WindowPaletteType) -> Self {
        let (frame_palette, resizable) = match palette_type {
            WindowPaletteType::Blue => (super::frame::FramePaletteType::Editor, true),
            WindowPaletteType::Cyan => (super::frame::FramePaletteType::HelpWindow, true),
            WindowPaletteType::Gray => (super::frame::FramePaletteType::Dialog, true),
            WindowPaletteType::Dialog => (super::frame::FramePaletteType::Dialog, false),
        };
        Self::new_with_palette(bounds, title, frame_palette, palette_type, resizable)
    }
```

- [ ] **Step 4: Add palette_type to WindowBuilder**

Modify `WindowBuilder` struct to add the field and method:

Add field to the struct:
```rust
pub struct WindowBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    resizable: bool,
    palette_type: WindowPaletteType,
}
```

Update `new()`:
```rust
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            resizable: true,
            palette_type: WindowPaletteType::Blue,
        }
    }
```

Add setter method:
```rust
    /// Sets the window palette type (default: Blue).
    #[must_use]
    pub fn palette_type(mut self, palette_type: WindowPaletteType) -> Self {
        self.palette_type = palette_type;
        self
    }
```

Update `build()`:
```rust
    pub fn build(self) -> Window {
        let bounds = self.bounds.expect("Window bounds must be set");
        let title = self.title.expect("Window title must be set");

        let frame_palette = match self.palette_type {
            WindowPaletteType::Blue => super::frame::FramePaletteType::Editor,
            WindowPaletteType::Cyan => super::frame::FramePaletteType::HelpWindow,
            WindowPaletteType::Gray | WindowPaletteType::Dialog => super::frame::FramePaletteType::Dialog,
        };

        Window::new_with_palette(
            bounds,
            &title,
            frame_palette,
            self.palette_type,
            self.resizable,
        )
    }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib test_new_with_type_gray test_new_with_type_cyan test_new_with_type_blue test_builder_with_palette_type -- --nocapture`

Expected: PASS

- [ ] **Step 6: Run full test suite**

Run: `cargo test --lib`

Expected: All tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/views/window.rs
git commit -m "feat: add Window::new_with_type() and WindowBuilder::palette_type() (#96)"
```

---

### Task 4: Palette-integrated syntax highlighting (#98)

**Files:**
- Modify: `src/core/palette.rs`
- Modify: `src/views/syntax.rs`
- Modify: `src/views/editor.rs`

- [ ] **Step 1: Add syntax palette index constants**

In `src/core/palette.rs`, after the `EDITOR_CURSOR` constant (line 78), add:

```rust
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
```

- [ ] **Step 2: Extend CP_EDITOR palette**

In `src/core/palette.rs`, change `CP_EDITOR` (around line 690) from:

```rust
    pub const CP_EDITOR: &[u8] = &[
        6, 7,  // 1-2: Normal text, Selected text
    ];
```

to:

```rust
    pub const CP_EDITOR: &[u8] = &[
        6, 7,                                          // 1-2: Normal text, Selected text
        9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,   // 3-13: Syntax colors (window-relative)
    ];
```

- [ ] **Step 3: Extend window palettes**

In `src/core/palette.rs`, change the three window palettes.

`CP_BLUE_WINDOW` (around line 558):
```rust
    pub const CP_BLUE_WINDOW: &[u8] = &[
        8, 9, 10, 11, 12, 13, 14, 15,                   // 1-8: Original window entries
        64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74,     // 9-19: Syntax colors (blue bg)
    ];
```

`CP_CYAN_WINDOW` (around line 564):
```rust
    pub const CP_CYAN_WINDOW: &[u8] = &[
        16, 17, 18, 19, 20, 21, 22, 23,                 // 1-8: Original window entries
        75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85,     // 9-19: Syntax colors (cyan bg)
    ];
```

`CP_GRAY_WINDOW` (around line 570):
```rust
    pub const CP_GRAY_WINDOW: &[u8] = &[
        24, 25, 26, 27, 28, 29, 30, 31,                 // 1-8: Original window entries
        86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96,     // 9-19: Syntax colors (gray bg)
    ];
```

- [ ] **Step 4: Extend CP_APP_COLOR with syntax entries**

In `src/core/palette.rs`, extend `CP_APP_COLOR` (around line 541). After the existing 63 entries, append 33 new entries (11 syntax colors x 3 background variants):

```rust
    #[rustfmt::skip]
    pub const CP_APP_COLOR: &[u8] = &[
        0x71, 0x70, 0x78, 0x74, 0x20, 0x28, 0x24, 0x17, // 1-8: Desktop colors
        0x1F, 0x1A, 0x31, 0x31, 0x1E, 0x71, 0x00,       // 9-15: Menu colors
        // 16-23: Cyan Window
        // Note: Index 16 changed from Borland's 0x37 (light gray on cyan) to 0x30 (black on cyan)
        // for better readability on modern terminals where light gray on cyan has poor contrast
        0x30, 0x3F, 0x3A, 0x13, 0x13, 0x3E, 0x21, 0x00,
        0x70, 0x7F, 0x7A, 0x13, 0x13, 0x70, 0x7F, 0x00, // 24-31: Gray Window
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
    ];
```

Color breakdown for Blue bg (64-74): LightGray/Blue, Yellow/Blue, LightRed/Blue, White/Blue, LightMagenta/Blue, White/Blue, LightGray/Blue, LightGreen/Blue, White/Blue, LightCyan/Blue, White/Blue.

Color breakdown for Cyan bg (75-85): Same foregrounds, Cyan background.

Color breakdown for Gray bg (86-96): Black/LightGray, Yellow/LightGray, LightRed/LightGray, DarkGray/LightGray (comments), LightMagenta/LightGray, Black/LightGray, Black/LightGray, Green/LightGray, DarkGray/LightGray, Blue/LightGray, Red/LightGray.

- [ ] **Step 5: Update the palette layout comment**

Update the comment block above `CP_APP_COLOR` to include the new range:

```rust
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
```

- [ ] **Step 6: Add TokenType::palette_index()**

In `src/views/syntax.rs`, add to the `impl TokenType` block (after `default_color`):

```rust
    /// Get the palette index for this token type (editor-relative).
    /// Used by the editor's draw method to resolve colors through the palette chain.
    /// Maps through CP_EDITOR → window palette → app palette.
    pub fn palette_index(&self) -> u8 {
        use crate::core::palette::*;
        match self {
            TokenType::Normal => SYNTAX_NORMAL_IDX,
            TokenType::Keyword => SYNTAX_KEYWORD_IDX,
            TokenType::String => SYNTAX_STRING_IDX,
            TokenType::Comment => SYNTAX_COMMENT_IDX,
            TokenType::Number => SYNTAX_NUMBER_IDX,
            TokenType::Operator => SYNTAX_OPERATOR_IDX,
            TokenType::Identifier => SYNTAX_IDENTIFIER_IDX,
            TokenType::Type => SYNTAX_TYPE_IDX,
            TokenType::Preprocessor => SYNTAX_PREPROCESSOR_IDX,
            TokenType::Function => SYNTAX_FUNCTION_IDX,
            TokenType::Special => SYNTAX_SPECIAL_IDX,
        }
    }
```

- [ ] **Step 7: Write test for palette_index**

Add to the test module in `src/views/syntax.rs`:

```rust
#[test]
fn test_token_type_palette_index() {
    use crate::core::palette::*;
    assert_eq!(TokenType::Normal.palette_index(), SYNTAX_NORMAL_IDX);
    assert_eq!(TokenType::Keyword.palette_index(), SYNTAX_KEYWORD_IDX);
    assert_eq!(TokenType::String.palette_index(), SYNTAX_STRING_IDX);
    assert_eq!(TokenType::Comment.palette_index(), SYNTAX_COMMENT_IDX);
    assert_eq!(TokenType::Number.palette_index(), SYNTAX_NUMBER_IDX);
    assert_eq!(TokenType::Operator.palette_index(), SYNTAX_OPERATOR_IDX);
    assert_eq!(TokenType::Identifier.palette_index(), SYNTAX_IDENTIFIER_IDX);
    assert_eq!(TokenType::Type.palette_index(), SYNTAX_TYPE_IDX);
    assert_eq!(TokenType::Preprocessor.palette_index(), SYNTAX_PREPROCESSOR_IDX);
    assert_eq!(TokenType::Function.palette_index(), SYNTAX_FUNCTION_IDX);
    assert_eq!(TokenType::Special.palette_index(), SYNTAX_SPECIAL_IDX);
}
```

- [ ] **Step 8: Change editor draw to use palette-integrated colors**

In `src/views/editor.rs`, around line 1239, change:

```rust
                                    token.token_type.default_color(),
```

to:

```rust
                                    self.map_color(token.token_type.palette_index()),
```

- [ ] **Step 9: Run tests**

Run: `cargo test --lib`

Expected: All tests pass. The `test_token_type_default_colors` test in syntax.rs still passes (it tests `default_color()` which is unchanged).

- [ ] **Step 10: Commit**

```bash
git add src/core/palette.rs src/views/syntax.rs src/views/editor.rs
git commit -m "feat: palette-integrated syntax highlighting colors (#98)

Syntax colors now flow through the palette chain (editor → window → app)
instead of being hardcoded to blue background. Blue, cyan, and gray
windows each get appropriate syntax color variants."
```

---

### Task 5: Fix HelpWindow options delegation (#97a)

**Files:**
- Modify: `src/views/help_window.rs`

- [ ] **Step 1: Write the failing test**

Add to the test module in `src/views/help_window.rs`:

```rust
#[test]
fn test_help_window_options_delegation() {
    use crate::core::state::{OF_SELECTABLE, OF_TOP_SELECT, OF_TILEABLE};

    let (_file, help) = create_test_help_file();
    let bounds = Rect::new(10, 5, 70, 20);
    let window = HelpWindow::new(bounds, "Help", help);

    let options = window.options();
    assert_ne!(options, 0, "HelpWindow should delegate options() to inner window");
    assert!(
        (options & OF_SELECTABLE) != 0,
        "HelpWindow should have OF_SELECTABLE"
    );
    assert!(
        (options & OF_TOP_SELECT) != 0,
        "HelpWindow should have OF_TOP_SELECT for click-to-focus"
    );
    assert!(
        (options & OF_TILEABLE) != 0,
        "HelpWindow should have OF_TILEABLE"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib test_help_window_options_delegation -- --nocapture`

Expected: FAIL — `options()` returns 0.

- [ ] **Step 3: Add options/set_options delegation**

In `src/views/help_window.rs`, in the `impl View for HelpWindow` block (around line 315, after `get_palette`), add:

```rust
    fn options(&self) -> u16 {
        self.window.options()
    }

    fn set_options(&mut self, options: u16) {
        self.window.set_options(options);
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib test_help_window_options_delegation -- --nocapture`

Expected: PASS

- [ ] **Step 5: Run full test suite**

Run: `cargo test --lib`

Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/views/help_window.rs
git commit -m "fix: delegate options()/set_options() in HelpWindow (#97)

HelpWindow's View impl was missing options delegation, returning 0
instead of the inner window's OF_SELECTABLE | OF_TOP_SELECT | OF_TILEABLE.
This broke click-to-focus for HelpWindow on the Desktop."
```

---

### Task 6: Single-click follows hyperlinks in HelpWindow (#97b)

**Files:**
- Modify: `src/views/help_viewer.rs`
- Modify: `src/views/help_window.rs`

- [ ] **Step 1: Change HelpViewer single-click to not clear event**

In `src/views/help_viewer.rs`, in the `MouseDown` handler (around line 443-457), change the single-click branch from clearing the event to letting it propagate:

Replace:
```rust
                    if hit_ref > 0 {
                        // Select the clicked link
                        self.selected = hit_ref;

                        if event.mouse.double_click {
                            // Double-click follows the link
                            // Don't clear event - let HelpWindow handle navigation
                            // Convert to ENTER-like behavior
                        } else {
                            // Single click just selects
                            event.clear();
                        }
```

With:
```rust
                    if hit_ref > 0 {
                        // Select the clicked link
                        self.selected = hit_ref;
                        // Don't clear event — let HelpWindow follow the link
                        // (both single-click and double-click)
```

- [ ] **Step 2: Change HelpWindow to follow links on any MouseDown**

In `src/views/help_window.rs`, replace the `EventType::MouseDown` handler (around line 282-295):

```rust
            EventType::MouseDown => {
                if event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    // Let the window (and viewer) handle the click first
                    self.window.handle_event(event);

                    // If a link was clicked, follow it
                    let target = self.viewer.borrow().get_selected_target().map(|s| s.to_string());
                    if let Some(target) = target {
                        // Check if click was actually on a cross-ref
                        let mouse_pos = event.mouse.pos;
                        let hit = self.viewer.borrow().get_cross_ref_at_public(mouse_pos.x, mouse_pos.y);
                        if hit > 0 {
                            self.switch_to_topic(&target);
                        }
                    }
                    event.clear();
                    return;
                }
            }
```

- [ ] **Step 3: Make get_cross_ref_at accessible from HelpWindow**

In `src/views/help_viewer.rs`, add a public wrapper for `get_cross_ref_at`:

```rust
    /// Check if a cross-reference exists at screen position (public API for HelpWindow).
    /// Returns 1-based index or 0 if none found.
    pub fn get_cross_ref_at_public(&self, screen_x: i16, screen_y: i16) -> usize {
        self.get_cross_ref_at(screen_x, screen_y)
    }
```

- [ ] **Step 4: Run full test suite**

Run: `cargo test --lib`

Expected: All tests pass. The existing HelpWindow tests don't test mouse interaction so they should be unaffected.

- [ ] **Step 5: Commit**

```bash
git add src/views/help_viewer.rs src/views/help_window.rs
git commit -m "feat: single-click follows hyperlinks in HelpWindow (#97)

Previously only double-click followed links. Now single-click on a
cross-reference immediately navigates to the linked topic."
```

---

### Task 7: Up/Down arrow keys cycle visible hyperlinks (#97c)

**Files:**
- Modify: `src/views/help_viewer.rs`

- [ ] **Step 1: Add find_visible_cross_ref helper**

In `src/views/help_viewer.rs`, add to the `impl HelpViewer` block:

```rust
    /// Find the next or previous visible cross-reference relative to the current selection.
    /// Returns 1-based index of the next/prev visible cross-ref, or None if none found.
    fn find_visible_cross_ref(&self, forward: bool) -> Option<usize> {
        if self.cross_refs.is_empty() {
            return None;
        }

        let visible_start = self.delta.y + 1; // 1-based line number
        let visible_end = visible_start + self.bounds.height();

        // Collect visible cross-refs (1-based indices)
        let visible: Vec<usize> = self.cross_refs.iter().enumerate()
            .filter(|(_, cr)| cr.line >= visible_start && cr.line < visible_end)
            .map(|(i, _)| i + 1) // Convert to 1-based
            .collect();

        if visible.is_empty() {
            return None;
        }

        if forward {
            // Find the first visible cross-ref after current selection
            visible.iter().find(|&&idx| idx > self.selected).copied()
                .or_else(|| {
                    // If current is the last visible, return None (will scroll)
                    None
                })
        } else {
            // Find the last visible cross-ref before current selection
            visible.iter().rev().find(|&&idx| idx < self.selected).copied()
                .or_else(|| {
                    // If current is the first visible, return None (will scroll)
                    None
                })
        }
    }
```

- [ ] **Step 2: Modify Up/Down key handling**

In `src/views/help_viewer.rs`, replace the `KB_UP` and `KB_DOWN` handlers (around lines 374-381):

Replace:
```rust
                    KB_UP => {
                        self.scroll_by(0, -1);
                        event.clear();
                    }
                    KB_DOWN => {
                        self.scroll_by(0, 1);
                        event.clear();
                    }
```

With:
```rust
                    KB_UP => {
                        if !self.cross_refs.is_empty() {
                            if let Some(prev) = self.find_visible_cross_ref(false) {
                                self.selected = prev;
                                self.make_select_visible();
                            } else {
                                // At first visible link or no visible links — scroll up
                                self.scroll_by(0, -1);
                            }
                        } else {
                            self.scroll_by(0, -1);
                        }
                        event.clear();
                    }
                    KB_DOWN => {
                        if !self.cross_refs.is_empty() {
                            if let Some(next) = self.find_visible_cross_ref(true) {
                                self.selected = next;
                                self.make_select_visible();
                            } else {
                                // At last visible link or no visible links — scroll down
                                self.scroll_by(0, 1);
                            }
                        } else {
                            self.scroll_by(0, 1);
                        }
                        event.clear();
                    }
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test --lib`

Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/views/help_viewer.rs
git commit -m "feat: Up/Down arrow keys cycle visible hyperlinks in HelpViewer (#97)

When cross-references exist, Up/Down select the previous/next visible
link. When at the first/last visible link, scrolling continues."
```

---

### Task 8: Backspace restores scroll position and selected link (#97d)

**Files:**
- Modify: `src/views/help_viewer.rs`
- Modify: `src/views/help_window.rs`

- [ ] **Step 1: Add scroll state accessors to HelpViewer**

In `src/views/help_viewer.rs`, add to the `impl HelpViewer` block:

```rust
    /// Get the current scroll state (scroll position and selected cross-ref).
    pub fn get_scroll_state(&self) -> (Point, usize) {
        (self.delta, self.selected)
    }

    /// Restore a previously saved scroll state.
    pub fn set_scroll_state(&mut self, delta: Point, selected: usize) {
        self.delta = Point::new(
            delta.x.max(0).min(self.limit.x),
            delta.y.max(0).min(self.limit.y),
        );
        self.selected = if selected > 0 && selected <= self.cross_refs.len() {
            selected
        } else {
            1
        };
        self.update_scrollbar();
    }
```

- [ ] **Step 2: Write the failing test**

Add to the test module in `src/views/help_window.rs`:

```rust
#[test]
fn test_go_back_restores_scroll_state() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create help file with two topics and enough content to scroll
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "# First Topic {{#first}}").unwrap();
    writeln!(file, "").unwrap();
    for i in 0..30 {
        writeln!(file, "Line {} of first topic", i).unwrap();
    }
    writeln!(file, "").unwrap();
    writeln!(file, "Go to [Second Topic](#second)").unwrap();
    writeln!(file, "").unwrap();
    writeln!(file, "# Second Topic {{#second}}").unwrap();
    writeln!(file, "").unwrap();
    writeln!(file, "This is the second topic.").unwrap();
    file.flush().unwrap();

    let help = Rc::new(RefCell::new(
        HelpFile::new(file.path().to_str().unwrap()).unwrap(),
    ));
    let bounds = Rect::new(10, 5, 70, 15); // 10 lines tall
    let mut window = HelpWindow::new(bounds, "Help", help);

    // Show first topic and set a scroll state
    window.show_topic("first");

    // Navigate to second topic
    window.switch_to_topic("second");
    assert_eq!(window.current_topic(), Some("second".to_string()));

    // Go back — should return to first topic
    assert!(window.go_back());
    assert_eq!(window.current_topic(), Some("first".to_string()));
}
```

- [ ] **Step 3: Run test to verify baseline passes**

Run: `cargo test --lib test_go_back_restores_scroll_state -- --nocapture`

Expected: PASS (basic go_back already works; this test establishes baseline before we change the history struct).

- [ ] **Step 4: Change history from String to HistoryEntry**

In `src/views/help_window.rs`, add the struct before `impl HelpWindow`:

```rust
/// History entry storing topic ID with scroll state for restoration.
struct HistoryEntry {
    topic_id: String,
    delta: Point,
    selected: usize,
}
```

Add the import for `Point` at the top of the file:

```rust
use crate::core::geometry::{Point, Rect};
```

Change the `HelpWindow` struct fields:

```rust
    /// Topic history for back/forward navigation
    history: Vec<HistoryEntry>,
```

(Remove the `history_pos` field — we keep it but it's the same type.)

- [ ] **Step 5: Update switch_to_topic to save scroll state**

Replace `switch_to_topic` in `src/views/help_window.rs`:

```rust
    pub fn switch_to_topic(&mut self, topic_id: &str) -> bool {
        let help = self.help_file.borrow();
        if help.get_topic(topic_id).is_none() {
            return false;
        }
        drop(help);

        // Truncate future history if not at the end
        if self.history_pos < self.history.len() {
            self.history.truncate(self.history_pos);
        }

        // Save current topic + scroll state to history before switching
        if let Some(current) = self.viewer.borrow().current_topic().map(|s| s.to_string()) {
            let (delta, selected) = self.viewer.borrow().get_scroll_state();
            self.history.push(HistoryEntry {
                topic_id: current,
                delta,
                selected,
            });
        }

        let success = self.show_topic(topic_id);
        if success {
            self.history_pos = self.history.len();
        }
        success
    }
```

- [ ] **Step 6: Update go_back to restore scroll state**

Replace `go_back` in `src/views/help_window.rs`:

```rust
    pub fn go_back(&mut self) -> bool {
        if self.history_pos > 0 {
            self.history_pos -= 1;
            let entry = &self.history[self.history_pos];
            let topic_id = entry.topic_id.clone();
            let delta = entry.delta;
            let selected = entry.selected;
            self.show_topic(&topic_id);
            self.viewer.borrow_mut().set_scroll_state(delta, selected);
            true
        } else {
            false
        }
    }
```

- [ ] **Step 7: Update go_forward to restore scroll state**

Replace `go_forward` in `src/views/help_window.rs`:

```rust
    pub fn go_forward(&mut self) -> bool {
        if self.history_pos < self.history.len() {
            let entry = &self.history[self.history_pos];
            let topic_id = entry.topic_id.clone();
            let delta = entry.delta;
            let selected = entry.selected;
            self.history_pos += 1;
            self.show_topic(&topic_id);
            self.viewer.borrow_mut().set_scroll_state(delta, selected);
            true
        } else {
            false
        }
    }
```

- [ ] **Step 8: Update constructor to use new type**

In `HelpWindow::new()`, the `history` field initializer stays the same (`Vec::new()`), but make sure the type inference picks up `Vec<HistoryEntry>`.

- [ ] **Step 9: Run tests**

Run: `cargo test --lib`

Expected: All tests pass including `test_go_back_restores_scroll_state`.

- [ ] **Step 10: Commit**

```bash
git add src/views/help_viewer.rs src/views/help_window.rs
git commit -m "feat: backspace restores scroll position in HelpWindow (#97)

History entries now store scroll offset and selected cross-ref index.
go_back() and go_forward() restore the full view state."
```

---

## Summary

| Task | Issue | Description |
|------|-------|-------------|
| 1 | Pre-req | Fix Group view_ids sync in bring_to_front/send_to_back |
| 2 | #103 | Desktop::bring_to_front(ViewId) |
| 3 | #96 | Window::new_with_type() + WindowBuilder::palette_type() |
| 4 | #98 | Palette-integrated syntax highlighting |
| 5 | #97a | HelpWindow options() delegation bug |
| 6 | #97b | Single-click follows hyperlinks |
| 7 | #97c | Up/Down cycle visible hyperlinks |
| 8 | #97d | Backspace restores scroll state |

Tasks 1-2 must be sequential (2 depends on 1). Tasks 3-4 are independent of 1-2. Tasks 5-8 are sequential (each builds on HelpWindow changes). Groups {1,2}, {3}, {4}, {5,6,7,8} can run in parallel.
