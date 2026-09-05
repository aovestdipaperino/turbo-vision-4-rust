// (C) 2025 - Enzo Lombardi

//! ScrollBar view - vertical or horizontal scrollbar with draggable indicator.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_DOWN, KB_END, KB_HOME, KB_LEFT, KB_PGDN, KB_PGUP, KB_RIGHT, KB_UP,
    MB_LEFT_BUTTON,
};
use crate::core::geometry::{Point, Rect};
use crate::core::palette::{SCROLLBAR_INDICATOR, SCROLLBAR_PAGE};
use crate::terminal::Terminal;
use std::time::{Duration, Instant};

/// Scroll bar part codes (used by getPartCode() method)
const SB_INDICATOR: i16 = 0;
const SB_UP_ARROW: i16 = 1;
const SB_DOWN_ARROW: i16 = 2;
const SB_PAGE_UP: i16 = 3;
const SB_PAGE_DOWN: i16 = 4;

/// Scroll bar characters for vertical scrollbar
pub const VSCROLL_CHARS: [char; 5] = [
    '█', // Indicator
    '▲', // Up arrow
    '▼', // Down arrow
    '░', // Page up area
    '░', // Page down area
];

/// Scroll bar characters for horizontal scrollbar
pub const HSCROLL_CHARS: [char; 5] = [
    '█', // Indicator
    '◄', // Left arrow
    '►', // Right arrow
    '░', // Page left area
    '░', // Page right area
];

/// How long a button must be held before the press starts repeating.
const REPEAT_DELAY: Duration = Duration::from_millis(400);
/// Gap between repeats once they start.
const REPEAT_INTERVAL: Duration = Duration::from_millis(80);

/// What a held mouse button keeps doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepeatAction {
    /// One arrow step, up or left.
    StepBack,
    /// One arrow step, down or right.
    StepForward,
    /// One page, towards the start.
    PageBack,
    /// One page, towards the end.
    PageForward,
}

pub struct ScrollBar {
    core: ViewCore,
    value: i32,
    min_val: i32,
    max_val: i32,
    pg_step: i32, // Page step
    ar_step: i32, // Arrow step
    total: i32,   // Total content size (lines or columns) for proportional thumb
    chars: [char; 5],
    is_vertical: bool,
    dragging_thumb: bool,
    /// What the held button is repeating, and when the next repeat is due.
    repeat: Option<(RepeatAction, Instant)>,
}

impl ScrollBar {
    pub fn new_vertical(bounds: Rect) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            value: 0,
            min_val: 0,
            max_val: 0,
            pg_step: 1,
            ar_step: 1,
            total: 0,
            chars: VSCROLL_CHARS,
            is_vertical: true,
            dragging_thumb: false,
            repeat: None,
        }
    }

    pub fn new_horizontal(bounds: Rect) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            value: 0,
            min_val: 0,
            max_val: 0,
            pg_step: 1,
            ar_step: 1,
            total: 0,
            chars: HSCROLL_CHARS,
            is_vertical: false,
            dragging_thumb: false,
            repeat: None,
        }
    }

    pub fn set_params(
        &mut self,
        value: i32,
        min_val: i32,
        max_val: i32,
        pg_step: i32,
        ar_step: i32,
    ) {
        // Ensure max_val >= min_val to prevent division by zero
        self.min_val = min_val;
        self.max_val = max_val.max(min_val);
        self.value = value.max(self.min_val).min(self.max_val);
        self.pg_step = pg_step;
        self.ar_step = ar_step;
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value.max(self.min_val).min(self.max_val);
    }

    pub fn set_range(&mut self, min_val: i32, max_val: i32) {
        self.min_val = min_val;
        self.max_val = max_val;
        self.value = self.value.max(min_val).min(max_val);
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// Set total content size (lines or columns) for proportional thumb sizing.
    pub fn set_total(&mut self, total: i32) {
        self.total = total.max(0);
    }

    /// Get the size of the scrollbar track (not including arrows)
    fn get_size(&self) -> i32 {
        if self.is_vertical {
            (self.core.bounds.height() - 2).max(1) as i32
        } else {
            (self.core.bounds.width() - 2).max(1) as i32
        }
    }

    /// Thumb size in track cells: each content unit (line/column) maps to
    /// `track / total` cells.
    fn get_thumb_size(&self) -> i32 {
        let track = self.get_size();
        // Nothing to scroll: no thumb at all (Borland draws a flat bar)
        if self.total <= 0 || self.max_val <= self.min_val {
            return 0;
        }
        let thumb = (track as i64 / self.total as i64) as i32;
        thumb.max(1).min(track)
    }

    /// Get the position of the indicator (first cell of the thumb).
    fn get_pos(&self) -> i32 {
        let track = self.get_size();
        let thumb = self.get_thumb_size();
        let usable = track - thumb;
        let range = self.max_val - self.min_val;
        if range <= 0 || usable <= 0 {
            0
        } else {
            ((self.value - self.min_val) as i64 * usable as i64 / range as i64)
                .max(0)
                .min(usable as i64) as i32
        }
    }

    /// Get the part of the scrollbar at a given position
    #[expect(
        dead_code,
        reason = "Borland TV API - reserved for advanced scrollbar interaction"
    )]
    fn get_part_at(&self, p: Point) -> i16 {
        let rel_x = p.x - self.core.bounds.a.x;
        let rel_y = p.y - self.core.bounds.a.y;

        if self.is_vertical {
            if rel_y == 0 {
                SB_UP_ARROW
            } else if rel_y == self.core.bounds.height() - 1 {
                SB_DOWN_ARROW
            } else {
                let pos = self.get_pos();
                if rel_y - 1 == pos as i16 {
                    SB_INDICATOR
                } else if rel_y - 1 < pos as i16 {
                    SB_PAGE_UP
                } else {
                    SB_PAGE_DOWN
                }
            }
        } else if rel_x == 0 {
            SB_UP_ARROW // Left arrow for horizontal
        } else if rel_x == self.core.bounds.width() - 1 {
            SB_DOWN_ARROW // Right arrow for horizontal
        } else {
            let pos = self.get_pos();
            if rel_x - 1 == pos as i16 {
                SB_INDICATOR
            } else if rel_x - 1 < pos as i16 {
                SB_PAGE_UP // Page left
            } else {
                SB_PAGE_DOWN // Page right
            }
        }
    }

    /// Scroll by a given part
    #[expect(
        dead_code,
        reason = "Borland TV API - reserved for advanced scrollbar interaction"
    )]
    fn scroll_step(&mut self, part: i16) -> i32 {
        match part {
            SB_UP_ARROW => -self.ar_step,
            SB_DOWN_ARROW => self.ar_step,
            SB_PAGE_UP => -self.pg_step,
            SB_PAGE_DOWN => self.pg_step,
            _ => 0,
        }
    }

    fn keypress(&mut self, event: &mut Event) {
        if self.is_vertical {
            match event.key_code {
                KB_UP => {
                    self.value = (self.value - self.ar_step).max(self.min_val);
                    event.clear();
                }
                KB_DOWN => {
                    self.value = (self.value + self.ar_step).min(self.max_val);
                    event.clear();
                }
                KB_PGUP => {
                    self.value = (self.value - self.pg_step).max(self.min_val);
                    event.clear();
                }
                KB_PGDN => {
                    self.value = (self.value + self.pg_step).min(self.max_val);
                    event.clear();
                }
                KB_HOME => {
                    self.value = self.min_val;
                    event.clear();
                }
                KB_END => {
                    self.value = self.max_val;
                    event.clear();
                }
                _ => {}
            }
        } else {
            match event.key_code {
                KB_LEFT => {
                    self.value = (self.value - self.ar_step).max(self.min_val);
                    event.clear();
                }
                KB_RIGHT => {
                    self.value = (self.value + self.ar_step).min(self.max_val);
                    event.clear();
                }
                KB_HOME => {
                    self.value = self.min_val;
                    event.clear();
                }
                KB_END => {
                    self.value = self.max_val;
                    event.clear();
                }
                _ => {}
            }
        }
    }
    fn drag(&mut self, event: &mut Event) {
        if self.is_vertical {
            // Update thumb position based on mouse Y
            let mouse_y = event.mouse.pos.y;
            let rel_y = (mouse_y - self.core.bounds.a.y - 1) as i32; // Relative to track start
            let range = self.max_val - self.min_val + 1;
            let s = self.get_size();
            log::debug!(
                "Dragging thumb to rel_y: {}, size: {}, range: {}, max_val: {}, min_val: {}",
                rel_y,
                s,
                range,
                self.max_val,
                self.min_val
            );
            if s > 0 && range > 0 {
                let new_pos = rel_y.max(0).min(s);
                log::debug!("Calculated new thumb position: {}", new_pos);
                self.value = (new_pos * range / s) + self.min_val;
            }
        } else {
            // Horizontal scrollbar
            let mouse_x = event.mouse.pos.x;
            let rel_x = (mouse_x - self.core.bounds.a.x - 1) as i32; // Relative to track start
            let range = self.max_val - self.min_val + 1;
            let s = self.get_size();
            log::debug!(
                "Dragging thumb to rel_x: {}, size: {}, range: {}, max_val: {}, min_val: {}",
                rel_x,
                s,
                range,
                self.max_val,
                self.min_val
            );
            if s > 0 && range > 0 {
                let new_pos = rel_x.max(0).min(s);
                log::debug!("Calculated new thumb position: {}", new_pos);
                self.value = (new_pos * range / s) + self.min_val;
            }
        }
    }
    /// Apply one repeat step. Returns true when the value actually moved, so a
    /// repeat that has run into the end of the range can stop.
    fn apply(&mut self, action: RepeatAction) -> bool {
        let before = self.value;
        self.value = match action {
            RepeatAction::StepBack => (self.value - self.ar_step).max(self.min_val),
            RepeatAction::StepForward => (self.value + self.ar_step).min(self.max_val),
            RepeatAction::PageBack => (self.value - self.pg_step).max(self.min_val),
            RepeatAction::PageForward => (self.value + self.pg_step).min(self.max_val),
        };
        self.value != before
    }

    /// Arm auto-repeat for a press that has just been applied once.
    fn arm_repeat(&mut self, action: RepeatAction) {
        self.repeat = Some((action, Instant::now() + REPEAT_DELAY));
    }

    /// Fire any repeat that has come due. Called from the
    /// `CM_MOUSE_AUTO_REPEAT` broadcast the application sends while a button is
    /// held down.
    ///
    /// Returns true when the value changed.
    fn tick_repeat(&mut self) -> bool {
        let Some((action, due)) = self.repeat else {
            return false;
        };
        if Instant::now() < due {
            return false;
        }
        if !self.apply(action) {
            // Ran into the end of the range: stop repeating rather than
            // spinning for as long as the button is held.
            self.repeat = None;
            return false;
        }
        self.repeat = Some((action, Instant::now() + REPEAT_INTERVAL));
        true
    }

    fn left_click(&mut self, event: &mut Event) {
        let mouse_pos = event.mouse.pos;

        if self.is_vertical {
            if mouse_pos.x >= self.core.bounds.a.x
                && mouse_pos.x < self.core.bounds.b.x
                && mouse_pos.y >= self.core.bounds.a.y
                && mouse_pos.y < self.core.bounds.b.y
            {
                let rel_y = mouse_pos.y - self.core.bounds.a.y;
                let height = self.core.bounds.height();

                if rel_y == 0 {
                    self.apply(RepeatAction::StepBack);
                    self.arm_repeat(RepeatAction::StepBack);
                    event.clear();
                } else if rel_y == height - 1 {
                    self.apply(RepeatAction::StepForward);
                    self.arm_repeat(RepeatAction::StepForward);
                    event.clear();
                } else {
                    let range = self.max_val - self.min_val;
                    if range > 0 {
                        let thumb_pos = self.get_pos() as i16;
                        let thumb_sz = self.get_thumb_size() as i16;
                        let track_y = rel_y - 1;

                        if track_y < thumb_pos {
                            self.apply(RepeatAction::PageBack);
                            self.arm_repeat(RepeatAction::PageBack);
                        } else if track_y >= thumb_pos + thumb_sz {
                            self.apply(RepeatAction::PageForward);
                            self.arm_repeat(RepeatAction::PageForward);
                        } else {
                            self.dragging_thumb = true;
                        }
                        event.clear();
                    }
                }
            }
        } else {
            if mouse_pos.y >= self.core.bounds.a.y
                && mouse_pos.y < self.core.bounds.b.y
                && mouse_pos.x >= self.core.bounds.a.x
                && mouse_pos.x < self.core.bounds.b.x
            {
                let rel_x = mouse_pos.x - self.core.bounds.a.x;
                let width = self.core.bounds.width();

                if rel_x == 0 {
                    self.apply(RepeatAction::StepBack);
                    self.arm_repeat(RepeatAction::StepBack);
                    event.clear();
                } else if rel_x == width - 1 {
                    self.apply(RepeatAction::StepForward);
                    self.arm_repeat(RepeatAction::StepForward);
                    event.clear();
                } else {
                    let range = self.max_val - self.min_val;
                    if range > 0 {
                        let thumb_pos = self.get_pos() as i16;
                        let thumb_sz = self.get_thumb_size() as i16;
                        let track_x = rel_x - 1;

                        if track_x < thumb_pos {
                            self.apply(RepeatAction::PageBack);
                            self.arm_repeat(RepeatAction::PageBack);
                        } else if track_x >= thumb_pos + thumb_sz {
                            self.apply(RepeatAction::PageForward);
                            self.arm_repeat(RepeatAction::PageForward);
                        } else {
                            self.dragging_thumb = true;
                        }
                        event.clear();
                    }
                }
            }
        }
    }
}

impl View for ScrollBar {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // ScrollBar palette indices:
        // 1: Page, 2: Arrows, 3: Indicator
        let page_attr = self.map_color(SCROLLBAR_PAGE);
        let indicator_attr = self.map_color(SCROLLBAR_INDICATOR);

        let pos = self.get_pos();
        let thumb = self.get_thumb_size();
        let thumb_start = pos as i16;
        let thumb_end = thumb_start + thumb as i16;

        if self.is_vertical {
            let height = self.core.bounds.height();

            for y in 0..height {
                let mut buf = DrawBuffer::new(1);
                let track_pos = y - 1;
                let in_thumb =
                    track_pos >= thumb_start && track_pos < thumb_end && y > 0 && y < height - 1;

                let ch = if y == 0 {
                    self.chars[1]
                } else if y == height - 1 {
                    self.chars[2]
                } else if in_thumb {
                    self.chars[0]
                } else {
                    self.chars[3]
                };

                let attr = if in_thumb { indicator_attr } else { page_attr };
                buf.put_char(0, ch, attr);
                write_line_to_terminal(
                    terminal,
                    self.core.bounds.a.x,
                    self.core.bounds.a.y + y,
                    &buf,
                );
            }
        } else {
            let width = self.core.bounds.width();
            let mut buf = DrawBuffer::new(width as usize);

            for x in 0..width {
                let track_pos = x - 1;
                let in_thumb =
                    track_pos >= thumb_start && track_pos < thumb_end && x > 0 && x < width - 1;

                let ch = if x == 0 {
                    self.chars[1]
                } else if x == width - 1 {
                    self.chars[2]
                } else if in_thumb {
                    self.chars[0]
                } else {
                    self.chars[3]
                };

                let attr = if in_thumb { indicator_attr } else { page_attr };
                buf.put_char(x as usize, ch, attr);
            }

            write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        log::debug!("ScrollBar received event: {:?}", event);
        match event.what {
            EventType::MouseMove => {
                if self.dragging_thumb {
                    self.drag(event);
                    event.clear();
                }
            }
            EventType::Keyboard => {
                self.keypress(event);
            }
            EventType::MouseDown => {
                if (event.mouse.buttons & MB_LEFT_BUTTON) != 0 {
                    self.left_click(event);
                }
            }
            EventType::MouseUp => {
                // Releasing ends both a thumb drag and any auto-repeat.
                self.repeat = None;
                if self.dragging_thumb {
                    self.dragging_thumb = false;
                    event.clear();
                }
            }
            EventType::Broadcast => {
                if event.command == crate::core::command::CM_MOUSE_AUTO_REPEAT {
                    self.tick_repeat();
                    // The broadcast is left alone: other views may be repeating
                    // too, and a broadcast reaches every child.
                }
            }
            _ => {}
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_SCROLLBAR))
    }
}

/// Builder for creating scrollbars with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::scrollbar::ScrollBarBuilder;
/// use turbo_vision::core::geometry::Rect;
///
/// // Create a vertical scrollbar
/// let scrollbar = ScrollBarBuilder::new()
///     .bounds(Rect::new(78, 1, 79, 20))
///     .vertical()
///     .params(0, 0, 100, 10, 1)
///     .build();
///
/// // Create a horizontal scrollbar
/// let scrollbar = ScrollBarBuilder::new()
///     .bounds(Rect::new(1, 23, 78, 24))
///     .horizontal()
///     .build();
/// ```
pub struct ScrollBarBuilder {
    bounds: Option<Rect>,
    is_vertical: bool,
    value: i32,
    min_val: i32,
    max_val: i32,
    pg_step: i32,
    ar_step: i32,
}

impl ScrollBarBuilder {
    /// Creates a new ScrollBarBuilder with default values (vertical orientation).
    pub fn new() -> Self {
        Self {
            bounds: None,
            is_vertical: true,
            value: 0,
            min_val: 0,
            max_val: 0,
            pg_step: 1,
            ar_step: 1,
        }
    }

    /// Sets the scrollbar bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the scrollbar to vertical orientation.
    #[must_use]
    pub fn vertical(mut self) -> Self {
        self.is_vertical = true;
        self
    }

    /// Sets the scrollbar to horizontal orientation.
    #[must_use]
    pub fn horizontal(mut self) -> Self {
        self.is_vertical = false;
        self
    }

    /// Sets the scrollbar parameters.
    #[must_use]
    pub fn params(
        mut self,
        value: i32,
        min_val: i32,
        max_val: i32,
        pg_step: i32,
        ar_step: i32,
    ) -> Self {
        self.value = value;
        self.min_val = min_val;
        self.max_val = max_val;
        self.pg_step = pg_step;
        self.ar_step = ar_step;
        self
    }

    /// Sets the initial value.
    #[must_use]
    pub fn value(mut self, value: i32) -> Self {
        self.value = value;
        self
    }

    /// Sets the range (min and max values).
    #[must_use]
    pub fn range(mut self, min_val: i32, max_val: i32) -> Self {
        self.min_val = min_val;
        self.max_val = max_val;
        self
    }

    /// Sets the page step.
    #[must_use]
    pub fn page_step(mut self, pg_step: i32) -> Self {
        self.pg_step = pg_step;
        self
    }

    /// Sets the arrow step.
    #[must_use]
    pub fn arrow_step(mut self, ar_step: i32) -> Self {
        self.ar_step = ar_step;
        self
    }

    /// Builds the ScrollBar.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds) are not set.
    pub fn build(self) -> ScrollBar {
        let bounds = self.bounds.expect("ScrollBar bounds must be set");

        let chars = if self.is_vertical {
            VSCROLL_CHARS
        } else {
            HSCROLL_CHARS
        };

        ScrollBar {
            core: ViewCore::new(bounds),
            value: self
                .value
                .max(self.min_val)
                .min(self.max_val.max(self.min_val)),
            min_val: self.min_val,
            max_val: self.max_val.max(self.min_val),
            pg_step: self.pg_step,
            ar_step: self.ar_step,
            total: 0,
            chars,
            is_vertical: self.is_vertical,
            dragging_thumb: false,
            repeat: None,
        }
    }

    /// Builds the ScrollBar as a Box.
    pub fn build_boxed(self) -> Box<ScrollBar> {
        Box::new(self.build())
    }
}

impl Default for ScrollBarBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_thumb_when_nothing_to_scroll() {
        // Regression: a zero range drew a full-length thumb
        let mut bar = ScrollBar::new_vertical(Rect::new(0, 0, 1, 12));
        bar.set_params(0, 0, 0, 10, 1);
        assert_eq!(bar.get_thumb_size(), 0);

        // With a real range the thumb reappears
        bar.set_params(0, 0, 50, 10, 1);
        bar.set_total(100);
        assert!(bar.get_thumb_size() >= 1);
    }

    // --- Mouse auto-repeat ----------------------------------------------

    fn repeat_bar() -> ScrollBar {
        let mut bar = ScrollBar::new_vertical(Rect::new(0, 0, 1, 10));
        bar.set_params(50, 0, 100, 10, 1);
        // Without a total there is no thumb at all, so a press anywhere on the
        // track pages instead of dragging.
        bar.set_total(8);
        bar
    }

    fn press(bar: &mut ScrollBar, y: i16) {
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.buttons = MB_LEFT_BUTTON;
        e.mouse.pos = Point::new(0, y);
        bar.handle_event(&mut e);
    }

    fn release(bar: &mut ScrollBar) {
        let mut e = Event::nothing();
        e.what = EventType::MouseUp;
        bar.handle_event(&mut e);
    }

    fn auto_repeat(bar: &mut ScrollBar) {
        let mut e = Event::broadcast(crate::core::command::CM_MOUSE_AUTO_REPEAT);
        bar.handle_event(&mut e);
    }

    /// Pretend the hold has lasted long enough for the next repeat.
    fn make_repeat_due(bar: &mut ScrollBar) {
        if let Some((action, _)) = bar.repeat {
            bar.repeat = Some((action, Instant::now()));
        }
    }

    #[test]
    fn pressing_an_arrow_steps_once_and_arms_a_repeat() {
        let mut bar = repeat_bar();
        press(&mut bar, 0);
        assert_eq!(bar.get_value(), 49);
        assert!(bar.repeat.is_some());
    }

    #[test]
    fn a_repeat_waits_for_the_initial_delay() {
        let mut bar = repeat_bar();
        press(&mut bar, 0);
        auto_repeat(&mut bar);
        assert_eq!(bar.get_value(), 49, "still inside the delay");
    }

    #[test]
    fn a_held_arrow_keeps_stepping_once_the_delay_passes() {
        let mut bar = repeat_bar();
        press(&mut bar, 0);
        for _ in 0..3 {
            make_repeat_due(&mut bar);
            auto_repeat(&mut bar);
        }
        assert_eq!(bar.get_value(), 46);
    }

    #[test]
    fn releasing_stops_the_repeat() {
        let mut bar = repeat_bar();
        press(&mut bar, 0);
        release(&mut bar);
        assert!(bar.repeat.is_none());
        auto_repeat(&mut bar);
        assert_eq!(bar.get_value(), 49);
    }

    #[test]
    fn a_repeat_stops_at_the_end_of_the_range() {
        let mut bar = repeat_bar();
        bar.set_value(1);
        press(&mut bar, 0);
        assert_eq!(bar.get_value(), 0);
        make_repeat_due(&mut bar);
        auto_repeat(&mut bar);
        assert_eq!(bar.get_value(), 0);
        assert!(bar.repeat.is_none(), "no point repeating into the end stop");
    }

    #[test]
    fn pressing_the_track_repeats_by_the_page() {
        let mut bar = repeat_bar();
        // Row 1 is the top of the track, above a thumb sitting mid-bar.
        press(&mut bar, 1);
        let after_press = bar.get_value();
        assert!(after_press < 50, "paged towards the start");
        make_repeat_due(&mut bar);
        auto_repeat(&mut bar);
        assert!(bar.get_value() < after_press, "kept paging");
    }

    #[test]
    fn dragging_the_thumb_arms_no_repeat() {
        let mut bar = repeat_bar();
        // The thumb sits around the middle for a value of 50.
        let thumb_row = bar.get_pos() as i16 + 1;
        press(&mut bar, thumb_row);
        assert!(bar.dragging_thumb, "the press landed on the thumb");
        assert!(bar.repeat.is_none(), "a drag is not a repeating press");
    }

    #[test]
    fn the_repeat_broadcast_is_left_for_other_views() {
        let mut bar = repeat_bar();
        press(&mut bar, 0);
        let mut e = Event::broadcast(crate::core::command::CM_MOUSE_AUTO_REPEAT);
        bar.handle_event(&mut e);
        assert_eq!(e.what, EventType::Broadcast);
    }
}
