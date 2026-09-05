// (C) 2026 - Enzo Lombardi

//! ScrollPane - scrolling interior for oversized dialogs (TV Tool Box style).

use turbo_vision::core::event::{Event, EventType, MB_LEFT_BUTTON};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::state::State;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::GroupLike;
use turbo_vision::views::group::Group;
use turbo_vision::views::view::ViewId;
use turbo_vision::views::{View, ViewCore};

/// Ctrl+Up / Ctrl+Down scroll the pane a row at a time.
const KB_CTRL_UP: u16 = 0x8D00;
const KB_CTRL_DOWN: u16 = 0x9100;

/// A viewport over a virtual area larger than its screen bounds.
///
/// TV Tool Box called these "scrolling dialog boxes": place more controls
/// than fit on screen inside a virtual rectangle, and the pane scrolls
/// them into view. Children are added with coordinates relative to the
/// virtual area; scrolling repositions them and clips drawing to the
/// pane's bounds. Tab focus changes automatically scroll the focused
/// control into view; Ctrl+Up / Ctrl+Down scroll manually.
///
/// # Examples
///
/// ```
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision_extras::ScrollPane;
///
/// // A 40x10 window over a 40x30 virtual form
/// let pane = ScrollPane::new(Rect::new(0, 0, 40, 10), 30);
/// assert_eq!(pane.scroll_offset(), 0);
/// ```
pub struct ScrollPane {
    core: ViewCore,
    /// Virtual height in rows (>= visible height).
    virtual_height: i16,
    /// Current vertical scroll offset in rows.
    offset: i16,
    group: Group,
    /// Virtual (unscrolled) bounds per child, parallel to the group.
    virtual_bounds: Vec<Rect>,
    palette_chain: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
}

impl std::fmt::Debug for ScrollPane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollPane")
            .field("bounds", &self.core.bounds)
            .field("virtual_height", &self.virtual_height)
            .field("offset", &self.offset)
            .field("children", &self.group.len())
            .finish()
    }
}

impl ScrollPane {
    /// Create a pane whose virtual area is `virtual_height` rows tall.
    pub fn new(bounds: Rect, virtual_height: i16) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: State::empty(),
                ..ViewCore::default()
            },
            virtual_height: virtual_height.max(bounds.height()),
            offset: 0,
            // The group fills the pane's own space, so its children are
            // pane-relative and it draws with no further origin push.
            group: Group::new(Rect::new(0, 0, bounds.width(), bounds.height())),
            virtual_bounds: Vec::new(),
            palette_chain: None,
        }
    }

    /// Add a child at `virtual_rect` (relative to the virtual area).
    pub fn add(&mut self, mut view: Box<dyn View>, virtual_rect: Rect) -> ViewId {
        self.virtual_bounds.push(virtual_rect);
        // Position on screen for the current offset; Group::add treats the
        // bounds as relative to the group, which shares our bounds
        let on_screen = Rect::new(
            virtual_rect.a.x,
            virtual_rect.a.y - self.offset,
            virtual_rect.b.x,
            virtual_rect.b.y - self.offset,
        );
        view.set_bounds(on_screen);
        self.group.add(view)
    }

    /// Current vertical scroll offset in rows.
    pub fn scroll_offset(&self) -> i16 {
        self.offset
    }

    /// Maximum scroll offset.
    fn max_offset(&self) -> i16 {
        (self.virtual_height - self.core.bounds.height()).max(0)
    }

    /// Scroll to an absolute offset, repositioning every child.
    pub fn scroll_to(&mut self, offset: i16) {
        let new_offset = offset.clamp(0, self.max_offset());
        let delta = new_offset - self.offset;
        if delta == 0 {
            return;
        }
        self.offset = new_offset;
        for i in 0..self.group.len() {
            let b = self.group.child_at(i).bounds();
            self.group.child_at_mut(i).set_bounds(Rect::new(
                b.a.x,
                b.a.y - delta,
                b.b.x,
                b.b.y - delta,
            ));
        }
    }

    /// Scroll by a number of rows (positive = down).
    pub fn scroll_by(&mut self, rows: i16) {
        self.scroll_to(self.offset + rows);
    }

    /// Scroll so the focused child is fully visible.
    fn scroll_focused_into_view(&mut self) {
        let Some(focused) = self.group.focused_child() else {
            return;
        };
        let b = focused.bounds();
        if b.a.y < 0 {
            self.scroll_by(b.a.y);
        } else if b.b.y > self.extent().b.y {
            self.scroll_by(b.b.y - self.extent().b.y);
        }
    }

    /// Access the interior group.
    pub fn group(&self) -> &Group {
        &self.group
    }

    /// Mutable access to the interior group.
    pub fn group_mut(&mut self) -> &mut Group {
        &mut self.group
    }
}

impl View for ScrollPane {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.core.bounds = bounds;
        self.group.set_bounds(self.extent());
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Clip so partially scrolled-out children don't paint outside
        terminal.push_clip(self.extent());
        self.group.draw(terminal);
        terminal.pop_clip();
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::Keyboard => match event.key_code {
                KB_CTRL_UP => {
                    self.scroll_by(-1);
                    event.clear();
                    return;
                }
                KB_CTRL_DOWN => {
                    self.scroll_by(1);
                    event.clear();
                    return;
                }
                _ => {}
            },
            EventType::MouseWheelUp => {
                self.scroll_by(-1);
                event.clear();
                return;
            }
            EventType::MouseWheelDown => {
                self.scroll_by(1);
                event.clear();
                return;
            }
            EventType::MouseDown => {
                // Clicks outside the visible pane never reach hidden children
                if event.mouse.buttons & MB_LEFT_BUTTON != 0
                    && !self.extent().contains(event.mouse.pos)
                {
                    return;
                }
            }
            _ => {}
        }

        self.group.handle_event(event);

        // A Tab/Shift+Tab focus change may have landed on a control that is
        // scrolled out of view — bring it back in
        if event.what == EventType::Nothing {
            self.scroll_focused_into_view();
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.set_state_flag(turbo_vision::core::state::State::FOCUSED, focused);
        if focused {
            self.group.set_initial_focus();
        } else {
            self.group.clear_all_focus();
        }
    }

    fn update_cursor(&self, terminal: &mut Terminal) {
        if let Some(focused) = self.group.focused_child() {
            // Only show the cursor for controls scrolled into view
            let b = focused.bounds();
            if b.a.y >= 0 && b.b.y <= self.extent().b.y {
                focused.update_cursor(terminal);
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
    use turbo_vision::views::static_text::StaticText;

    fn make() -> ScrollPane {
        // 20 rows visible over a 60-row virtual form
        let mut pane = ScrollPane::new(Rect::new(0, 5, 40, 25), 60);
        pane.add(
            Box::new(StaticText::new(Rect::new(1, 0, 20, 1), "top")),
            Rect::new(1, 0, 20, 1),
        );
        pane.add(
            Box::new(StaticText::new(Rect::new(1, 50, 20, 51), "bottom")),
            Rect::new(1, 50, 20, 51),
        );
        pane
    }

    #[test]
    fn scrolling_repositions_children_and_clamps() {
        let mut pane = make();
        let top_before = pane.group().child_at(0).bounds();

        pane.scroll_by(10);
        assert_eq!(pane.scroll_offset(), 10);
        let top_after = pane.group().child_at(0).bounds();
        assert_eq!(top_after.a.y, top_before.a.y - 10);

        // Clamped at virtual_height - visible = 60 - 20 = 40
        pane.scroll_by(1000);
        assert_eq!(pane.scroll_offset(), 40);
        pane.scroll_by(-1000);
        assert_eq!(pane.scroll_offset(), 0);
        let top_restored = pane.group().child_at(0).bounds();
        assert_eq!(top_restored, top_before);
    }

    #[test]
    fn wheel_events_scroll() {
        let mut pane = make();
        let mut ev = Event::nothing();
        ev.what = EventType::MouseWheelDown;
        pane.handle_event(&mut ev);
        assert_eq!(pane.scroll_offset(), 1);
        assert_eq!(ev.what, EventType::Nothing);
    }
}
