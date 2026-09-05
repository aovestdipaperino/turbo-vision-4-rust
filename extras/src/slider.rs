// (C) 2026 - Enzo Lombardi

//! Slider - horizontal value selector (TV Tool Box style).

use turbo_vision::core::command::CommandId;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{
    Event, EventType, KB_END, KB_HOME, KB_LEFT, KB_RIGHT, MB_LEFT_BUTTON,
};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::core::state::State;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{View, ViewCore};

/// Horizontal slider: `min ────────◆────── max`.
///
/// Left/Right adjust by one step, Home/End jump to the ends, and clicking
/// or dragging on the track sets the value directly. When `on_change` is
/// set, every change converts the handled event into a Broadcast carrying
/// the new value in `event.info`.
///
/// # Examples
///
/// ```
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision_extras::Slider;
///
/// let mut slider = Slider::new(Rect::new(2, 2, 32, 3), 0, 10);
/// slider.set_value(5);
/// assert_eq!(slider.value(), 5);
/// ```
#[derive(Debug)]
pub struct Slider {
    core: ViewCore,
    min: i32,
    max: i32,
    value: i32,
    step: i32,
    on_change: Option<CommandId>,
    palette_chain: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
}

impl Slider {
    /// Create a slider over `min..=max` (max is clamped to at least min+1).
    pub fn new(bounds: Rect, min: i32, max: i32) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: State::empty(),
                ..ViewCore::default()
            },
            min,
            max: max.max(min + 1),
            value: min,
            step: 1,
            on_change: None,
            palette_chain: None,
        }
    }

    /// Broadcast this command (value in `event.info`) whenever the value
    /// changes through user interaction.
    pub fn set_on_change(&mut self, command: CommandId) {
        self.on_change = Some(command);
    }

    /// Set the arrow-key step (default 1, clamped to at least 1).
    pub fn set_step(&mut self, step: i32) {
        self.step = step.max(1);
    }

    /// Set the current value, clamped to the range.
    pub fn set_value(&mut self, value: i32) {
        self.value = value.clamp(self.min, self.max);
    }

    /// Current value.
    pub fn value(&self) -> i32 {
        self.value
    }

    fn value_to_col(&self, width: usize) -> usize {
        let track = width.saturating_sub(1).max(1);
        (track as i64 * (self.value - self.min) as i64 / (self.max - self.min) as i64) as usize
    }

    fn col_to_value(&self, col: i16, width: usize) -> i32 {
        let track = width.saturating_sub(1).max(1) as i64;
        let col = (col as i64).clamp(0, track);
        self.min + ((self.max - self.min) as i64 * col / track) as i32
    }

    fn changed(&self, event: &mut Event) {
        if let Some(cmd) = self.on_change {
            *event = Event::broadcast_with_info(cmd, self.value.clamp(0, u16::MAX as i32) as u16);
        } else {
            event.clear();
        }
    }
}

impl View for Slider {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        if width == 0 {
            return;
        }

        let track_attr = if self.is_focused() {
            Attr::new(TvColor::White, TvColor::Blue)
        } else {
            Attr::new(TvColor::LightGray, TvColor::Blue)
        };
        let thumb_attr = Attr::new(TvColor::Yellow, TvColor::Blue);

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, '─', track_attr, width);
        buf.put_char(self.value_to_col(width), '◆', thumb_attr);
        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        let width = self.core.bounds.width_clamped() as usize;
        match event.what {
            EventType::Keyboard if self.is_focused() => {
                let new_value = match event.key_code {
                    KB_LEFT => self.value - self.step,
                    KB_RIGHT => self.value + self.step,
                    KB_HOME => self.min,
                    KB_END => self.max,
                    _ => return,
                };
                self.set_value(new_value);
                self.changed(event);
            }
            EventType::MouseDown | EventType::MouseMove => {
                let pos = event.mouse.pos;
                let pressed = event.mouse.buttons & MB_LEFT_BUTTON != 0;
                if pressed && self.core.bounds.contains(pos) {
                    self.set_value(self.col_to_value(pos.x - self.core.bounds.a.x, width));
                    self.changed(event);
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        true
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

    #[test]
    fn keyboard_adjusts_and_clamps() {
        let mut s = Slider::new(Rect::new(0, 0, 30, 1), 0, 10);
        s.set_focus(true);

        let mut ev = Event::keyboard(KB_RIGHT);
        s.handle_event(&mut ev);
        assert_eq!(s.value(), 1);
        assert_eq!(ev.what, EventType::Nothing);

        let mut ev = Event::keyboard(KB_END);
        s.handle_event(&mut ev);
        assert_eq!(s.value(), 10);
        let mut ev = Event::keyboard(KB_RIGHT);
        s.handle_event(&mut ev);
        assert_eq!(s.value(), 10); // clamped
    }

    #[test]
    fn click_sets_value_and_broadcasts() {
        let mut s = Slider::new(Rect::new(0, 0, 11, 1), 0, 10);
        s.set_on_change(500);

        // Click at the far right of an 11-cell track: value = max
        let mut ev = Event::mouse(
            EventType::MouseDown,
            Point::new(10, 0),
            MB_LEFT_BUTTON,
            false,
        );
        s.handle_event(&mut ev);
        assert_eq!(s.value(), 10);
        assert_eq!(ev.what, EventType::Broadcast);
        assert_eq!(ev.command, 500);
        assert_eq!(ev.info, 10);
    }
}
