// (C) 2025 - Enzo Lombardi

//! Spinner view - a numeric field with up and down steppers.
//!
//! Not part of the Borland Turbo Vision widget set. It edits one integer held
//! within a range, so it never has to reject what the user typed the way an
//! `InputLine` with a `RangeValidator` does: out-of-range input is clamped as
//! it is entered.
//!
//! # Keys
//!
//! | Key | Action |
//! |-----|--------|
//! | Up, Down | Step by one step |
//! | PgUp, PgDn | Step by ten steps |
//! | Home, End | Jump to the minimum or maximum |
//! | Digits | Append a digit, clamped to the range |
//! | Minus | Negate, when the range allows it |
//! | Backspace | Drop the last digit |
//!
//! Clicking the up or down glyph steps in that direction.
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::views::spinner::Spinner;
//! use turbo_vision::core::geometry::Rect;
//!
//! let mut spin = Spinner::new(Rect::new(10, 3, 22, 4), 0, 100);
//! spin.set_value(42);
//! spin.step_up();
//! assert_eq!(spin.value(), 43);
//! ```

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command::CommandId;
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_BACKSPACE, KB_DOWN, KB_END, KB_HOME, KB_PGDN, KB_PGUP, KB_UP,
};
use crate::core::geometry::Rect;
use crate::core::palette::{INPUT_ARROWS, INPUT_NORMAL, INPUT_SELECTED};
use crate::core::state::{State, StateFlags};
use crate::terminal::Terminal;

/// Glyph for the increment stepper.
const UP_ARROW: char = '\u{25B2}'; // ▲
/// Glyph for the decrement stepper.
const DOWN_ARROW: char = '\u{25BC}'; // ▼
/// Cells the stepper pair occupies at the right edge.
const STEPPER_WIDTH: usize = 2;
/// Multiplier applied to the step for PgUp and PgDn.
const PAGE_FACTOR: i64 = 10;

/// A numeric field with up and down steppers.
pub struct Spinner {
    core: ViewCore,
    value: i64,
    min: i64,
    max: i64,
    step: i64,
    /// Text appended after the number, such as a unit.
    suffix: String,
    /// Whether stepping past an end continues from the other one.
    wrap: bool,
    /// Command broadcast when the value changes. Zero means none.
    on_change: CommandId,
    /// True until the user types a digit into this focus session. The first
    /// digit then replaces the value instead of extending it.
    fresh: bool,
    view_state: StateFlags,
}

impl Spinner {
    /// Create a spinner over the inclusive range `min..=max`, starting at `min`.
    ///
    /// A reversed range is swapped rather than rejected, so the control is
    /// always in a usable state.
    pub fn new(bounds: Rect, min: i64, max: i64) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            value: min,
            min,
            max,
            step: 1,
            suffix: String::new(),
            wrap: false,
            on_change: 0,
            fresh: true,
            view_state: State::empty(),
        }
    }

    /// Current value, always within the range.
    pub fn value(&self) -> i64 {
        self.value
    }

    /// Set the value, clamped to the range.
    ///
    /// Returns true when the value actually changed.
    pub fn set_value(&mut self, value: i64) -> bool {
        let clamped = value.clamp(self.min, self.max);
        let changed = clamped != self.value;
        self.value = clamped;
        changed
    }

    /// The inclusive range.
    pub fn range(&self) -> (i64, i64) {
        (self.min, self.max)
    }

    /// Set the range, swapping a reversed one, and re-clamp the value.
    pub fn set_range(&mut self, min: i64, max: i64) {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        self.min = min;
        self.max = max;
        self.value = self.value.clamp(min, max);
    }

    /// Amount one Up or Down press moves the value. Zero is treated as one.
    pub fn set_step(&mut self, step: i64) {
        self.step = if step == 0 { 1 } else { step.abs() };
    }

    /// Text shown after the number, such as `"%"` or `" ms"`.
    pub fn set_suffix(&mut self, suffix: impl Into<String>) {
        self.suffix = suffix.into();
    }

    /// Whether stepping past an end continues from the other end.
    pub fn set_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    /// Command broadcast when the value changes. Zero, the default, sends none.
    pub fn set_on_change(&mut self, command: CommandId) {
        self.on_change = command;
    }

    /// Step up by one step. Returns true when the value changed.
    pub fn step_up(&mut self) -> bool {
        self.step_by(self.step)
    }

    /// Step down by one step. Returns true when the value changed.
    pub fn step_down(&mut self) -> bool {
        self.step_by(-self.step)
    }

    /// Move the value by `delta`, wrapping or clamping at the ends.
    ///
    /// Saturating arithmetic keeps a large step from overflowing near the
    /// extremes of `i64`.
    fn step_by(&mut self, delta: i64) -> bool {
        let raw = self.value.saturating_add(delta);
        let next = if self.wrap {
            if raw > self.max {
                self.min
            } else if raw < self.min {
                self.max
            } else {
                raw
            }
        } else {
            raw.clamp(self.min, self.max)
        };
        let changed = next != self.value;
        self.value = next;
        changed
    }

    /// Append a typed digit to the value, as an editor would.
    ///
    /// The first digit of a focus session replaces the value, so typing `8`
    /// into a field showing 35 gives 8 rather than clamping 358 to the maximum.
    /// Later digits extend it, and the result is clamped, so typing into a
    /// small range simply stops growing rather than rejecting the keypress.
    fn push_digit(&mut self, digit: u32) -> bool {
        let d = digit as i64;
        if self.fresh {
            self.fresh = false;
            return self.set_value(d);
        }
        let base = self.value.saturating_mul(10);
        let candidate = if self.value < 0 {
            base.saturating_sub(d)
        } else {
            base.saturating_add(d)
        };
        self.set_value(candidate)
    }

    /// Drop the last digit, as Backspace would in a text field.
    fn pop_digit(&mut self) -> bool {
        self.set_value(self.value / 10)
    }

    /// Flip the sign, when the range holds the result.
    fn negate(&mut self) -> bool {
        let negated = self.value.saturating_neg();
        if negated >= self.min && negated <= self.max {
            return self.set_value(negated);
        }
        false
    }

    /// Text shown in the field: the number plus any suffix.
    fn display_text(&self) -> String {
        format!("{}{}", self.value, self.suffix)
    }

    /// Screen column of the up stepper, or `None` when the field is too narrow
    /// to draw the steppers.
    fn up_arrow_x(&self) -> Option<i16> {
        let width = self.core.bounds.width_clamped();
        (width as usize > STEPPER_WIDTH).then(|| self.core.bounds.b.x - STEPPER_WIDTH as i16)
    }

    /// Turn a value change into the outgoing event: the change broadcast when
    /// one is configured, otherwise a cleared event.
    fn report(&self, event: &mut Event, changed: bool) {
        if changed && self.on_change != 0 {
            *event = Event::broadcast(self.on_change);
        } else {
            event.clear();
        }
    }
}

impl View for Spinner {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn can_focus(&self) -> bool {
        true
    }

    /// Taking or losing focus starts a new typing session.
    fn set_focus(&mut self, focused: bool) {
        self.set_state_flag(crate::core::state::State::FOCUSED, focused);
        self.fresh = true;
    }

    fn state(&self) -> StateFlags {
        self.view_state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.view_state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped().max(0) as usize;
        if width == 0 {
            return;
        }
        // Borland's input palette gives "normal" and "focused" the same colour,
        // because an InputLine shows focus with its cursor. These controls draw
        // no cursor, so they borrow the selected-text colour to make focus
        // visible.
        let text_attr = if self.is_focused() {
            self.map_color(INPUT_SELECTED)
        } else {
            self.map_color(INPUT_NORMAL)
        };
        let arrow_attr = self.map_color(INPUT_ARROWS);

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', text_attr, width);

        let field_width = width.saturating_sub(STEPPER_WIDTH);
        let text = self.display_text();
        let chars: Vec<char> = text.chars().collect();
        if field_width > 0 {
            // Right-align the number against the steppers, the way a numeric
            // field reads; a too-long value keeps its tail.
            let shown: String = if chars.len() > field_width {
                chars[chars.len() - field_width..].iter().collect()
            } else {
                chars.iter().collect()
            };
            let start = field_width.saturating_sub(shown.chars().count());
            buf.move_str(start, &shown, text_attr);
        }
        if width > STEPPER_WIDTH {
            buf.put_char(width - 2, UP_ARROW, arrow_attr);
            buf.put_char(width - 1, DOWN_ARROW, arrow_attr);
        }

        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        if event.what == EventType::MouseDown && self.core.bounds.contains(event.mouse.pos) {
            if let Some(up_x) = self.up_arrow_x() {
                let changed = if event.mouse.pos.x == up_x {
                    self.step_up()
                } else if event.mouse.pos.x == up_x + 1 {
                    self.step_down()
                } else {
                    // A click on the number itself only takes focus.
                    return;
                };
                self.report(event, changed);
            }
            return;
        }

        if event.what == EventType::MouseWheelUp && self.core.bounds.contains(event.mouse.pos) {
            let changed = self.step_up();
            self.report(event, changed);
            return;
        }
        if event.what == EventType::MouseWheelDown && self.core.bounds.contains(event.mouse.pos) {
            let changed = self.step_down();
            self.report(event, changed);
            return;
        }

        if !self.is_focused() || event.what != EventType::Keyboard {
            return;
        }

        // Anything but a first digit means the user is editing the existing
        // value, so a digit after it extends rather than replaces.
        if event.key_code != KB_BACKSPACE {
            let ch = (event.key_code & 0xFF) as u8 as char;
            if !ch.is_ascii_digit() {
                self.fresh = false;
            }
        }

        let changed = match event.key_code {
            KB_UP => self.step_up(),
            KB_DOWN => self.step_down(),
            KB_PGUP => self.step_by(self.step.saturating_mul(PAGE_FACTOR)),
            KB_PGDN => self.step_by(-self.step.saturating_mul(PAGE_FACTOR)),
            KB_HOME => self.set_value(self.min),
            KB_END => self.set_value(self.max),
            KB_BACKSPACE => self.pop_digit(),
            code => {
                // Printable keys arrive as their ASCII value in the low byte.
                let ch = (code & 0xFF) as u8 as char;
                if let Some(d) = ch.to_digit(10) {
                    self.push_digit(d)
                } else if ch == '-' {
                    self.negate()
                } else {
                    // Not ours: leave it for the dialog's focus and hotkey
                    // handling.
                    return;
                }
            }
        };

        self.report(event, changed);
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_INPUT_LINE))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating spinners with a fluent API.
pub struct SpinnerBuilder {
    bounds: Option<Rect>,
    min: i64,
    max: i64,
    value: Option<i64>,
    step: i64,
    suffix: String,
    wrap: bool,
    on_change: CommandId,
}

impl SpinnerBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            min: 0,
            max: 100,
            value: None,
            step: 1,
            suffix: String::new(),
            wrap: false,
            on_change: 0,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn range(mut self, min: i64, max: i64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    #[must_use]
    pub fn value(mut self, value: i64) -> Self {
        self.value = Some(value);
        self
    }

    #[must_use]
    pub fn step(mut self, step: i64) -> Self {
        self.step = step;
        self
    }

    #[must_use]
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    #[must_use]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    #[must_use]
    pub fn on_change(mut self, command: CommandId) -> Self {
        self.on_change = command;
        self
    }

    pub fn build(self) -> Spinner {
        let bounds = self.bounds.expect("Spinner bounds must be set");
        let mut spin = Spinner::new(bounds, self.min, self.max);
        spin.set_step(self.step);
        spin.set_suffix(self.suffix);
        spin.set_wrap(self.wrap);
        spin.set_on_change(self.on_change);
        if let Some(v) = self.value {
            spin.set_value(v);
        }
        spin
    }

    pub fn build_boxed(self) -> Box<Spinner> {
        Box::new(self.build())
    }
}

impl Default for SpinnerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geometry::Point;

    fn spin(min: i64, max: i64) -> Spinner {
        let mut s = Spinner::new(Rect::new(0, 0, 10, 1), min, max);
        s.set_state(State::FOCUSED);
        s
    }

    fn press(s: &mut Spinner, code: u16) -> Event {
        let mut e = Event::keyboard(code);
        s.handle_event(&mut e);
        e
    }

    fn type_char(s: &mut Spinner, ch: char) {
        let mut e = Event::keyboard(ch as u16);
        s.handle_event(&mut e);
    }

    #[test]
    fn starts_at_the_minimum() {
        assert_eq!(spin(5, 20).value(), 5);
    }

    #[test]
    fn reversed_range_is_swapped() {
        let s = spin(20, 5);
        assert_eq!(s.range(), (5, 20));
        assert_eq!(s.value(), 5);
    }

    #[test]
    fn value_is_clamped_to_the_range() {
        let mut s = spin(0, 10);
        s.set_value(99);
        assert_eq!(s.value(), 10);
        s.set_value(-99);
        assert_eq!(s.value(), 0);
    }

    #[test]
    fn arrows_step_by_the_step_size() {
        let mut s = spin(0, 100);
        s.set_step(5);
        press(&mut s, KB_UP);
        assert_eq!(s.value(), 5);
        press(&mut s, KB_DOWN);
        assert_eq!(s.value(), 0);
    }

    #[test]
    fn stepping_clamps_at_the_ends() {
        let mut s = spin(0, 3);
        for _ in 0..10 {
            press(&mut s, KB_UP);
        }
        assert_eq!(s.value(), 3);
    }

    #[test]
    fn wrap_mode_continues_from_the_other_end() {
        let mut s = spin(0, 3);
        s.set_wrap(true);
        s.set_value(3);
        press(&mut s, KB_UP);
        assert_eq!(s.value(), 0);
        press(&mut s, KB_DOWN);
        assert_eq!(s.value(), 3);
    }

    #[test]
    fn a_huge_step_does_not_overflow() {
        let mut s = spin(i64::MIN, i64::MAX);
        s.set_step(i64::MAX);
        // From i64::MIN it takes three maximum steps to cross the whole range.
        s.step_up();
        s.step_up();
        s.step_up();
        assert_eq!(s.value(), i64::MAX, "saturated rather than wrapped");
        s.step_up();
        assert_eq!(s.value(), i64::MAX, "stays put at the top");
    }

    #[test]
    fn page_keys_step_ten_times_as_far() {
        let mut s = spin(0, 1000);
        s.set_step(3);
        press(&mut s, KB_PGUP);
        assert_eq!(s.value(), 30);
        press(&mut s, KB_PGDN);
        assert_eq!(s.value(), 0);
    }

    #[test]
    fn home_and_end_jump_to_the_range_ends() {
        let mut s = spin(7, 42);
        press(&mut s, KB_END);
        assert_eq!(s.value(), 42);
        press(&mut s, KB_HOME);
        assert_eq!(s.value(), 7);
    }

    #[test]
    fn typing_digits_builds_a_number() {
        let mut s = spin(0, 999);
        type_char(&mut s, '1');
        type_char(&mut s, '2');
        type_char(&mut s, '3');
        assert_eq!(s.value(), 123);
    }

    #[test]
    fn the_first_typed_digit_replaces_the_old_value() {
        let mut s = spin(0, 100);
        s.set_value(35);
        type_char(&mut s, '8');
        assert_eq!(s.value(), 8, "not 358 clamped to 100");
        type_char(&mut s, '0');
        assert_eq!(s.value(), 80, "later digits extend");
    }

    #[test]
    fn refocusing_starts_a_new_typing_session() {
        let mut s = spin(0, 999);
        type_char(&mut s, '1');
        type_char(&mut s, '2');
        assert_eq!(s.value(), 12);
        s.set_focus(false);
        s.set_focus(true);
        type_char(&mut s, '7');
        assert_eq!(s.value(), 7);
    }

    #[test]
    fn stepping_then_typing_extends_rather_than_replaces() {
        let mut s = spin(0, 999);
        press(&mut s, KB_UP);
        assert_eq!(s.value(), 1);
        type_char(&mut s, '5');
        assert_eq!(s.value(), 15, "the arrow ended the fresh state");
    }

    #[test]
    fn typing_past_the_maximum_clamps_instead_of_rejecting() {
        let mut s = spin(0, 50);
        type_char(&mut s, '9');
        type_char(&mut s, '9');
        assert_eq!(s.value(), 50);
    }

    #[test]
    fn backspace_drops_the_last_digit() {
        let mut s = spin(0, 999);
        s.set_value(123);
        press(&mut s, KB_BACKSPACE);
        assert_eq!(s.value(), 12);
    }

    #[test]
    fn minus_negates_only_when_the_range_allows_it() {
        let mut s = spin(-100, 100);
        s.set_value(25);
        type_char(&mut s, '-');
        assert_eq!(s.value(), -25);

        let mut positive_only = spin(0, 100);
        positive_only.set_value(25);
        type_char(&mut positive_only, '-');
        assert_eq!(positive_only.value(), 25, "range has no negatives");
    }

    #[test]
    fn digits_extend_a_negative_value_downward() {
        let mut s = spin(-999, 999);
        s.set_value(-1);
        type_char(&mut s, '-'); // ends the fresh state without changing the sign twice
        s.set_value(-1);
        type_char(&mut s, '2');
        assert_eq!(s.value(), -12);
    }

    #[test]
    fn unhandled_keys_are_left_alone() {
        let mut s = spin(0, 10);
        let e = press(&mut s, crate::core::event::KB_ENTER);
        assert_eq!(
            e.what,
            EventType::Keyboard,
            "Enter must still reach the default button"
        );
    }

    #[test]
    fn keys_do_nothing_when_not_focused() {
        let mut s = Spinner::new(Rect::new(0, 0, 10, 1), 0, 10);
        press(&mut s, KB_UP);
        assert_eq!(s.value(), 0);
    }

    #[test]
    fn clicking_the_steppers_changes_the_value() {
        let mut s = spin(0, 10);
        let up_x = s.up_arrow_x().unwrap();

        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.pos = Point::new(up_x, 0);
        s.handle_event(&mut e);
        assert_eq!(s.value(), 1);

        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.pos = Point::new(up_x + 1, 0);
        s.handle_event(&mut e);
        assert_eq!(s.value(), 0);
    }

    #[test]
    fn clicking_the_number_only_takes_focus() {
        let mut s = spin(0, 10);
        let mut e = Event::nothing();
        e.what = EventType::MouseDown;
        e.mouse.pos = Point::new(0, 0);
        s.handle_event(&mut e);
        assert_eq!(s.value(), 0);
        assert_eq!(e.what, EventType::MouseDown, "left for focus handling");
    }

    #[test]
    fn on_change_broadcasts_only_on_a_real_change() {
        let mut s = spin(0, 1);
        s.set_on_change(555);
        let e = press(&mut s, KB_UP);
        assert_eq!(e.command, 555);
        // Already at the maximum, so nothing changes.
        let e = press(&mut s, KB_UP);
        assert_eq!(e.what, EventType::Nothing);
    }

    #[test]
    fn display_includes_the_suffix() {
        let mut s = spin(0, 100);
        s.set_suffix("%");
        s.set_value(60);
        assert_eq!(s.display_text(), "60%");
    }

    #[test]
    fn narrow_field_draws_no_steppers() {
        let s = Spinner::new(Rect::new(0, 0, 2, 1), 0, 10);
        assert_eq!(s.up_arrow_x(), None);
    }

    #[test]
    fn builder_configures_the_spinner() {
        let s = SpinnerBuilder::new()
            .bounds(Rect::new(0, 0, 12, 1))
            .range(10, 20)
            .value(15)
            .step(2)
            .suffix(" ms")
            .build();
        assert_eq!(s.value(), 15);
        assert_eq!(s.range(), (10, 20));
        assert_eq!(s.display_text(), "15 ms");
    }
}
