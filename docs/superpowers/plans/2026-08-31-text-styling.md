# Text Styling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real terminal text styling (bold, italic, underline, reverse, dim, strikethrough) to the rendering pipeline as a first-class attribute.

**Architecture:** Introduce a hand-rolled `Style(u8)` bitset and add a `style` field to the core `Attr` type. Thread the style into the two SGR-emitting render surfaces (`terminal::flush` for live/SSH, `ansi_dump` for text snapshots), and convert the two places that currently *fake* styling (the ANSI parser and the help viewer) to set/apply real styles.

**Tech Stack:** Rust (edition 2024), no new crate dependencies (`Style` is a `u8` newtype). Crate: `turbo-vision`.

**Spec:** `docs/superpowers/specs/2026-08-31-text-styling-design.md`

## Global Constraints

- No new crate dependencies — `Style` is a hand-rolled `u8` newtype (6 bits).
- `Attr::new(fg, bg)` MUST keep its 2-argument signature and stay `const fn` (used in ~111 sites, many in `const` context). New style is defaulted to empty.
- `to_u8`/`from_u8` stay color-only (classic `fg | (bg << 4)` byte); `from_u8` sets `style = Style::empty()`.
- Existing unstyled `ansi_dump` output MUST stay byte-identical.
- Styles supported: bold, italic, underline, reverse, dim, strikethrough. No blink. PNG screenshots stay color-only.
- SGR codes: bold `1`, dim `2`, italic `3`, underline `4`, reverse `7`, strikethrough `9`; off-codes `22` (bold+dim), `23` (italic), `24` (underline), `27` (reverse), `29` (strikethrough).
- Run `cargo test` and `cargo build` from the repo root. Follow existing file conventions (each source file starts with `// (C) 2025 - Enzo Lombardi`).

## File Structure

- **Modify** `src/core/palette.rs` — add `Style` type + `style` field on `Attr` + builder methods; update `swap`/`darken`/`from_u8`.
- **Modify** `src/terminal/mod.rs` — emit style SGR codes in `flush()`.
- **Modify** `src/core/ansi_dump.rs` — emit style transition codes in `dump_buffer` and `dump_buffer_region`.
- **Modify** `src/core/ansi.rs` — parse SGR style codes into `Style` and set on cells (keep brighten-on-bold).
- **Modify** `src/views/help_viewer.rs` — apply real `.bold()`/`.italic()` to Bold/Italic segments.

---

### Task 1: `Style` bitset

**Files:**
- Modify: `src/core/palette.rs` (add `Style` above the `Attr` struct at line 356)
- Test: inline `#[cfg(test)]` in `src/core/palette.rs`

**Interfaces:**
- Produces: `pub struct Style(u8)` with associated consts `BOLD`, `ITALIC`, `UNDERLINE`, `REVERSE`, `DIM`, `STRIKETHROUGH`; methods `empty() -> Style`, `bits(self) -> u8`, `is_empty(self) -> bool`, `contains(self, Style) -> bool`, `insert(&mut self, Style)`, `remove(&mut self, Style)`; `impl BitOr`, `impl BitOrAssign`.

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `src/core/palette.rs` (create the block near the end of the file if none exists):

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib style_bitset`
Expected: FAIL — `cannot find type Style` / does not compile.

- [ ] **Step 3: Write minimal implementation**

Insert above the `Attr` struct (before line 356 `#[derive(...)] pub struct Attr`) in `src/core/palette.rs`:

```rust
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib style_bitset`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/palette.rs
git commit -m "feat(palette): add Style bitset for text attributes"
```

---

### Task 2: `style` field + builders on `Attr`

**Files:**
- Modify: `src/core/palette.rs` (`Attr` struct at 356-360; `new` at 363-365; `from_u8` at 367-372; `swap` at 380-385; `darken` at 390-403)
- Test: inline `#[cfg(test)]` in `src/core/palette.rs`

**Interfaces:**
- Consumes: `Style` from Task 1.
- Produces: `Attr { fg, bg, style }`; `Attr::new(fg, bg) -> Attr` (const, style empty); builder methods (const) `bold`, `italic`, `underline`, `reverse`, `dim`, `strikethrough`, and `with_style(Style) -> Attr`. `from_u8` sets `style = Style::empty()`; `swap`/`darken` preserve `style`.

- [ ] **Step 1: Write the failing test**

Add to the tests block in `src/core/palette.rs`:

```rust
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib attr_style_builders attr_u8_drops_style attr_swap_darken`
Expected: FAIL — no field `style` / no method `bold`.

- [ ] **Step 3: Write minimal implementation**

In `src/core/palette.rs`, change the `Attr` struct (lines 356-360) to:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attr {
    pub fg: TvColor,
    pub bg: TvColor,
    pub style: Style,
}
```

Change `new` (363-365) to keep signature but init style, and add builders right after it:

```rust
    pub const fn new(fg: TvColor, bg: TvColor) -> Self {
        Self { fg, bg, style: Style::empty() }
    }

    /// Returns a copy with the given style flags set (replacing any existing flags).
    pub const fn with_style(self, style: Style) -> Self {
        Self { fg: self.fg, bg: self.bg, style }
    }

    /// Returns a copy with an additional style flag set.
    const fn add_style(self, flag: Style) -> Self {
        Self { fg: self.fg, bg: self.bg, style: Style(self.style.bits() | flag.bits()) }
    }

    pub const fn bold(self) -> Self { self.add_style(Style::BOLD) }
    pub const fn italic(self) -> Self { self.add_style(Style::ITALIC) }
    pub const fn underline(self) -> Self { self.add_style(Style::UNDERLINE) }
    pub const fn reverse(self) -> Self { self.add_style(Style::REVERSE) }
    pub const fn dim(self) -> Self { self.add_style(Style::DIM) }
    pub const fn strikethrough(self) -> Self { self.add_style(Style::STRIKETHROUGH) }
```

> Note: `add_style` builds `Style` via `Style(..)` directly because it lives in the same module (the tuple field is in scope). Keep it as shown.

Change `from_u8` (367-372) to add `style: Style::empty()`:

```rust
    pub fn from_u8(byte: u8) -> Self {
        Self {
            fg: TvColor::from_u8(byte & 0x0F),
            bg: TvColor::from_u8((byte >> 4) & 0x0F),
            style: Style::empty(),
        }
    }
```

Change `swap` (380-385) to preserve style:

```rust
    pub fn swap(self) -> Self {
        Self {
            fg: self.bg,
            bg: self.fg,
            style: self.style,
        }
    }
```

Change the `darken` return (the `Self { fg: ..., bg: ... }` at lines 399-402) to include `style: self.style`:

```rust
        Self {
            fg: darken_color(self.fg),
            bg: darken_color(self.bg),
            style: self.style,
        }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib attr_style_builders attr_u8_drops_style attr_swap_darken`
Expected: PASS.

- [ ] **Step 5: Verify the whole crate still builds (backward compat of ~111 call sites)**

Run: `cargo build` then `cargo test --lib`
Expected: Builds clean; all pre-existing tests pass (the added `style` field defaults through `Attr::new`, so no call site breaks).

- [ ] **Step 6: Commit**

```bash
git add src/core/palette.rs
git commit -m "feat(palette): add style field and builders to Attr"
```

---

### Task 3: Emit style SGR in live terminal `flush`

**Files:**
- Modify: `src/terminal/mod.rs` (`flush` at 447-508; specifically the color-emit block at 474-481)
- Test: inline `#[cfg(test)]` in `src/terminal/mod.rs`

**Interfaces:**
- Consumes: `Attr.style`, `Style` from Tasks 1-2.
- Produces: `fn attr_to_sgr(attr: Attr) -> String` returning `\x1b[0;38;2;r;g;b;48;2;r;g;b{;<style codes>}m`; `flush` uses it per changed run. The leading `0;` reset clears stale style from the previous run.

> **Why a pure helper instead of driving `flush`:** there is no byte-capturing test backend in this repo (`MockTerminal` stores cells directly and bypasses `flush`; only `CrosstermBackend`/`SshBackend`/a no-op `NullBackend` implement `Backend`). Extracting the escape-sequence construction into a pure function makes the SGR output directly unit-testable without a backend, and keeps `flush` readable.

- [ ] **Step 1: Write the failing test**

Add to `#[cfg(test)] mod tests` in `src/terminal/mod.rs`:

```rust
#[test]
fn test_attr_to_sgr_bold_and_reset() {
    use crate::core::palette::{Attr, TvColor};
    let bold = Attr::new(TvColor::White, TvColor::Blue).bold();
    let plain = Attr::new(TvColor::White, TvColor::Blue);

    let s = attr_to_sgr(bold);
    assert!(s.starts_with("\x1b[0;38;2;"), "leading reset + truecolor fg");
    assert!(s.contains(";48;2;"), "truecolor bg present");
    assert!(s.ends_with(";1m"), "bold code appended: {s:?}");

    // Plain attr emits no style codes, but still carries the leading reset so a
    // previous run's style does not bleed into it.
    let p = attr_to_sgr(plain);
    assert!(p.starts_with("\x1b[0;38;2;"));
    assert!(!p.contains(";1"), "no bold code for plain attr");
    assert!(p.ends_with("m"));
}

#[test]
fn test_attr_to_sgr_multiple_styles_ordered() {
    use crate::core::palette::{Attr, TvColor, Style};
    let a = Attr::new(TvColor::White, TvColor::Blue)
        .with_style(Style::ITALIC | Style::UNDERLINE);
    let s = attr_to_sgr(a);
    // italic(3) before underline(4)
    let i3 = s.find(";3").unwrap();
    let i4 = s.find(";4").unwrap();
    assert!(i3 < i4, "style codes emitted in canonical order: {s:?}");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib attr_to_sgr`
Expected: FAIL — `cannot find function attr_to_sgr`.

- [ ] **Step 3: Write minimal implementation**

In `src/terminal/mod.rs`, add a free function near the top of the file (after imports):

```rust
/// Build the full SGR escape for a cell run: a reset, truecolor fg/bg, then any
/// style flags. The leading `0` reset prevents a previous run's style from
/// bleeding, since each run re-emits absolute colors.
/// Style order: bold(1), dim(2), italic(3), underline(4), reverse(7), strikethrough(9).
fn attr_to_sgr(attr: crate::core::palette::Attr) -> String {
    use crate::core::palette::Style;
    let (fg_r, fg_g, fg_b) = attr.fg.to_rgb();
    let (bg_r, bg_g, bg_b) = attr.bg.to_rgb();
    let mut s = format!(
        "\x1b[0;38;2;{};{};{};48;2;{};{};{}",
        fg_r, fg_g, fg_b, bg_r, bg_g, bg_b
    );
    if attr.style.contains(Style::BOLD) { s.push_str(";1"); }
    if attr.style.contains(Style::DIM) { s.push_str(";2"); }
    if attr.style.contains(Style::ITALIC) { s.push_str(";3"); }
    if attr.style.contains(Style::UNDERLINE) { s.push_str(";4"); }
    if attr.style.contains(Style::REVERSE) { s.push_str(";7"); }
    if attr.style.contains(Style::STRIKETHROUGH) { s.push_str(";9"); }
    s.push('m');
    s
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib attr_to_sgr`
Expected: PASS.

- [ ] **Step 5: Use the helper in `flush`**

In `flush`, replace the color-emit block (lines 474-481) with:

```rust
                // Full SGR for this run (reset + truecolor + style).
                output.extend_from_slice(attr_to_sgr(current_attr).as_bytes());
```

- [ ] **Step 6: Full regression**

Run: `cargo test`
Expected: All pass (no test pins the old exact `flush` bytes — verified during design; the only behavioral change is the added leading `0;` reset and trailing style codes).

- [ ] **Step 7: Commit**

```bash
git add src/terminal/mod.rs
git commit -m "feat(terminal): emit style SGR codes in flush"
```

---

### Task 4: Emit style transitions in `ansi_dump`

**Files:**
- Modify: `src/core/ansi_dump.rs` (`dump_buffer` at 94-139; `dump_buffer_region` at 152+)
- Test: inline `#[cfg(test)]` in `src/core/ansi_dump.rs`

**Interfaces:**
- Consumes: `Attr.style`, `Style`.
- Produces: within a line, tracks `last_style` (init empty); on change emits minimal on-codes (1/2/3/4/7/9) for newly-set bits and off-codes (22/23/24/27/29) for newly-cleared bits, before the character. Unstyled buffers produce byte-identical output to before.

- [ ] **Step 1: Write the failing test**

Add to `#[cfg(test)] mod tests` in `src/core/ansi_dump.rs`:

```rust
#[test]
fn test_dump_unstyled_unchanged() {
    use crate::core::palette::{Attr, TvColor};
    let attr = Attr::new(TvColor::White, TvColor::Blue);
    let row = vec![Cell::new('H', attr), Cell::new('i', attr)];
    let buffer = vec![row];
    let mut out = Vec::new();
    dump_buffer(&mut out, &buffer, 2, 1).unwrap();
    let s = String::from_utf8(out).unwrap();
    // No style SGR codes for unstyled content.
    assert!(!s.contains(";1m") && !s.contains("[1m"));
    assert!(!s.contains("[3m") && !s.contains("[4m"));
}

#[test]
fn test_dump_emits_bold_then_clears() {
    use crate::core::palette::{Attr, TvColor};
    let bold = Attr::new(TvColor::White, TvColor::Blue).bold();
    let plain = Attr::new(TvColor::White, TvColor::Blue);
    let row = vec![Cell::new('B', bold), Cell::new('n', plain)];
    let buffer = vec![row];
    let mut out = Vec::new();
    dump_buffer(&mut out, &buffer, 2, 1).unwrap();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("\x1b[1m"), "bold on-code emitted");
    assert!(s.contains("\x1b[22m"), "bold off-code emitted before plain char");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib dump_emits_bold_then_clears`
Expected: FAIL — no `\x1b[1m` emitted.

- [ ] **Step 3: Write minimal implementation**

Add a helper near the top of `src/core/ansi_dump.rs` (after imports):

```rust
/// Emit the minimal SGR transition codes to move from `from` to `to`.
fn write_style_transition<W: Write>(
    writer: &mut W,
    from: crate::core::palette::Style,
    to: crate::core::palette::Style,
) -> io::Result<()> {
    use crate::core::palette::Style;
    // Pairs of (flag, on-code, off-code).
    const FLAGS: &[(Style, u8, u8)] = &[
        (Style::BOLD, 1, 22),
        (Style::DIM, 2, 22),
        (Style::ITALIC, 3, 23),
        (Style::UNDERLINE, 4, 24),
        (Style::REVERSE, 7, 27),
        (Style::STRIKETHROUGH, 9, 29),
    ];
    for &(flag, on, off) in FLAGS {
        let was = from.contains(flag);
        let now = to.contains(flag);
        if now && !was {
            write!(writer, "\x1b[{}m", on)?;
        } else if was && !now {
            write!(writer, "\x1b[{}m", off)?;
        }
    }
    Ok(())
}
```

> Note: bold and dim share off-code 22; because both are handled independently and 22 clears both, a buffer that toggles only one while the other stays on would over-clear. This is acceptable for the dump path (snapshots), and both being simultaneously set is rare. Keep as shown.

In `dump_buffer`, add `let mut last_style = crate::core::palette::Style::empty();` next to `last_fg`/`last_bg` (line 102), and immediately before `write!(writer, "{}", cell.ch)?;` (line 131) insert:

```rust
            if cell.attr.style != last_style {
                write_style_transition(writer, last_style, cell.attr.style)?;
                last_style = cell.attr.style;
            }
```

The end-of-line `\x1b[0m` (line 135) already resets style; `last_style` resets to `empty()` at the top of the next row loop iteration, matching that reset.

Apply the identical change to `dump_buffer_region` (add `last_style` init alongside its `last_fg`/`last_bg`, and the same transition block before it writes each `cell.ch`).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib dump_unstyled_unchanged dump_emits_bold_then_clears`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/core/ansi_dump.rs
git commit -m "feat(ansi_dump): emit style transition codes"
```

---

### Task 5: Parse style SGR into cells (ANSI parser)

**Files:**
- Modify: `src/core/ansi.rs` (`parse_line` at 138-169; `parse_sgr` signature 172-178 and code loop 214-308; return tuple)
- Test: inline `#[cfg(test)]` in `src/core/ansi.rs`

**Interfaces:**
- Consumes: `Style`, `Attr.with_style`.
- Produces: `parse_line` sets real `Style` bits on each cell's `Attr` while KEEPING the existing brighten-on-bold color. `parse_sgr` returns `(TvColor, TvColor, bool, Style)`.

- [ ] **Step 1: Write the failing test**

Add to the tests block in `src/core/ansi.rs` (keep the existing `test_parse_bold` untouched — it must still pass):

```rust
#[test]
fn test_parse_style_flags() {
    let parser = AnsiParser::new();
    let cells = parser.parse_line("\x1b[1mA\x1b[3mB\x1b[4mC\x1b[0mD");
    assert!(cells[0].attr.style.contains(Style::BOLD));           // A: bold
    assert!(cells[1].attr.style.contains(Style::BOLD));           // B: bold+italic
    assert!(cells[1].attr.style.contains(Style::ITALIC));
    assert!(cells[2].attr.style.contains(Style::UNDERLINE));      // C: +underline
    assert!(cells[3].attr.style.is_empty());                      // D: reset
}
```

Ensure `Style` is imported in the test module (add `use crate::core::palette::Style;` if needed).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib parse_style_flags`
Expected: FAIL — no field `style` bits set / does not compile.

- [ ] **Step 3: Write minimal implementation**

In `src/core/ansi.rs`:

1. Change `parse_line` (138-169). Add a style accumulator and thread it:

```rust
    pub fn parse_line(&self, line: &str) -> Vec<Cell> {
        let mut cells = Vec::new();
        let mut current_fg = self.default_fg;
        let mut current_bg = self.default_bg;
        let mut bright = false;
        let mut style = crate::core::palette::Style::empty();

        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    let (new_fg, new_bg, new_bright, new_style) =
                        self.parse_sgr(&mut chars, current_fg, current_bg, bright, style);
                    current_fg = new_fg;
                    current_bg = new_bg;
                    bright = new_bright;
                    style = new_style;
                }
            } else if ch != '\r' {
                let fg = if bright {
                    Self::brighten(current_fg)
                } else {
                    current_fg
                };
                cells.push(Cell::new(ch, Attr::new(fg, current_bg).with_style(style)));
            }
        }

        cells
    }
```

2. Change `parse_sgr` signature (172-178) to accept and return `Style`:

```rust
    fn parse_sgr(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        mut fg: TvColor,
        mut bg: TvColor,
        mut bright: bool,
        mut style: crate::core::palette::Style,
    ) -> (TvColor, TvColor, bool, crate::core::palette::Style) {
```

3. Update the two early-return `return (fg, bg, bright);` (line ~209) and the final `(fg, bg, bright)` (line ~308) to include `style`: `return (fg, bg, bright, style);` and `(fg, bg, bright, style)`.

4. In the code-processing `match code` block, update/extend the style arms. Replace the existing `0`, `1`, and `22` arms and add the new ones:

```rust
                0 => {
                    // Reset
                    fg = self.default_fg;
                    bg = self.default_bg;
                    bright = false;
                    style = crate::core::palette::Style::empty();
                }
                1 => {
                    // Bold: keep brighten AND set the real bold flag.
                    bright = true;
                    style.insert(crate::core::palette::Style::BOLD);
                }
                2 => style.insert(crate::core::palette::Style::DIM),
                3 => style.insert(crate::core::palette::Style::ITALIC),
                4 => style.insert(crate::core::palette::Style::UNDERLINE),
                7 => style.insert(crate::core::palette::Style::REVERSE),
                9 => style.insert(crate::core::palette::Style::STRIKETHROUGH),
                22 => {
                    // Normal intensity
                    bright = false;
                    style.remove(crate::core::palette::Style::BOLD);
                    style.remove(crate::core::palette::Style::DIM);
                }
                23 => style.remove(crate::core::palette::Style::ITALIC),
                24 => style.remove(crate::core::palette::Style::UNDERLINE),
                27 => style.remove(crate::core::palette::Style::REVERSE),
                29 => style.remove(crate::core::palette::Style::STRIKETHROUGH),
```

Leave all color arms (30-37, 38, 39, 40-47, 48, 49, 90-97, 100-107) unchanged.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib parse_style_flags parse_bold`
Expected: BOTH pass — `test_parse_bold` still sees `LightBlue` (brighten kept); `parse_style_flags` sees the new flags.

- [ ] **Step 5: Commit**

```bash
git add src/core/ansi.rs
git commit -m "feat(ansi): parse style SGR codes into cell attributes"
```

---

### Task 6: Real bold/italic in the help viewer

**Files:**
- Modify: `src/views/help_viewer.rs` (color vars at 359-364; segment→color match at 385-400)
- Test: inline `#[cfg(test)]` in `src/views/help_viewer.rs` (or extend existing help_viewer tests)

**Interfaces:**
- Consumes: `Attr.bold()`, `Attr.italic()`, `Style`.
- Produces: `TextSegment::Bold` renders with `Style::BOLD` set; `TextSegment::Italic` with `Style::ITALIC`. `Code`/`Link`/`Normal` unchanged.

- [ ] **Step 1: Apply the style in the draw mapping**

In `src/views/help_viewer.rs`, change the match arms at 387-388:

```rust
                            TextSegment::Bold(_) => bold_color.bold(),
                            TextSegment::Italic(_) => italic_color.italic(),
```

(Leave `Normal`, `Code`, and `Link` arms unchanged.)

- [ ] **Step 2: Write the test**

The mapping is a local `match` inside `draw`, so test at the drawing level via `MockTerminal` if the help viewer has a testable draw path, OR extract the mapping. Prefer a lightweight extraction: add a private helper and unit-test it. Add near the other `impl` methods:

```rust
/// Resolve the base color of a segment into a styled attribute.
/// Bold/Italic segments carry real style flags; others are color-only.
fn styled_attr(segment: &TextSegment, normal: Attr, bold_color: Attr,
               italic_color: Attr, code_color: Attr, keyword: Attr) -> Attr {
    match segment {
        TextSegment::Normal(_) => normal,
        TextSegment::Bold(_) => bold_color.bold(),
        TextSegment::Italic(_) => italic_color.italic(),
        TextSegment::Code(_) => code_color,
        TextSegment::Link { .. } => keyword,
    }
}
```

Then in `draw`, for the non-link arms, call `styled_attr` (keep the existing selected-link logic inline for the `Link` case, or pass the already-resolved `keyword`/`sel_keyword`). Add the test:

```rust
#[test]
fn test_help_segment_styles() {
    use crate::core::palette::{Attr, TvColor, Style};
    let c = Attr::new(TvColor::White, TvColor::Blue);
    let bold = styled_attr(&TextSegment::Bold("x".into()), c, c, c, c, c);
    assert!(bold.style.contains(Style::BOLD));
    let ital = styled_attr(&TextSegment::Italic("x".into()), c, c, c, c, c);
    assert!(ital.style.contains(Style::ITALIC));
    let norm = styled_attr(&TextSegment::Normal("x".into()), c, c, c, c, c);
    assert!(norm.style.is_empty());
}
```

> If extracting `styled_attr` complicates the existing selected-link branch, keep the inline `match` change from Step 1 and instead assert the behavior through the existing help-viewer draw/render test harness if one exists. The inline `.bold()`/`.italic()` change (Step 1) is the required deliverable; the helper is only to make it unit-testable.

- [ ] **Step 3: Run test to verify it passes**

Run: `cargo test --lib help_segment_styles`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/views/help_viewer.rs
git commit -m "feat(help): render bold/italic segments with real styles"
```

---

### Task 7: Final regression + example

**Files:**
- Optional add: extend `examples/ui_features.rs` or add a small styled-text demo (only if a natural spot exists).

- [ ] **Step 1: Full test + build**

Run: `cargo test && cargo build --all-targets`
Expected: All pass; workspace (incl. `extras`) builds clean.

- [ ] **Step 2: Clippy (match project bar)**

Run: `cargo clippy --all-targets -- -D warnings` (if the project uses this; otherwise `cargo clippy`)
Expected: No new warnings from the added code.

- [ ] **Step 3: Manual smoke via ansi_dump**

Confirm a styled buffer round-trips visibly. Reuse the `verify` skill or run an example that draws styled text and dump it. Expected: bold/italic/underline appear in a capable terminal.

- [ ] **Step 4: Commit any example/doc additions**

```bash
git add -A
git commit -m "docs(examples): demonstrate text styling"
```

---

## Self-Review

**Spec coverage:**
- Style bitset → Task 1. ✓
- `Attr.style` + builders + `new`/`from_u8`/`swap`/`darken` → Task 2. ✓
- Live terminal (real + SSH via `flush`) SGR → Task 3. ✓
- ANSI text dumps → Task 4. ✓
- ANSI parser rewire (keep brighten + set BOLD) → Task 5. ✓
- Help viewer real bold/italic → Task 6. ✓
- PNG screenshots color-only, no blink → not implemented (correctly out of scope). ✓
- Backward compat of `Attr::new` call sites → Task 2 Step 5. ✓
- Unstyled `ansi_dump` byte-identical → Task 4 Step 1 (`test_dump_unstyled_unchanged`). ✓

**Placeholder scan:** No TBD/TODO; each code step shows real code. Two steps (Task 3 Step 2, Task 6 Step 2) instruct verifying exact local symbol names via `grep` before writing the test — this is deliberate because those names live in files not fully quoted here; the assertions and implementation are fully specified.

**Type consistency:** `Style`, `Attr::new`, `with_style`, `.bold()`/`.italic()`/`.underline()`/etc., `contains`/`insert`/`remove`/`is_empty`/`bits`, `parse_sgr` returning the 4-tuple, `style_sgr_suffix`, `write_style_transition`, `styled_attr` — names used consistently across tasks.
