// (C) 2025 - Enzo Lombardi

//! SplitPane view - two panes divided by a draggable splitter.
//!
//! Not part of the Borland Turbo Vision widget set. The roadmap called this a
//! splitter sitting between two sibling views, but a bare divider cannot resize
//! siblings it does not own: a child view has no handle on the others in its
//! group. So this owns both halves, each a [`Group`], the same way
//! [`TabbedPane`](super::tabbed_pane::TabbedPane) owns its pages.
//!
//! Build each half's group with [`SplitPane::first_area`] and
//! [`SplitPane::second_area`], which report where the halves currently sit.
//!
//! # Keys and mouse
//!
//! | Input | Action |
//! |-------|--------|
//! | Drag the divider | Move it, within both halves' minimum sizes |
//! | Click either half | Focus it |
//! | F8 | Move focus to the other half |
//!
//! Moving the divider from the keyboard is left to the host, which can call
//! [`SplitPane::grow_first`] and [`SplitPane::shrink_first`] from a command of
//! its own. Binding it here would have to steal a key from the controls inside
//! the panes.
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::views::split_pane::{Orientation, SplitPane};
//! use turbo_vision::views::group::Group;
//! use turbo_vision::core::geometry::Rect;
//!
//! let mut split = SplitPane::new(Rect::new(0, 0, 40, 10), Orientation::Vertical, 20);
//! let first = Group::new(split.first_area());
//! let second = Group::new(split.second_area());
//! split.set_panes(first, second);
//! assert_eq!(split.position(), 20);
//! ```

use super::group::Group;
use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, KB_F8, MB_LEFT_BUTTON};
use crate::core::geometry::{Point, Rect};
use crate::core::state::{State, StateFlags};
use crate::terminal::Terminal;

/// The divider drawn between two side-by-side panes.
const DIVIDER_VERTICAL: char = '\u{2502}';
/// The divider drawn between two stacked panes.
const DIVIDER_HORIZONTAL: char = '\u{2500}';
/// Cells the divider itself occupies.
const DIVIDER_SIZE: i16 = 1;

/// Label-palette entry for disabled text, which in a gray dialog resolves to
/// dark grey on the dialog background. The divider uses it so it reads as quiet
/// structure rather than as black text.
const LABEL_DIMMED: u8 = 5;

/// Which way the two panes sit relative to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Side by side, divided by a vertical line.
    Vertical,
    /// Stacked, divided by a horizontal line.
    Horizontal,
}

/// Two panes divided by a draggable splitter.
pub struct SplitPane {
    core: ViewCore,
    orientation: Orientation,
    /// Cells given to the first pane, measured from the pane's own edge.
    position: i16,
    /// Smallest the first pane may become.
    min_first: i16,
    /// Smallest the second pane may become.
    min_second: i16,
    first: Group,
    second: Group,
    /// True while the second pane holds the focus.
    focus_second: bool,
    view_state: StateFlags,
}

impl SplitPane {
    /// Create a split pane with empty halves and the divider `position` cells in.
    ///
    /// Both halves start with a minimum size of one cell. Fill them in with
    /// [`SplitPane::set_panes`], or reach them later with
    /// [`SplitPane::first_mut`] and [`SplitPane::second_mut`].
    pub fn new(bounds: Rect, orientation: Orientation, position: i16) -> Self {
        let mut split = Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            orientation,
            position,
            min_first: 1,
            min_second: 1,
            first: Group::new(Rect::new(0, 0, 0, 0)),
            second: Group::new(Rect::new(0, 0, 0, 0)),
            focus_second: false,
            view_state: State::empty(),
        };
        split.position = split.clamp_position(position);
        split.first.set_bounds(split.first_area());
        split.second.set_bounds(split.second_area());
        split
    }

    /// Install both halves, moving each into place.
    pub fn set_panes(&mut self, first: Group, second: Group) {
        self.first = first;
        self.second = second;
        self.layout();
    }

    /// The first half: the left pane, or the top one.
    pub fn first_mut(&mut self) -> &mut Group {
        &mut self.first
    }

    /// The second half: the right pane, or the bottom one.
    pub fn second_mut(&mut self) -> &mut Group {
        &mut self.second
    }

    /// Which way the panes sit.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Cells currently given to the first pane.
    pub fn position(&self) -> i16 {
        self.position
    }

    /// Move the divider, clamped so neither half falls below its minimum.
    ///
    /// Returns true when the divider actually moved.
    pub fn set_position(&mut self, position: i16) -> bool {
        let clamped = self.clamp_position(position);
        if clamped == self.position {
            return false;
        }
        self.position = clamped;
        self.layout();
        true
    }

    /// Set the smallest size each half may be squeezed to, then re-clamp the
    /// divider. Minimums that cannot both be honoured are shared out from the
    /// first pane, which keeps the divider inside the pane either way.
    pub fn set_minimums(&mut self, first: i16, second: i16) {
        self.min_first = first.max(0);
        self.min_second = second.max(0);
        let position = self.position;
        self.position = self.clamp_position(position);
        self.layout();
    }

    /// Give the first pane one more cell.
    pub fn grow_first(&mut self) -> bool {
        self.set_position(self.position + 1)
    }

    /// Give the first pane one fewer cell.
    pub fn shrink_first(&mut self) -> bool {
        self.set_position(self.position - 1)
    }

    /// Whether the second half currently holds the focus.
    pub fn second_focused(&self) -> bool {
        self.focus_second
    }

    /// Move the focus to the other half.
    pub fn focus_other(&mut self) {
        self.focus_half(!self.focus_second);
    }

    /// Give the focused half's first control the focus.
    pub fn set_initial_focus(&mut self) {
        self.focus_half(self.focus_second);
    }

    /// Focus one half and drop the other's.
    fn focus_half(&mut self, second: bool) {
        self.focus_second = second;
        if second {
            self.first.clear_all_focus();
            self.second.set_initial_focus();
        } else {
            self.second.clear_all_focus();
            self.first.set_initial_focus();
        }
    }

    /// Total cells across the split: width for a vertical divider, height for a
    /// horizontal one.
    fn span(&self) -> i16 {
        match self.orientation {
            Orientation::Vertical => self.core.bounds.width(),
            Orientation::Horizontal => self.core.bounds.height(),
        }
    }

    /// Clamp a divider position so both halves keep their minimum size.
    ///
    /// When the pane is too small to honour both minimums, the first pane's
    /// bound wins and the second is squeezed; the divider always stays inside.
    fn clamp_position(&self, position: i16) -> i16 {
        let span = self.span();
        let highest = (span - DIVIDER_SIZE - self.min_second).max(0);
        position.clamp(self.min_first.min(highest), highest)
    }

    /// The rect the first half occupies.
    pub fn first_area(&self) -> Rect {
        match self.orientation {
            Orientation::Vertical => Rect::new(
                self.core.bounds.a.x,
                self.core.bounds.a.y,
                self.core.bounds.a.x + self.position,
                self.core.bounds.b.y,
            ),
            Orientation::Horizontal => Rect::new(
                self.core.bounds.a.x,
                self.core.bounds.a.y,
                self.core.bounds.b.x,
                self.core.bounds.a.y + self.position,
            ),
        }
    }

    /// The rect the second half occupies.
    pub fn second_area(&self) -> Rect {
        match self.orientation {
            Orientation::Vertical => Rect::new(
                self.core.bounds.a.x + self.position + DIVIDER_SIZE,
                self.core.bounds.a.y,
                self.core.bounds.b.x,
                self.core.bounds.b.y,
            ),
            Orientation::Horizontal => Rect::new(
                self.core.bounds.a.x,
                self.core.bounds.a.y + self.position + DIVIDER_SIZE,
                self.core.bounds.b.x,
                self.core.bounds.b.y,
            ),
        }
    }

    /// The divider's own rect: one cell thick, spanning the pane.
    pub fn divider_area(&self) -> Rect {
        match self.orientation {
            Orientation::Vertical => Rect::new(
                self.core.bounds.a.x + self.position,
                self.core.bounds.a.y,
                self.core.bounds.a.x + self.position + DIVIDER_SIZE,
                self.core.bounds.b.y,
            ),
            Orientation::Horizontal => Rect::new(
                self.core.bounds.a.x,
                self.core.bounds.a.y + self.position,
                self.core.bounds.b.x,
                self.core.bounds.a.y + self.position + DIVIDER_SIZE,
            ),
        }
    }

    /// Push both halves to where the divider currently puts them.
    fn layout(&mut self) {
        let first = self.first_area();
        let second = self.second_area();
        self.first.set_bounds(first);
        self.second.set_bounds(second);
    }

    /// Divider position implied by a mouse at `pos`.
    fn position_for(&self, pos: Point) -> i16 {
        match self.orientation {
            Orientation::Vertical => pos.x - self.core.bounds.a.x,
            Orientation::Horizontal => pos.y - self.core.bounds.a.y,
        }
    }

    fn is_dragging(&self) -> bool {
        self.view_state.contains(State::DRAGGING)
    }

    fn set_dragging(&mut self, dragging: bool) {
        self.set_state_flag(State::DRAGGING, dragging);
    }
}

impl View for SplitPane {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.core.bounds = bounds;
        // Keep the divider inside the new size before moving the halves.
        let position = self.position;
        self.position = self.clamp_position(position);
        self.layout();
    }

    /// The pane takes focus on behalf of the half that holds it, then hands it
    /// straight to that half's controls.
    fn can_focus(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.set_state_flag(State::FOCUSED, focused);
        if focused {
            self.focus_half(self.focus_second);
        } else {
            self.first.clear_all_focus();
            self.second.clear_all_focus();
        }
    }

    fn state(&self) -> StateFlags {
        self.view_state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.view_state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let chain = crate::core::palette_chain::PaletteChainNode::new(
            self.get_palette(),
            self.core.palette_chain.clone(),
        );
        self.first.set_palette_chain(Some(chain.clone()));
        self.first.draw(terminal);
        self.second.set_palette_chain(Some(chain));
        self.second.draw(terminal);

        // The divider is drawn last so neither half can paint over it.
        let painter = DividerPainter {
            core: ViewCore {
                palette_chain: self.core.palette_chain.clone(),
                ..ViewCore::default()
            },
        };
        let attr = painter.map_color(LABEL_DIMMED);
        let area = self.divider_area();
        match self.orientation {
            Orientation::Vertical => {
                let mut buf = DrawBuffer::new(1);
                buf.put_char(0, DIVIDER_VERTICAL, attr);
                for y in area.a.y..area.b.y {
                    write_line_to_terminal(terminal, area.a.x, y, &buf);
                }
            }
            Orientation::Horizontal => {
                let width = area.width_clamped().max(0) as usize;
                if width > 0 {
                    let mut buf = DrawBuffer::new(width);
                    buf.move_char(0, DIVIDER_HORIZONTAL, attr, width);
                    write_line_to_terminal(terminal, area.a.x, area.a.y, &buf);
                }
            }
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // A drag in progress owns every mouse event, even once the pointer has
        // left the divider; the group forwards them here while State::DRAGGING is
        // set on the focused child.
        if self.is_dragging() {
            match event.what {
                EventType::MouseMove => {
                    let target = self.position_for(event.mouse.pos);
                    self.set_position(target);
                    event.clear();
                    return;
                }
                EventType::MouseUp => {
                    self.set_dragging(false);
                    event.clear();
                    return;
                }
                _ => {}
            }
        }

        if event.what == EventType::MouseDown && event.mouse.buttons & MB_LEFT_BUTTON != 0 {
            if self.divider_area().contains(event.mouse.pos) {
                self.set_dragging(true);
                event.clear();
                return;
            }
            // A click in either half focuses it before the half sees the click,
            // so the control under the pointer ends up focused.
            if self.first_area().contains(event.mouse.pos) {
                self.focus_half(false);
            } else if self.second_area().contains(event.mouse.pos) {
                self.focus_half(true);
            }
        }

        if event.what == EventType::Keyboard && event.key_code == KB_F8 {
            self.focus_other();
            event.clear();
            return;
        }

        // Everything else belongs to the focused half.
        if self.focus_second {
            self.second.handle_event(event);
        } else {
            self.first.handle_event(event);
        }
    }

    /// Transparent, so each half's controls map their colours through whatever
    /// owns the split, exactly as if they sat in the dialog directly.
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

/// Resolves the divider's colour against the split pane's chain.
///
/// The pane is transparent so its halves inherit the owner's palette unchanged;
/// this borrows the chain to resolve the one dimmed entry the divider needs. It
/// never joins the view hierarchy.
struct DividerPainter {
    core: ViewCore,
}

impl View for DividerPainter {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, _terminal: &mut Terminal) {}

    fn handle_event(&mut self, _event: &mut Event) {}

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_LABEL))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating split panes with a fluent API.
pub struct SplitPaneBuilder {
    bounds: Option<Rect>,
    orientation: Orientation,
    position: Option<i16>,
    min_first: i16,
    min_second: i16,
}

impl SplitPaneBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            orientation: Orientation::Vertical,
            position: None,
            min_first: 1,
            min_second: 1,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Cells given to the first pane. Defaults to half the span.
    #[must_use]
    pub fn position(mut self, position: i16) -> Self {
        self.position = Some(position);
        self
    }

    #[must_use]
    pub fn minimums(mut self, first: i16, second: i16) -> Self {
        self.min_first = first;
        self.min_second = second;
        self
    }

    pub fn build(self) -> SplitPane {
        let bounds = self.bounds.expect("SplitPane bounds must be set");
        let span = match self.orientation {
            Orientation::Vertical => bounds.width(),
            Orientation::Horizontal => bounds.height(),
        };
        let position = self.position.unwrap_or(span / 2);
        let mut split = SplitPane::new(bounds, self.orientation, position);
        split.set_minimums(self.min_first, self.min_second);
        split.set_position(position);
        split
    }

    pub fn build_boxed(self) -> Box<SplitPane> {
        Box::new(self.build())
    }
}

impl Default for SplitPaneBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split() -> SplitPane {
        // 40 wide, divider at 20.
        SplitPane::new(Rect::new(0, 0, 40, 10), Orientation::Vertical, 20)
    }

    fn click(x: i16, y: i16) -> Event {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.pos = Point::new(x, y);
        e
    }

    fn mouse(what: EventType, x: i16, y: i16) -> Event {
        let mut e = Event::nothing();
        e.what = what;
        e.mouse.pos = Point::new(x, y);
        e
    }

    #[test]
    fn the_halves_flank_the_divider() {
        let s = split();
        assert_eq!(s.first_area(), Rect::new(0, 0, 20, 10));
        assert_eq!(s.divider_area(), Rect::new(20, 0, 21, 10));
        assert_eq!(s.second_area(), Rect::new(21, 0, 40, 10));
    }

    #[test]
    fn a_horizontal_split_stacks_the_halves() {
        let s = SplitPane::new(Rect::new(0, 0, 40, 10), Orientation::Horizontal, 4);
        assert_eq!(s.first_area(), Rect::new(0, 0, 40, 4));
        assert_eq!(s.divider_area(), Rect::new(0, 4, 40, 5));
        assert_eq!(s.second_area(), Rect::new(0, 5, 40, 10));
    }

    #[test]
    fn the_divider_stays_inside_the_pane() {
        let mut s = split();
        s.set_position(999);
        assert_eq!(
            s.position(),
            38,
            "the divider plus the second pane's one-cell minimum"
        );
        s.set_position(-5);
        assert_eq!(s.position(), 1, "the first pane keeps its minimum");
    }

    #[test]
    fn minimums_are_honoured_on_both_sides() {
        let mut s = split();
        s.set_minimums(8, 12);
        s.set_position(0);
        assert_eq!(s.position(), 8);
        s.set_position(999);
        assert_eq!(s.position(), 27, "40 less the divider less 12");
    }

    #[test]
    fn impossible_minimums_keep_the_divider_in_range() {
        let mut s = SplitPane::new(Rect::new(0, 0, 10, 5), Orientation::Vertical, 5);
        // Both halves cannot have 20 cells in a 10-cell pane.
        s.set_minimums(20, 20);
        assert!(s.position() >= 0 && s.position() < 10);
    }

    #[test]
    fn growing_and_shrinking_move_one_cell() {
        let mut s = split();
        assert!(s.grow_first());
        assert_eq!(s.position(), 21);
        assert!(s.shrink_first());
        assert_eq!(s.position(), 20);
    }

    #[test]
    fn a_move_that_changes_nothing_reports_false() {
        let mut s = split();
        assert!(!s.set_position(20));
    }

    #[test]
    fn resizing_the_pane_re_clamps_the_divider() {
        let mut s = split();
        s.set_bounds(Rect::new(0, 0, 12, 10));
        assert_eq!(s.position(), 10, "the divider followed the shrinking pane");
        assert_eq!(s.second_area(), Rect::new(11, 0, 12, 10));
    }

    #[test]
    fn resizing_moves_both_halves() {
        let mut s = split();
        s.set_bounds(Rect::new(5, 5, 45, 15));
        assert_eq!(s.first_area(), Rect::new(5, 5, 25, 15));
        assert_eq!(s.first.bounds(), Rect::new(5, 5, 25, 15));
        assert_eq!(s.second.bounds(), Rect::new(26, 5, 45, 15));
    }

    #[test]
    fn pressing_the_divider_starts_a_drag() {
        let mut s = split();
        let mut e = click(20, 3);
        s.handle_event(&mut e);
        assert!(s.is_dragging());
        assert_eq!(e.what, EventType::Nothing);
    }

    #[test]
    fn dragging_moves_the_divider_and_releasing_ends_it() {
        let mut s = split();
        s.handle_event(&mut click(20, 3));
        s.handle_event(&mut mouse(EventType::MouseMove, 30, 3));
        assert_eq!(s.position(), 30);
        s.handle_event(&mut mouse(EventType::MouseUp, 30, 3));
        assert!(!s.is_dragging());
    }

    #[test]
    fn a_drag_past_the_edge_clamps_instead_of_escaping() {
        let mut s = split();
        s.set_minimums(5, 5);
        s.handle_event(&mut click(20, 3));
        s.handle_event(&mut mouse(EventType::MouseMove, 100, 3));
        assert_eq!(s.position(), 34, "40 less the divider less 5");
    }

    #[test]
    fn clicking_a_half_focuses_it() {
        let mut s = split();
        assert!(!s.second_focused());
        s.handle_event(&mut click(30, 3));
        assert!(s.second_focused());
        s.handle_event(&mut click(5, 3));
        assert!(!s.second_focused());
    }

    #[test]
    fn f8_moves_focus_to_the_other_half() {
        let mut s = split();
        let mut e = Event::keyboard(KB_F8);
        s.handle_event(&mut e);
        assert!(s.second_focused());
        assert_eq!(e.what, EventType::Nothing);
        s.handle_event(&mut Event::keyboard(KB_F8));
        assert!(!s.second_focused());
    }

    #[test]
    fn other_keys_reach_the_focused_half() {
        let mut s = split();
        let mut e = Event::keyboard(crate::core::event::KB_DOWN);
        s.handle_event(&mut e);
        // Both halves are empty, so nothing consumed it.
        assert_eq!(e.what, EventType::Keyboard);
    }

    #[test]
    fn builder_defaults_the_divider_to_the_middle() {
        let s = SplitPaneBuilder::new()
            .bounds(Rect::new(0, 0, 30, 8))
            .build();
        assert_eq!(s.position(), 15);
    }

    #[test]
    fn builder_applies_minimums_before_the_position() {
        let s = SplitPaneBuilder::new()
            .bounds(Rect::new(0, 0, 30, 8))
            .minimums(10, 10)
            .position(2)
            .build();
        assert_eq!(s.position(), 10, "clamped up to the first minimum");
    }
}
