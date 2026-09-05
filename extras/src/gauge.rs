// (C) 2026 - Enzo Lombardi

//! Gauge - horizontal progress bar (TV Tool Box style).

use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::Event;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{View, ViewCore};

/// Horizontal progress bar with optional percentage caption.
///
/// # Examples
///
/// ```
/// use turbo_vision::core::geometry::Rect;
/// use turbo_vision_extras::Gauge;
///
/// let mut gauge = Gauge::new(Rect::new(2, 2, 42, 3), 100);
/// gauge.set_value(40);
/// assert_eq!(gauge.value(), 40);
/// ```
#[derive(Debug)]
pub struct Gauge {
    core: ViewCore,
    value: i32,
    max: i32,
    show_percent: bool,
    palette_chain: Option<turbo_vision::core::palette_chain::PaletteChainNode>,
}

impl Gauge {
    /// Create a gauge running from 0 to `max` (clamped to at least 1).
    pub fn new(bounds: Rect, max: i32) -> Self {
        Self {
            core: ViewCore {
                bounds,
                state: 0,
                ..ViewCore::default()
            },
            value: 0,
            max: max.max(1),
            show_percent: true,
            palette_chain: None,
        }
    }

    /// Show or hide the centered percentage caption (default: shown).
    pub fn set_show_percent(&mut self, show: bool) {
        self.show_percent = show;
    }

    /// Set the current value, clamped to `0..=max`.
    pub fn set_value(&mut self, value: i32) {
        self.value = value.clamp(0, self.max);
    }

    /// Current value.
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Maximum value.
    pub fn max(&self) -> i32 {
        self.max
    }
}

impl View for Gauge {
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

        let filled = (width as i64 * self.value as i64 / self.max as i64) as usize;
        let bar_attr = Attr::new(TvColor::Cyan, TvColor::Blue);

        let mut buf = DrawBuffer::new(width);
        for x in 0..width {
            let ch = if x < filled { '█' } else { '░' };
            buf.put_char(x, ch, bar_attr);
        }

        if self.show_percent {
            let pct = format!(" {}% ", self.value as i64 * 100 / self.max as i64);
            let start = width.saturating_sub(pct.chars().count()) / 2;
            let text_attr = Attr::new(TvColor::White, TvColor::Blue);
            buf.move_str(start, &pct, text_attr);
        }

        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {}

    fn can_focus(&self) -> bool {
        false
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_clamps_to_range() {
        let mut g = Gauge::new(Rect::new(0, 0, 20, 1), 100);
        g.set_value(150);
        assert_eq!(g.value(), 100);
        g.set_value(-5);
        assert_eq!(g.value(), 0);
    }

    #[test]
    fn zero_max_is_clamped() {
        let g = Gauge::new(Rect::new(0, 0, 20, 1), 0);
        assert_eq!(g.max(), 1);
    }
}
