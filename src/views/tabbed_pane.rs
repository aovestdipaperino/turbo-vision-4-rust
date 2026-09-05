// (C) 2025 - Enzo Lombardi

//! TabbedPane view - a strip of tabs over a stack of pages.
//!
//! Not part of the Borland Turbo Vision widget set. Each page is a [`Group`],
//! so a page holds ordinary controls and runs its own focus traversal; the pane
//! draws the frame, keeps one page active, and forwards everything else to it.
//!
//! Each tab is drawn as its own box sitting on top of the page frame. The
//! active tab's floor is open, so it reads as the front of the page:
//!
//! ```text
//!  \u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510} \u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}
//!  \u{2502} General \u{2502} \u{2502} About \u{2502}
//! \u{250C}\u{2518}         \u{2514}\u{2534}\u{2500}\u{2534}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2534}\u{2500}\u{2500}\u{2510}
//! \u{2502}                        \u{2502}
//! \u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}
//! ```
//!
//! The header is three rows tall, so [`TabbedPane::page_area`] starts on the
//! fourth.
//!
//! Tab switching deliberately avoids the arrow and Tab keys, because those
//! belong to the controls on the page.
//!
//! # Keys
//!
//! | Key | Action |
//! |-----|--------|
//! | F6 | Next tab |
//! | Shift+F6 | Previous tab |
//! | Ctrl+PgDn | Next tab |
//! | Ctrl+PgUp | Previous tab |
//! | Alt+letter | The tab whose title marks that letter with tildes |
//!
//! F6 is the binding to rely on. Several terminals, macOS Terminal among them,
//! send no distinct sequence for a modified Page key, so Ctrl+PgUp and Ctrl+PgDn
//! simply never arrive there; a plain function key always does. Clicking a tab
//! works everywhere the mouse does.
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::views::tabbed_pane::TabbedPane;
//! use turbo_vision::views::group::Group;
//! use turbo_vision::core::geometry::Rect;
//!
//! let bounds = Rect::new(2, 2, 40, 12);
//! let mut pane = TabbedPane::new(bounds);
//! pane.add_page("~G~eneral", Group::new(pane.page_area()));
//! pane.add_page("~A~dvanced", Group::new(pane.page_area()));
//! assert_eq!(pane.active(), 0);
//! ```

use super::group::Group;
use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_F6, KB_PGDN, KB_PGUP, MB_LEFT_BUTTON};
use crate::core::geometry::{Point, Rect};
use crate::core::palette::{Attr, LABEL_NORMAL, LABEL_SELECTED, LABEL_SHORTCUT};
use crate::core::state::State;
use crate::core::state::StateFlags;
use crate::terminal::Terminal;

/// Blank cells on each side of a tab title.
const TAB_PADDING: usize = 1;
/// The joint drawn between two tabs, one cell wide.
const TAB_GAP: usize = 1;

/// Box-drawing pieces of the pane frame.
const FRAME_TOP_LEFT: char = '\u{250C}';
const FRAME_TOP_RIGHT: char = '\u{2510}';
const FRAME_BOTTOM_LEFT: char = '\u{2514}';
const FRAME_BOTTOM_RIGHT: char = '\u{2518}';
const FRAME_HORIZONTAL: char = '\u{2500}';
const FRAME_VERTICAL: char = '\u{2502}';
/// Joint where an inactive tab's wall meets the page's top edge.
const FRAME_TAB_JOINT: char = '\u{2534}';
/// The page's top edge turning up into the active tab's left wall.
const FRAME_ACTIVE_LEFT: char = '\u{2518}';
/// The page's top edge turning up into the active tab's right wall.
const FRAME_ACTIVE_RIGHT: char = '\u{2514}';

/// Rows the tab header occupies: tab tops, titles, and the page's top edge.
const HEADER_ROWS: i16 = 3;

/// Label-palette entry for disabled text, which in a gray dialog resolves to
/// dark grey on the dialog background. The box rules use it so they read as
/// quiet structure rather than as black text.
const LABEL_DIMMED: u8 = 5;

/// One tab and the page it shows.
struct Tab {
    /// Title as given, tildes included.
    title: String,
    /// Title with the tilde markers stripped, which is what gets drawn.
    label: String,
    /// The letter between tildes, lowercased, for the Alt hotkey.
    hotkey: Option<char>,
    /// Position of the hotkey within `label`, for highlighting.
    hotkey_pos: Option<usize>,
    page: Group,
}

impl Tab {
    fn new(title: &str, page: Group) -> Self {
        let (label, hotkey, hotkey_pos) = parse_title(title);
        Self {
            title: title.to_string(),
            label,
            hotkey,
            hotkey_pos,
            page,
        }
    }

    /// Cells this tab occupies, padding included.
    fn width(&self) -> usize {
        self.label.chars().count() + TAB_PADDING * 2
    }
}

/// Split a title such as `"~G~eneral"` into its drawn label, its hotkey letter
/// and the letter's position.
///
/// A title with no tildes, or an unterminated one, simply has no hotkey.
fn parse_title(title: &str) -> (String, Option<char>, Option<usize>) {
    let mut label = String::new();
    let mut hotkey = None;
    let mut hotkey_pos = None;
    let mut chars = title.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '~' {
            label.push(ch);
            continue;
        }
        // The character after the tilde is the hotkey, and a closing tilde is
        // swallowed if present.
        if let Some(letter) = chars.next() {
            if hotkey.is_none() {
                hotkey = Some(letter.to_ascii_lowercase());
                hotkey_pos = Some(label.chars().count());
            }
            label.push(letter);
            if chars.peek() == Some(&'~') {
                chars.next();
            }
        }
    }
    (label, hotkey, hotkey_pos)
}

/// A strip of tabs over a stack of pages.
pub struct TabbedPane {
    core: ViewCore,
    tabs: Vec<Tab>,
    active: usize,
    view_state: StateFlags,
}

impl TabbedPane {
    /// Create an empty pane. The top row is the tab strip.
    pub fn new(bounds: Rect) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            tabs: Vec::new(),
            active: 0,
            view_state: State::empty(),
        }
    }

    /// The rect a page occupies: everything under the tab strip.
    ///
    /// Build each page's [`Group`] with this, so pages line up with the pane.
    pub fn page_area(&self) -> Rect {
        Rect::new(
            self.core.bounds.a.x + 1,
            self.core.bounds.a.y + HEADER_ROWS,
            self.core.bounds.b.x - 1,
            self.core.bounds.b.y - 1,
        )
    }

    /// Add a page under `title`.
    ///
    /// A tilde-wrapped letter in the title, as in `"~G~eneral"`, becomes the
    /// tab's Alt hotkey. Returns the new tab's index.
    pub fn add_page(&mut self, title: &str, page: Group) -> usize {
        self.tabs.push(Tab::new(title, page));
        self.tabs.len() - 1
    }

    /// Number of tabs.
    pub fn page_count(&self) -> usize {
        self.tabs.len()
    }

    /// Index of the active tab.
    pub fn active(&self) -> usize {
        self.active
    }

    /// Title of the active tab, tildes included, or `None` when there are no
    /// tabs.
    pub fn active_title(&self) -> Option<&str> {
        self.tabs.get(self.active).map(|t| &*t.title)
    }

    /// Show a page. Out-of-range indices are ignored.
    ///
    /// Returns true when the active page actually changed.
    pub fn set_active(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() || index == self.active {
            return false;
        }
        // The outgoing page stops being drawn, so it must not keep a focused
        // control; the incoming one takes over.
        let focused = self.is_focused();
        if let Some(page) = self.active_page_mut() {
            page.clear_all_focus();
        }
        self.active = index;
        if focused {
            if let Some(page) = self.active_page_mut() {
                page.set_initial_focus();
            }
        }
        true
    }

    /// The active page, for adding controls after construction.
    pub fn page_mut(&mut self, index: usize) -> Option<&mut Group> {
        self.tabs.get_mut(index).map(|t| &mut t.page)
    }

    /// The active page.
    pub fn active_page_mut(&mut self) -> Option<&mut Group> {
        let active = self.active;
        self.page_mut(active)
    }

    /// Give the active page's first focusable control the focus.
    pub fn set_initial_focus(&mut self) {
        if let Some(page) = self.active_page_mut() {
            page.set_initial_focus();
        }
    }

    /// Move to the next or previous tab, wrapping around.
    ///
    /// Returns true when the active page changed.
    fn cycle(&mut self, forward: bool) -> bool {
        if self.tabs.len() < 2 {
            return false;
        }
        let next = if forward {
            (self.active + 1) % self.tabs.len()
        } else {
            (self.active + self.tabs.len() - 1) % self.tabs.len()
        };
        self.set_active(next)
    }

    /// Starting cell of each tab within the strip.
    /// Starting cell of each tab within the top edge.
    ///
    /// The first tab begins one cell in, past the frame's top-left corner; each
    /// following one is separated by a single joint cell.
    fn tab_offsets(&self) -> Vec<usize> {
        let mut offsets = Vec::with_capacity(self.tabs.len());
        let mut x = 1;
        for tab in &self.tabs {
            offsets.push(x);
            x += tab.width() + TAB_GAP;
        }
        offsets
    }

    /// Index of the tab under a screen point, if the point is on the strip.
    fn tab_at(&self, pos: Point) -> Option<usize> {
        // Any of the tab's own three rows counts as a hit on it, so a click
        // near the top or bottom edge is not silently lost.
        let local_y = pos.y - self.core.bounds.a.y;
        if !(0..HEADER_ROWS).contains(&local_y) || !self.core.bounds.contains(pos) {
            return None;
        }
        let local_x = (pos.x - self.core.bounds.a.x) as usize;
        let offsets = self.tab_offsets();
        for (index, offset) in offsets.iter().enumerate() {
            if local_x >= *offset && local_x < offset + self.tabs[index].width() {
                return Some(index);
            }
        }
        None
    }

    /// Index of the tab whose hotkey is `letter`.
    fn tab_for_hotkey(&self, letter: char) -> Option<usize> {
        let letter = letter.to_ascii_lowercase();
        self.tabs.iter().position(|t| t.hotkey == Some(letter))
    }

    /// Draw the tab boxes and the page frame under them.
    ///
    /// Three header rows: the tab tops, the titles, then the page's top edge,
    /// which is broken open under the active tab so the two read as one shape.
    fn draw_frame(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped().max(0) as usize;
        let height = self.core.bounds.height_clamped().max(0) as usize;
        if width < 2 || height <= HEADER_ROWS as usize {
            return;
        }
        // Tab titles sit on the owner's own background, so a tab reads as part
        // of the dialog rather than as a button; the active one is told apart by
        // its open floor and its brighter text.
        let tabs_painter = HeaderPainter {
            core: ViewCore {
                palette_chain: self.core.palette_chain.clone(),
                ..ViewCore::default()
            },
            palette: crate::core::palette::palettes::CP_LABEL,
        };
        let rules_painter = HeaderPainter {
            core: ViewCore {
                palette_chain: self.core.palette_chain.clone(),
                ..ViewCore::default()
            },
            palette: crate::core::palette::palettes::CP_LABEL,
        };
        let normal = tabs_painter.map_color(LABEL_NORMAL);
        let active = tabs_painter.map_color(LABEL_SELECTED);
        let shortcut = tabs_painter.map_color(LABEL_SHORTCUT);
        // The box rules are drawn dimmed on the owner's background, so the pane
        // sits in a dialog without a colour of its own.
        let frame = rules_painter.map_color(LABEL_DIMMED);

        let x0 = self.core.bounds.a.x;
        let y0 = self.core.bounds.a.y;

        // Row 0 and row 1: each tab as its own box. Blank elsewhere, so the
        // dialog shows between tabs.
        let mut tops = DrawBuffer::new(width);
        tops.move_char(0, ' ', frame, width);
        let mut titles = DrawBuffer::new(width);
        titles.move_char(0, ' ', frame, width);
        // Row 2: the page's top edge, later broken open under the active tab.
        let mut edge = DrawBuffer::new(width);
        edge.move_char(0, FRAME_HORIZONTAL, frame, width);
        edge.put_char(0, FRAME_TOP_LEFT, frame);
        edge.put_char(width - 1, FRAME_TOP_RIGHT, frame);

        for (index, offset) in self.tab_offsets().into_iter().enumerate() {
            let tab = &self.tabs[index];
            let span = tab.width();
            // A tab that would run past the right frame edge is dropped rather
            // than clipped, so a half-drawn box never appears.
            if offset + span > width - 1 {
                break;
            }
            let is_active = index == self.active;
            let attr: Attr = if is_active { active } else { normal };
            let right = offset + span - 1;

            // Tab roof.
            tops.move_char(offset, FRAME_HORIZONTAL, frame, span);
            tops.put_char(offset, FRAME_TOP_LEFT, frame);
            tops.put_char(right, FRAME_TOP_RIGHT, frame);

            // Tab walls and title.
            titles.move_char(offset, ' ', attr, span);
            titles.put_char(offset, FRAME_VERTICAL, frame);
            titles.put_char(right, FRAME_VERTICAL, frame);
            let text_at = offset + TAB_PADDING;
            titles.move_str(text_at, &tab.label, attr);
            // Repaint just the hotkey letter in the shortcut colour.
            if let Some(pos) = tab.hotkey_pos {
                if let Some(letter) = tab.label.chars().nth(pos) {
                    titles.put_char(text_at + pos, letter, shortcut);
                }
            }

            // Where the tab meets the page edge: the active tab opens into the
            // page, an inactive one closes onto it.
            if is_active {
                edge.put_char(offset, FRAME_ACTIVE_LEFT, frame);
                // Open floor: this belongs to the page below, so it takes the
                // owner's background rather than the tab's colour.
                edge.move_char(offset + 1, ' ', frame, span.saturating_sub(2));
                edge.put_char(right, FRAME_ACTIVE_RIGHT, frame);
            } else {
                edge.put_char(offset, FRAME_TAB_JOINT, frame);
                edge.put_char(right, FRAME_TAB_JOINT, frame);
            }
        }

        write_line_to_terminal(terminal, x0, y0, &tops);
        write_line_to_terminal(terminal, x0, y0 + 1, &titles);
        write_line_to_terminal(terminal, x0, y0 + 2, &edge);

        // Side edges. The page draws itself between them.
        let mut wall = DrawBuffer::new(1);
        wall.put_char(0, FRAME_VERTICAL, frame);
        for row in HEADER_ROWS as usize..height - 1 {
            let y = y0 + row as i16;
            write_line_to_terminal(terminal, x0, y, &wall);
            write_line_to_terminal(terminal, self.core.bounds.b.x - 1, y, &wall);
        }

        // Bottom edge.
        let mut bottom = DrawBuffer::new(width);
        bottom.move_char(0, FRAME_HORIZONTAL, frame, width);
        bottom.put_char(0, FRAME_BOTTOM_LEFT, frame);
        bottom.put_char(width - 1, FRAME_BOTTOM_RIGHT, frame);
        write_line_to_terminal(terminal, x0, self.core.bounds.b.y - 1, &bottom);
    }
}

/// Resolves one palette's colours against the pane's chain.
///
/// The pane itself is transparent so its pages inherit the owner's palette
/// unchanged. The header needs two palettes of its own: button colours for the
/// tab titles and static-text colours for the box rules, which read as ordinary
/// text on the owner's background. Neither ever joins the view hierarchy.
struct HeaderPainter {
    core: ViewCore,
    palette: &'static [u8],
}

impl View for HeaderPainter {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, _terminal: &mut Terminal) {}

    fn handle_event(&mut self, _event: &mut Event) {}

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        Some(crate::core::palette::Palette::from_slice(self.palette))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl View for TabbedPane {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.core.bounds = bounds;
        // Every page follows the pane, so a resized dialog resizes its pages.
        let area = self.page_area();
        for tab in &mut self.tabs {
            tab.page.set_bounds(area);
        }
    }

    /// The pane takes focus on behalf of its active page, then hands it
    /// straight to that page's first control.
    fn can_focus(&self) -> bool {
        true
    }

    /// Focus follows through to the active page's controls, so the pane never
    /// swallows the focus itself.
    fn set_focus(&mut self, focused: bool) {
        self.set_state_flag(crate::core::state::State::FOCUSED, focused);
        if let Some(page) = self.active_page_mut() {
            if focused {
                page.set_initial_focus();
            } else {
                page.clear_all_focus();
            }
        }
    }

    fn state(&self) -> StateFlags {
        self.view_state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.view_state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.draw_frame(terminal);

        let chain = crate::core::palette_chain::PaletteChainNode::new(
            self.get_palette(),
            self.core.palette_chain.clone(),
        );
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.page.set_palette_chain(Some(chain));
            tab.page.draw(terminal);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // A click on the strip switches tabs; a click anywhere else belongs to
        // the page.
        if event.what == EventType::MouseDown && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
            if let Some(index) = self.tab_at(event.mouse.pos) {
                self.set_active(index);
                event.clear();
                return;
            }
        }

        if event.what == EventType::Keyboard {
            let modifiers = event.key_modifiers;
            let ctrl = modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
            let shift = modifiers.contains(crossterm::event::KeyModifiers::SHIFT);

            let switched = match event.key_code {
                KB_F6 => self.cycle(!shift),
                KB_PGDN if ctrl => self.cycle(true),
                KB_PGUP if ctrl => self.cycle(false),
                _ => false,
            };
            if switched {
                event.clear();
                return;
            }
            if modifiers.contains(crossterm::event::KeyModifiers::ALT) {
                let letter = (event.key_code & 0xFF) as u8 as char;
                if let Some(index) = self.tab_for_hotkey(letter) {
                    self.set_active(index);
                    event.clear();
                    return;
                }
            }
        }

        // Everything else is the active page's business.
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.page.handle_event(event);
        }
    }

    /// Transparent, so a page's controls map their colours through whatever
    /// owns the pane, exactly as if they sat in the dialog directly. The strip
    /// resolves its own colours through a private painter instead.
    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating tabbed panes with a fluent API.
pub struct TabbedPaneBuilder {
    bounds: Option<Rect>,
    titles: Vec<String>,
    active: usize,
}

impl TabbedPaneBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            titles: Vec::new(),
            active: 0,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Add an empty page under `title`. Fill it in afterwards with
    /// [`TabbedPane::page_mut`].
    #[must_use]
    pub fn page(mut self, title: impl Into<String>) -> Self {
        self.titles.push(title.into());
        self
    }

    #[must_use]
    pub fn active(mut self, index: usize) -> Self {
        self.active = index;
        self
    }

    pub fn build(self) -> TabbedPane {
        let bounds = self.bounds.expect("TabbedPane bounds must be set");
        let mut pane = TabbedPane::new(bounds);
        let area = pane.page_area();
        for title in &self.titles {
            pane.add_page(title, Group::new(area));
        }
        pane.set_active(self.active);
        pane
    }

    pub fn build_boxed(self) -> Box<TabbedPane> {
        Box::new(self.build())
    }
}

impl Default for TabbedPaneBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{KB_DOWN, KB_TAB};
    use crate::views::group::GroupLike;
    use crossterm::event::KeyModifiers;

    fn pane() -> TabbedPane {
        TabbedPaneBuilder::new()
            .bounds(Rect::new(0, 0, 40, 12))
            .page("~G~eneral")
            .page("~A~dvanced")
            .page("About")
            .build()
    }

    fn ctrl(code: u16) -> Event {
        let mut e = Event::keyboard(code);
        e.key_modifiers = KeyModifiers::CONTROL;
        e
    }

    fn alt(letter: char) -> Event {
        let mut e = Event::keyboard(letter as u16);
        e.key_modifiers = KeyModifiers::ALT;
        e
    }

    fn click(x: i16, y: i16) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.pos = Point::new(x, y);
        e
    }

    #[test]
    fn titles_lose_their_tilde_markers() {
        let (label, hotkey, pos) = parse_title("~G~eneral");
        assert_eq!(label, "General");
        assert_eq!(hotkey, Some('g'));
        assert_eq!(pos, Some(0));
    }

    #[test]
    fn a_hotkey_can_sit_mid_word() {
        let (label, hotkey, pos) = parse_title("Ad~v~anced");
        assert_eq!(label, "Advanced");
        assert_eq!(hotkey, Some('v'));
        assert_eq!(pos, Some(2));
    }

    #[test]
    fn a_title_without_tildes_has_no_hotkey() {
        let (label, hotkey, _) = parse_title("About");
        assert_eq!(label, "About");
        assert_eq!(hotkey, None);
    }

    #[test]
    fn an_unterminated_tilde_does_not_panic() {
        let (label, hotkey, _) = parse_title("Trailing~");
        assert_eq!(label, "Trailing");
        assert_eq!(hotkey, None);
    }

    #[test]
    fn the_first_page_starts_active() {
        let p = pane();
        assert_eq!(p.page_count(), 3);
        assert_eq!(p.active(), 0);
        assert_eq!(p.active_title(), Some("~G~eneral"));
    }

    #[test]
    fn pages_sit_inside_the_frame_below_the_header() {
        let p = pane();
        assert_eq!(p.page_area(), Rect::new(1, 3, 39, 11));
    }

    #[test]
    fn out_of_range_activation_is_ignored() {
        let mut p = pane();
        assert!(!p.set_active(99));
        assert_eq!(p.active(), 0);
    }

    #[test]
    fn f6_cycles_forward_and_shift_f6_back() {
        let mut p = pane();
        let mut e = Event::keyboard(crate::core::event::KB_F6);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 1);
        assert_eq!(e.what, EventType::Nothing);

        let mut e = Event::keyboard(crate::core::event::KB_F6);
        e.key_modifiers = KeyModifiers::SHIFT;
        p.handle_event(&mut e);
        assert_eq!(p.active(), 0);
    }

    #[test]
    fn unmodified_page_keys_reach_the_page() {
        let mut p = pane();
        // A list on the page needs PgDn; only Ctrl+PgDn is the pane's.
        let mut e = Event::keyboard(KB_PGDN);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 0);
        assert_eq!(e.what, EventType::Keyboard);
    }

    #[test]
    fn ctrl_page_keys_cycle_and_wrap() {
        let mut p = pane();
        let mut e = ctrl(KB_PGDN);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 1);
        assert_eq!(e.what, EventType::Nothing, "the key was consumed");

        p.handle_event(&mut ctrl(KB_PGDN));
        p.handle_event(&mut ctrl(KB_PGDN));
        assert_eq!(p.active(), 0, "wrapped past the last tab");

        p.handle_event(&mut ctrl(KB_PGUP));
        assert_eq!(p.active(), 2, "wrapped backwards");
    }

    #[test]
    fn a_lone_tab_does_not_cycle() {
        let mut p = TabbedPaneBuilder::new()
            .bounds(Rect::new(0, 0, 20, 6))
            .page("Only")
            .build();
        let mut e = ctrl(KB_PGDN);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 0);
    }

    #[test]
    fn alt_hotkeys_select_their_tab() {
        let mut p = pane();
        let mut e = alt('a');
        p.handle_event(&mut e);
        assert_eq!(p.active(), 1);
        assert_eq!(e.what, EventType::Nothing);
    }

    #[test]
    fn an_unknown_hotkey_falls_through_to_the_page() {
        let mut p = pane();
        let mut e = alt('z');
        p.handle_event(&mut e);
        assert_eq!(p.active(), 0);
        assert_eq!(e.what, EventType::Keyboard, "left for the dialog");
    }

    #[test]
    fn plain_keys_reach_the_page_instead_of_switching_tabs() {
        let mut p = pane();
        // Arrows and Tab belong to the controls on the page.
        let mut e = Event::keyboard(KB_TAB);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 0);
        let mut e = Event::keyboard(KB_DOWN);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 0);
    }

    #[test]
    fn clicking_a_tab_selects_it() {
        let mut p = pane();
        // "General" is 7 wide plus its two walls and a frame corner, so tab 1
        // starts at cell 11. Its title row is the second of the three.
        let mut e = click(12, 1);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 1);
        assert_eq!(e.what, EventType::Nothing);
    }

    #[test]
    fn clicking_the_gap_between_tabs_changes_nothing() {
        let mut p = pane();
        // Cell 10 is the gap between "General" and "Files".
        let mut e = click(10, 1);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 0);
    }

    #[test]
    fn clicking_below_the_header_goes_to_the_page() {
        let mut p = pane();
        // Row 5 is inside the page, below the three header rows.
        let mut e = click(12, 5);
        p.handle_event(&mut e);
        assert_eq!(p.active(), 0, "not a tab click");
    }

    #[test]
    fn any_of_a_tabs_three_rows_selects_it() {
        for row in 0..3 {
            let mut p = pane();
            let mut e = click(12, row);
            p.handle_event(&mut e);
            assert_eq!(p.active(), 1, "row {row} of the tab box");
        }
    }

    #[test]
    fn tab_offsets_follow_the_title_widths() {
        let p = pane();
        // The first tab starts one cell in, past the frame corner. "General" is
        // 7 + 2 padding = 9, plus a 1-cell joint, so tab 1 starts at 11;
        // "Advanced" is 8 + 2 + 1, so tab 2 starts at 22.
        assert_eq!(p.tab_offsets(), vec![1, 11, 22]);
    }

    #[test]
    fn resizing_the_pane_resizes_every_page() {
        let mut p = pane();
        p.set_bounds(Rect::new(5, 5, 45, 20));
        assert_eq!(p.page_area(), Rect::new(6, 8, 44, 19));
        assert_eq!(p.page_mut(2).unwrap().bounds(), Rect::new(6, 8, 44, 19));
    }

    #[test]
    fn switching_away_and_back_keeps_a_page_intact() {
        let mut p = pane();
        p.page_mut(0)
            .unwrap()
            .add(crate::views::static_text::StaticText::new(
                Rect::new(1, 1, 10, 2),
                "hello",
            ));
        p.set_active(2);
        p.set_active(0);
        assert_eq!(p.page_mut(0).unwrap().len(), 1);
    }
}
