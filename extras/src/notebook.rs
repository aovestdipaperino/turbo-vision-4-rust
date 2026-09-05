// (C) 2026 - Enzo Lombardi

//! Notebook - tabbed pages (TMBASIC-inspired).

use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{Event, EventType, MB_LEFT_BUTTON};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::group::Group;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{View, ViewCore};

/// Ctrl+PgDn / Ctrl+PgUp switch to the next/previous tab.
const KB_CTRL_PGDN: u16 = 0x7600;
const KB_CTRL_PGUP: u16 = 0x8400;

/// Tabbed page container: a row of tab captions over one visible page.
///
/// Each page is a [`Group`]; add controls to a page with
/// [`add_to_page`](Self::add_to_page) using coordinates relative to the
/// page area (the notebook bounds minus the tab row). Click a tab or use
/// Ctrl+PgDn / Ctrl+PgUp to switch pages. Only the active page draws and
/// receives events.
///
/// # Examples
///
/// ```
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision_extras::Notebook;
///
/// let mut notebook = Notebook::new(Rect::new(0, 0, 60, 15));
/// notebook.add_page("General");
/// notebook.add_page("Advanced");
/// assert_eq!(notebook.active_page(), 0);
/// notebook.set_active_page(1);
/// assert_eq!(notebook.active_page(), 1);
/// ```
pub struct Notebook {
    core: ViewCore,
    labels: Vec<String>,
    pages: Vec<Group>,
    active: usize,
    palette_chain: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
}

impl std::fmt::Debug for Notebook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Notebook")
            .field("bounds", &self.core.bounds)
            .field("pages", &self.labels)
            .field("active", &self.active)
            .finish()
    }
}

impl Notebook {
    /// Create an empty notebook.
    pub fn new(bounds: Rect) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: 0,
                ..ViewCore::default()
            },
            labels: Vec::new(),
            pages: Vec::new(),
            active: 0,
            palette_chain: None,
        }
    }

    /// Screen area of the pages (bounds minus the tab row).
    fn page_bounds(&self) -> Rect {
        Rect::new(
            self.core.bounds.a.x,
            self.core.bounds.a.y + 1,
            self.core.bounds.b.x,
            self.core.bounds.b.y,
        )
    }

    /// Append a page and return its index.
    pub fn add_page(&mut self, label: impl Into<String>) -> usize {
        self.labels.push(label.into());
        self.pages.push(Group::new(self.page_bounds()));
        self.pages.len() - 1
    }

    /// Add a view to a page. Bounds are relative to the page area.
    pub fn add_to_page(&mut self, page: usize, view: Box<dyn View>) {
        if let Some(group) = self.pages.get_mut(page) {
            group.add(view);
        }
    }

    /// Number of pages.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Index of the visible page.
    pub fn active_page(&self) -> usize {
        self.active
    }

    /// Switch pages, moving focus into the newly visible page.
    pub fn set_active_page(&mut self, page: usize) {
        if page < self.pages.len() && page != self.active {
            if let Some(old) = self.pages.get_mut(self.active) {
                old.clear_all_focus();
            }
            self.active = page;
            if self.is_focused() {
                self.pages[self.active].set_initial_focus();
            }
        }
    }

    /// Access the active page's group (e.g. to reach child views).
    pub fn active_group(&self) -> Option<&Group> {
        self.pages.get(self.active)
    }

    /// Mutable access to the active page's group.
    pub fn active_group_mut(&mut self) -> Option<&mut Group> {
        self.pages.get_mut(self.active)
    }

    /// Tab caption cell ranges: (start_x, end_x) per tab, bounds-relative.
    fn tab_spans(&self) -> Vec<(i16, i16)> {
        let mut spans = Vec::with_capacity(self.labels.len());
        let mut x = 0i16;
        for label in &self.labels {
            let w = label.chars().count() as i16 + 2; // " label "
            spans.push((x, x + w));
            x += w + 1;
        }
        spans
    }
}

impl View for Notebook {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.core.bounds = bounds;
        let page_bounds = self.page_bounds();
        for page in &mut self.pages {
            page.set_bounds(page_bounds);
        }
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        if width == 0 {
            return;
        }

        let tab_attr = Attr::new(TvColor::Black, TvColor::Cyan);
        let active_attr = Attr::new(TvColor::White, TvColor::Blue);

        // Tab row
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', tab_attr, width);
        let spans = self.tab_spans();
        for (i, ((start, _end), label)) in spans.iter().zip(&self.labels).enumerate() {
            let attr = if i == self.active {
                active_attr
            } else {
                tab_attr
            };
            let text = format!(" {label} ");
            if (*start as usize) < width {
                buf.move_str(*start as usize, &text, attr);
            }
        }
        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);

        // Active page
        if let Some(page) = self.pages.get_mut(self.active) {
            page.draw(terminal);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard if self.is_focused() => match event.key_code {
                KB_CTRL_PGDN => {
                    let next = (self.active + 1) % self.pages.len().max(1);
                    self.set_active_page(next);
                    event.clear();
                    return;
                }
                KB_CTRL_PGUP => {
                    let count = self.pages.len().max(1);
                    self.set_active_page((self.active + count - 1) % count);
                    event.clear();
                    return;
                }
                _ => {}
            },
            EventType::MouseDown => {
                let pos = event.mouse.pos;
                if event.mouse.buttons & MB_LEFT_BUTTON != 0 && pos.y == self.core.bounds.a.y {
                    let rel_x = pos.x - self.core.bounds.a.x;
                    for (i, (start, end)) in self.tab_spans().iter().enumerate() {
                        if rel_x >= *start && rel_x < *end {
                            self.set_active_page(i);
                            event.clear();
                            return;
                        }
                    }
                }
            }
            _ => {}
        }

        // Everything else goes to the active page
        if let Some(page) = self.pages.get_mut(self.active) {
            page.handle_event(event);
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.set_state_flag(turbo_vision::core::state::SF_FOCUSED, focused);
        if let Some(page) = self.pages.get_mut(self.active) {
            if focused {
                page.set_initial_focus();
            } else {
                page.clear_all_focus();
            }
        }
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }

    fn set_palette_chain(
        &mut self,
        node: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
    ) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&turbo_vision::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::geometry::Point;

    fn make() -> Notebook {
        let mut nb = Notebook::new(Rect::new(0, 0, 40, 10));
        nb.add_page("One");
        nb.add_page("Two");
        nb.add_page("Three");
        nb.set_focus(true);
        nb
    }

    #[test]
    fn ctrl_page_keys_cycle_tabs() {
        let mut nb = make();
        let mut ev = Event::keyboard(KB_CTRL_PGDN);
        nb.handle_event(&mut ev);
        assert_eq!(nb.active_page(), 1);
        let mut ev = Event::keyboard(KB_CTRL_PGUP);
        nb.handle_event(&mut ev);
        assert_eq!(nb.active_page(), 0);
        // Wraps backwards
        let mut ev = Event::keyboard(KB_CTRL_PGUP);
        nb.handle_event(&mut ev);
        assert_eq!(nb.active_page(), 2);
    }

    #[test]
    fn clicking_a_tab_activates_it() {
        let mut nb = make();
        // Tabs: " One "(0..5) sep " Two "(6..11)
        let mut ev = Event::mouse(
            EventType::MouseDown,
            Point::new(7, 0),
            MB_LEFT_BUTTON,
            false,
        );
        nb.handle_event(&mut ev);
        assert_eq!(nb.active_page(), 1);
        assert_eq!(ev.what, EventType::Nothing);
    }

    #[test]
    fn pages_hold_views() {
        let mut nb = make();
        nb.add_to_page(
            0,
            Box::new(turbo_vision::views::static_text::StaticText::new(
                Rect::new(1, 1, 20, 2),
                "hello",
            )),
        );
        assert_eq!(nb.active_group().map(|g| g.len()), Some(1));
    }
}
