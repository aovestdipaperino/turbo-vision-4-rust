// (C) 2025 - Enzo Lombardi

//! Tooltip view - hover hints for the controls of a dialog.
//!
//! Not part of the Borland Turbo Vision widget set, which pushed hints to the
//! status line instead. One `Tooltip` serves a whole dialog: register a rect and
//! a line of text per control, and the pointer resting on one of them raises the
//! hint beside it.
//!
//! Two things it needs from its owner:
//!
//! - **Add it last.** A tooltip draws over its neighbours, and a group draws its
//!   children in order, so it must be the final view added to the dialog.
//! - **A running event loop.** The hover delay is driven by the `CM_IDLE_TICK`
//!   broadcast that `Application::idle` sends, so the hint appears a moment
//!   after the pointer settles rather than the instant it crosses a control.
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::views::tooltip::Tooltip;
//! use turbo_vision::core::geometry::Rect;
//!
//! // The tooltip may draw anywhere in the dialog's interior.
//! let mut tips = Tooltip::new(Rect::new(0, 0, 60, 20));
//! tips.add_hint(Rect::new(2, 3, 20, 4), "Where the output is written");
//! assert_eq!(tips.hint_count(), 1);
//! ```

use super::view::{View, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType};
use crate::core::geometry::{Point, Rect};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use std::time::{Duration, Instant};

/// How long the pointer must rest on a control before its hint appears.
const DEFAULT_DELAY: Duration = Duration::from_millis(600);

/// Blank cells on each side of the hint text.
const HINT_PADDING: usize = 1;

/// Label-palette entry for "light" text, which reads as a raised note against
/// the dialog background.
const LABEL_LIGHT: u8 = 2;

/// One control's hint.
struct Hint {
    /// Screen rect of the control this describes.
    target: Rect,
    text: String,
}

/// Hover hints for the controls of a dialog.
pub struct Tooltip {
    /// The area the popup may be drawn in, normally the dialog's interior.
    bounds: Rect,
    hints: Vec<Hint>,
    /// Which hint the pointer is over, and when it arrived.
    hover: Option<(usize, Instant)>,
    /// Which hint is currently shown.
    shown: Option<usize>,
    delay: Duration,
    view_state: StateFlags,
    palette_chain: Option<crate::core::palette_chain::PaletteChainNode>,
}

impl Tooltip {
    /// Create an empty tooltip that may draw anywhere within `bounds`.
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            hints: Vec::new(),
            hover: None,
            shown: None,
            delay: DEFAULT_DELAY,
            view_state: 0,
            palette_chain: None,
        }
    }

    /// Register a hint for the control occupying `target`.
    ///
    /// The rect is in the same coordinates the tooltip itself is given, so
    /// register hints after the dialog has placed its controls, using each
    /// control's own bounds.
    pub fn add_hint(&mut self, target: Rect, text: impl Into<String>) {
        self.hints.push(Hint {
            target,
            text: text.into(),
        });
    }

    /// Number of registered hints.
    pub fn hint_count(&self) -> usize {
        self.hints.len()
    }

    /// Drop every hint and hide anything showing.
    pub fn clear_hints(&mut self) {
        self.hints.clear();
        self.hover = None;
        self.shown = None;
    }

    /// How long the pointer must rest before a hint appears.
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = delay;
    }

    /// Whether a hint is on screen.
    pub fn is_showing(&self) -> bool {
        self.shown.is_some()
    }

    /// The text currently on screen, if any.
    pub fn shown_text(&self) -> Option<&str> {
        self.shown.and_then(|i| self.hints.get(i)).map(|h| &*h.text)
    }

    /// Hide whatever is showing and forget the hover.
    ///
    /// Called whenever the user does something: a hint that stayed up over a
    /// control the user has started typing into would be in the way.
    pub fn hide(&mut self) {
        self.hover = None;
        self.shown = None;
    }

    /// Index of the hint whose target covers `pos`.
    ///
    /// Later hints win, so a hint registered for a control inside another one
    /// takes precedence over the outer one.
    fn hint_at(&self, pos: Point) -> Option<usize> {
        self.hints.iter().rposition(|h| h.target.contains(pos))
    }

    /// Note where the pointer is. Moving to a different control restarts the
    /// delay, so dragging across a row of buttons does not flash a hint on each.
    fn track(&mut self, pos: Point) {
        match self.hint_at(pos) {
            Some(index) => {
                let same = self.hover.is_some_and(|(i, _)| i == index);
                if !same {
                    self.hover = Some((index, Instant::now()));
                    self.shown = None;
                }
            }
            None => self.hide(),
        }
    }

    /// Raise the hint once the pointer has rested long enough.
    fn tick(&mut self) {
        if self.shown.is_some() {
            return;
        }
        if let Some((index, since)) = self.hover {
            if since.elapsed() >= self.delay {
                self.shown = Some(index);
            }
        }
    }

    /// Where the popup for `index` goes: one row, just below its control, or
    /// above it when there is no room, and pushed left to fit the bounds.
    fn popup_area(&self, index: usize) -> Option<Rect> {
        let hint = self.hints.get(index)?;
        let width = hint.text.chars().count() + HINT_PADDING * 2;
        let width = (width as i16).min(self.bounds.width()).max(1);

        let mut x = hint.target.a.x;
        if x + width > self.bounds.b.x {
            x = (self.bounds.b.x - width).max(self.bounds.a.x);
        }

        let mut y = hint.target.b.y;
        if y >= self.bounds.b.y {
            // No room below: sit above the control instead.
            y = hint.target.a.y - 1;
        }
        if y < self.bounds.a.y || y >= self.bounds.b.y {
            return None;
        }

        Some(Rect::new(x, y, x + width, y + 1))
    }
}

impl View for Tooltip {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    /// A tooltip is never focused; it only watches the pointer.
    fn can_focus(&self) -> bool {
        false
    }

    fn state(&self) -> StateFlags {
        self.view_state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.view_state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let Some(index) = self.shown else {
            return;
        };
        let Some(area) = self.popup_area(index) else {
            return;
        };
        let width = area.width_clamped().max(0) as usize;
        if width == 0 {
            return;
        }

        let attr = self.map_color(LABEL_LIGHT);
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', attr, width);
        if HINT_PADDING < width {
            let room = width - HINT_PADDING;
            let shown: String = self.hints[index].text.chars().take(room).collect();
            buf.move_str(HINT_PADDING, &shown, attr);
        }
        write_line_to_terminal(terminal, area.a.x, area.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseMove => {
                self.track(event.mouse.pos);
                // The move is not consumed: other views track the pointer too.
            }
            // Any real interaction takes the hint down.
            EventType::MouseDown | EventType::Keyboard => self.hide(),
            EventType::Broadcast => {
                if event.command == crate::core::command::CM_IDLE_TICK {
                    self.tick();
                    // Left alone on purpose: a broadcast stops travelling once
                    // it is consumed, and other views want this tick too.
                }
            }
            _ => {}
        }
    }

    fn set_palette_chain(&mut self, node: Option<crate::core::palette_chain::PaletteChainNode>) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

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

/// Builder for creating tooltips with a fluent API.
pub struct TooltipBuilder {
    bounds: Option<Rect>,
    hints: Vec<(Rect, String)>,
    delay: Duration,
}

impl TooltipBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            hints: Vec::new(),
            delay: DEFAULT_DELAY,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn hint(mut self, target: Rect, text: impl Into<String>) -> Self {
        self.hints.push((target, text.into()));
        self
    }

    #[must_use]
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn build(self) -> Tooltip {
        let bounds = self.bounds.expect("Tooltip bounds must be set");
        let mut tip = Tooltip::new(bounds);
        tip.set_delay(self.delay);
        for (target, text) in self.hints {
            tip.add_hint(target, text);
        }
        tip
    }

    pub fn build_boxed(self) -> Box<Tooltip> {
        Box::new(self.build())
    }
}

impl Default for TooltipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tips() -> Tooltip {
        let mut t = Tooltip::new(Rect::new(0, 0, 40, 12));
        t.add_hint(Rect::new(2, 2, 12, 3), "First control");
        t.add_hint(Rect::new(2, 5, 12, 6), "Second control");
        // No delay, so a single move-then-tick is enough to raise a hint.
        t.set_delay(Duration::ZERO);
        t
    }

    fn move_to(t: &mut Tooltip, x: i16, y: i16) {
        let mut e = Event::nothing();
        e.what = EventType::MouseMove;
        e.mouse.pos = Point::new(x, y);
        t.handle_event(&mut e);
    }

    fn tick(t: &mut Tooltip) {
        let mut e = Event::broadcast(crate::core::command::CM_IDLE_TICK);
        t.handle_event(&mut e);
    }

    #[test]
    fn nothing_shows_until_the_pointer_arrives() {
        let mut t = tips();
        assert!(!t.is_showing());
        tick(&mut t);
        assert!(!t.is_showing());
    }

    #[test]
    fn resting_on_a_control_raises_its_hint() {
        let mut t = tips();
        move_to(&mut t, 5, 2);
        assert!(!t.is_showing(), "not until a tick has passed");
        tick(&mut t);
        assert_eq!(t.shown_text(), Some("First control"));
    }

    #[test]
    fn the_delay_must_actually_elapse() {
        let mut t = tips();
        t.set_delay(Duration::from_secs(30));
        move_to(&mut t, 5, 2);
        tick(&mut t);
        assert!(!t.is_showing());
    }

    #[test]
    fn moving_off_a_control_takes_the_hint_down() {
        let mut t = tips();
        move_to(&mut t, 5, 2);
        tick(&mut t);
        assert!(t.is_showing());
        move_to(&mut t, 30, 9);
        assert!(!t.is_showing());
    }

    #[test]
    fn moving_between_controls_restarts_the_delay() {
        let mut t = tips();
        move_to(&mut t, 5, 2);
        tick(&mut t);
        move_to(&mut t, 5, 5);
        assert!(!t.is_showing(), "the second hint has not waited yet");
        tick(&mut t);
        assert_eq!(t.shown_text(), Some("Second control"));
    }

    #[test]
    fn staying_on_one_control_does_not_restart_the_delay() {
        let mut t = tips();
        t.set_delay(Duration::from_millis(1));
        move_to(&mut t, 5, 2);
        move_to(&mut t, 6, 2);
        std::thread::sleep(Duration::from_millis(2));
        tick(&mut t);
        assert!(
            t.is_showing(),
            "a nudge within the same control still counts"
        );
    }

    #[test]
    fn typing_or_clicking_takes_the_hint_down() {
        for mut event in [
            {
                let mut e = Event::nothing();
                e.what = EventType::MouseDown;
                e
            },
            Event::keyboard(crate::core::event::KB_DOWN),
        ] {
            let mut t = tips();
            move_to(&mut t, 5, 2);
            tick(&mut t);
            assert!(t.is_showing());
            t.handle_event(&mut event);
            assert!(!t.is_showing());
        }
    }

    #[test]
    fn the_mouse_move_is_left_for_other_views() {
        let mut t = tips();
        let mut e = Event::nothing();
        e.what = EventType::MouseMove;
        e.mouse.pos = Point::new(5, 2);
        t.handle_event(&mut e);
        assert_eq!(e.what, EventType::MouseMove);
    }

    #[test]
    fn the_idle_tick_is_left_for_other_views() {
        let mut t = tips();
        let mut e = Event::broadcast(crate::core::command::CM_IDLE_TICK);
        t.handle_event(&mut e);
        assert_eq!(e.what, EventType::Broadcast);
    }

    #[test]
    fn the_popup_sits_under_its_control() {
        let t = tips();
        let area = t.popup_area(0).unwrap();
        assert_eq!(area.a, Point::new(2, 3), "the row below the control");
        assert_eq!(area.width(), "First control".len() as i16 + 2);
    }

    #[test]
    fn a_popup_near_the_right_edge_is_pushed_left() {
        let mut t = Tooltip::new(Rect::new(0, 0, 20, 10));
        t.add_hint(Rect::new(16, 2, 19, 3), "A long hint");
        let area = t.popup_area(0).unwrap();
        assert!(area.b.x <= 20, "stayed inside the bounds: {area:?}");
    }

    #[test]
    fn a_popup_with_no_room_below_flips_above() {
        let mut t = Tooltip::new(Rect::new(0, 0, 20, 10));
        t.add_hint(Rect::new(2, 9, 10, 10), "Bottom control");
        let area = t.popup_area(0).unwrap();
        assert_eq!(area.a.y, 8, "above the control");
    }

    #[test]
    fn overlapping_hints_prefer_the_one_added_later() {
        let mut t = Tooltip::new(Rect::new(0, 0, 40, 12));
        t.set_delay(Duration::ZERO);
        t.add_hint(Rect::new(0, 0, 20, 6), "The group");
        t.add_hint(Rect::new(2, 2, 10, 3), "The control inside it");
        move_to(&mut t, 5, 2);
        tick(&mut t);
        assert_eq!(t.shown_text(), Some("The control inside it"));
    }

    #[test]
    fn clearing_hints_takes_anything_showing_down() {
        let mut t = tips();
        move_to(&mut t, 5, 2);
        tick(&mut t);
        t.clear_hints();
        assert_eq!(t.hint_count(), 0);
        assert!(!t.is_showing());
    }

    #[test]
    fn builder_configures_the_tooltip() {
        let t = TooltipBuilder::new()
            .bounds(Rect::new(0, 0, 30, 8))
            .hint(Rect::new(1, 1, 5, 2), "Hint")
            .delay(Duration::from_millis(50))
            .build();
        assert_eq!(t.hint_count(), 1);
        assert_eq!(t.delay, Duration::from_millis(50));
    }
}
