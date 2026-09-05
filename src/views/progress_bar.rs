// (C) 2025 - Enzo Lombardi

//! ProgressBar view - determinate and indeterminate progress indicator.
//!
//! Not part of the Borland Turbo Vision widget set; added because any
//! application that copies, compiles or downloads needs one.
//!
//! Two modes:
//! - [`ProgressMode::Determinate`] fills the track in proportion to
//!   `value / max`, optionally overlaying a centred percentage or caption.
//! - [`ProgressMode::Marquee`] sweeps a fixed-width block back and forth for
//!   work of unknown duration. Call [`ProgressBar::tick`] from an idle handler
//!   to animate it.
//!
//! # Example
//!
//! ```rust
//! use turbo_vision::views::progress_bar::ProgressBar;
//! use turbo_vision::core::geometry::Rect;
//!
//! let mut bar = ProgressBar::new(Rect::new(2, 3, 40, 4), 100);
//! bar.set_value(45);
//! assert_eq!(bar.percent(), 45);
//! ```

use super::view::{View, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::Attr;
use crate::terminal::Terminal;
use std::time::{Duration, Instant};

/// Fill mode of a [`ProgressBar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressMode {
    /// Fill proportional to `value / max`.
    Determinate,
    /// A block sweeping back and forth; progress is unknown.
    Marquee,
}

/// Character set used to draw the track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStyle {
    /// Solid block filled, light shade empty. Whole cells only.
    Blocks,
    /// Like [`ProgressStyle::Blocks`] but the leading cell uses partial block
    /// glyphs, giving eighth-of-a-cell resolution.
    Smooth,
    /// `#` filled, `-` empty. For terminals without box-drawing glyphs.
    Ascii,
}

impl ProgressStyle {
    fn filled_char(self) -> char {
        match self {
            ProgressStyle::Blocks | ProgressStyle::Smooth => '\u{2588}', // full block
            ProgressStyle::Ascii => '#',
        }
    }

    fn empty_char(self) -> char {
        match self {
            ProgressStyle::Blocks | ProgressStyle::Smooth => '\u{2591}', // light shade
            ProgressStyle::Ascii => '-',
        }
    }

    /// Partial block for `eighths` in 1..=7, or `None` when this style does not
    /// render partial cells.
    fn partial_char(self, eighths: u32) -> Option<char> {
        if self != ProgressStyle::Smooth || eighths == 0 || eighths >= 8 {
            return None;
        }
        // U+258F LEFT ONE EIGHTH BLOCK .. U+2589 LEFT SEVEN EIGHTHS BLOCK
        char::from_u32(0x2590 - eighths)
    }
}

/// What to overlay on the middle of the track.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Caption {
    None,
    Percent,
    Text(String),
}

/// A horizontal progress indicator.
pub struct ProgressBar {
    bounds: Rect,
    value: u64,
    max: u64,
    mode: ProgressMode,
    style: ProgressStyle,
    caption: Caption,
    /// Leading edge of the marquee block, in cells from the left.
    marquee_pos: u16,
    /// Current marquee direction: `true` moves right.
    marquee_forward: bool,
    /// Minimum wall-clock gap between automatic marquee steps.
    tick_interval: Duration,
    /// When the marquee last stepped, for the `IdleView` animation.
    last_tick: Instant,
    palette_chain: Option<crate::core::palette_chain::PaletteChainNode>,
}

/// Marquee block width as a fraction of the track width.
const MARQUEE_DIVISOR: u16 = 4;

/// Default wall-clock gap between marquee steps when animated from idle.
const DEFAULT_TICK_INTERVAL: Duration = Duration::from_millis(80);

impl ProgressBar {
    /// Create a determinate bar with the given upper bound.
    ///
    /// A `max` of zero is clamped to one so the bar never divides by zero; it
    /// then reads as 0% until a real maximum is set.
    pub fn new(bounds: Rect, max: u64) -> Self {
        Self {
            bounds,
            value: 0,
            max: max.max(1),
            mode: ProgressMode::Determinate,
            style: ProgressStyle::Smooth,
            caption: Caption::Percent,
            marquee_pos: 0,
            marquee_forward: true,
            tick_interval: DEFAULT_TICK_INTERVAL,
            last_tick: Instant::now(),
            palette_chain: None,
        }
    }

    /// Current value, always within `0..=max`.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Set the current value. Values above `max` are clamped.
    pub fn set_value(&mut self, value: u64) {
        self.value = value.min(self.max);
    }

    /// Add to the current value, clamping at `max` and saturating on overflow.
    pub fn advance(&mut self, delta: u64) {
        self.set_value(self.value.saturating_add(delta));
    }

    /// Upper bound of the bar.
    pub fn max(&self) -> u64 {
        self.max
    }

    /// Set the upper bound. Zero is clamped to one; the current value is
    /// re-clamped to the new bound.
    pub fn set_max(&mut self, max: u64) {
        self.max = max.max(1);
        self.value = self.value.min(self.max);
    }

    /// Reset the value to zero and rewind the marquee.
    pub fn reset(&mut self) {
        self.value = 0;
        self.marquee_pos = 0;
        self.marquee_forward = true;
    }

    /// Completion as a fraction in `0.0..=1.0`.
    pub fn fraction(&self) -> f64 {
        self.value as f64 / self.max as f64
    }

    /// Completion as a whole percentage in `0..=100`, truncated.
    pub fn percent(&self) -> u32 {
        // Compute in integers so 99.9% never rounds up to a misleading 100%.
        ((self.value as u128 * 100) / self.max as u128) as u32
    }

    /// Current mode.
    pub fn mode(&self) -> ProgressMode {
        self.mode
    }

    /// Switch between determinate and marquee display.
    pub fn set_mode(&mut self, mode: ProgressMode) {
        self.mode = mode;
    }

    /// Set the glyphs used to draw the track.
    pub fn set_style(&mut self, style: ProgressStyle) {
        self.style = style;
    }

    /// Overlay the truncated percentage on the track. This is the default.
    pub fn show_percent(&mut self) {
        self.caption = Caption::Percent;
    }

    /// Turn the percentage overlay on or off.
    ///
    /// Enabling replaces any fixed caption set with [`ProgressBar::set_caption`].
    /// Disabling clears the overlay entirely, but only when the percentage is
    /// what is currently shown, so this never silently drops a fixed caption.
    pub fn set_show_percent(&mut self, show: bool) {
        match (show, &self.caption) {
            (true, _) => self.caption = Caption::Percent,
            (false, Caption::Percent) => self.caption = Caption::None,
            (false, _) => {}
        }
    }

    /// Whether the percentage overlay is currently shown.
    pub fn is_percent_shown(&self) -> bool {
        self.caption == Caption::Percent
    }

    /// Overlay fixed text on the track instead of a percentage.
    pub fn set_caption(&mut self, text: impl Into<String>) {
        self.caption = Caption::Text(text.into());
    }

    /// Draw the track with nothing overlaid.
    pub fn hide_caption(&mut self) {
        self.caption = Caption::None;
    }

    /// Advance the marquee by one cell, reversing at either end.
    ///
    /// No-op in [`ProgressMode::Determinate`]. Call this from an idle handler
    /// at whatever rate you want the animation to run.
    pub fn tick(&mut self) {
        if self.mode != ProgressMode::Marquee {
            return;
        }
        let width = self.bounds.width_clamped().max(0) as u16;
        let block = Self::marquee_width(width);
        // The leading edge travels over the cells the block cannot occupy.
        let span = width.saturating_sub(block);
        if span == 0 {
            self.marquee_pos = 0;
            return;
        }
        if self.marquee_forward {
            if self.marquee_pos >= span {
                self.marquee_forward = false;
                self.marquee_pos = span.saturating_sub(1);
            } else {
                self.marquee_pos += 1;
            }
        } else if self.marquee_pos == 0 {
            self.marquee_forward = true;
            self.marquee_pos = 1.min(span);
        } else {
            self.marquee_pos -= 1;
        }
    }

    /// Set how often the idle animation steps the marquee. Ignored when the
    /// bar is stepped manually with [`ProgressBar::tick`].
    pub fn set_tick_interval(&mut self, interval: Duration) {
        self.tick_interval = interval;
    }

    fn marquee_width(track_width: u16) -> u16 {
        (track_width / MARQUEE_DIVISOR).max(1).min(track_width)
    }

    /// Text to overlay, or `None`.
    fn caption_text(&self) -> Option<String> {
        match &self.caption {
            Caption::None => None,
            Caption::Text(t) => Some(t.clone()),
            Caption::Percent => match self.mode {
                // A percentage is meaningless when progress is unknown.
                ProgressMode::Marquee => None,
                ProgressMode::Determinate => Some(format!("{}%", self.percent())),
            },
        }
    }

    /// Number of whole filled cells plus the eighths of the next cell, for a
    /// determinate bar of `width` cells.
    fn fill_cells(&self, width: u16) -> (u16, u32) {
        if width == 0 {
            return (0, 0);
        }
        let eighths_total = width as u128 * 8;
        let filled_eighths = (self.value as u128 * eighths_total) / self.max as u128;
        let whole = (filled_eighths / 8) as u16;
        let rem = (filled_eighths % 8) as u32;
        (whole.min(width), rem)
    }

    /// Render the track into a buffer. Split out from `draw` so tests can
    /// inspect the result without a terminal.
    fn render(&self, width: usize, filled_attr: Attr, empty_attr: Attr) -> DrawBuffer {
        let mut buf = DrawBuffer::new(width);
        if width == 0 {
            return buf;
        }
        let w = width as u16;

        buf.move_char(0, self.style.empty_char(), empty_attr, width);

        // `filled` is the count of cells drawn in the filled attribute; the
        // caption uses it to pick a colour per cell.
        let filled = match self.mode {
            ProgressMode::Determinate => {
                let (whole, rem) = self.fill_cells(w);
                if whole > 0 {
                    buf.move_char(0, self.style.filled_char(), filled_attr, whole as usize);
                }
                if let Some(ch) = self.style.partial_char(rem) {
                    if whole < w {
                        buf.put_char(whole as usize, ch, filled_attr);
                    }
                }
                whole
            }
            ProgressMode::Marquee => {
                let block = Self::marquee_width(w);
                let start = self.marquee_pos.min(w.saturating_sub(block));
                buf.move_char(
                    start as usize,
                    self.style.filled_char(),
                    filled_attr,
                    block as usize,
                );
                // Marquee cells are not a prefix, so the caption cannot use a
                // simple "before this index" test; report zero and let it draw
                // in the empty attribute throughout.
                0
            }
        };

        if let Some(text) = self.caption_text() {
            let chars: Vec<char> = text.chars().collect();
            if chars.len() <= width {
                let start = (width - chars.len()) / 2;
                for (i, ch) in chars.into_iter().enumerate() {
                    let pos = start + i;
                    let attr = if (pos as u16) < filled {
                        filled_attr
                    } else {
                        empty_attr
                    };
                    buf.put_char(pos, ch, attr);
                }
            }
        }

        buf
    }
}

impl View for ProgressBar {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width_clamped().max(0) as usize;
        // CP_PROGRESS_BAR: 1 = filled, 2 = empty track.
        let filled = self.map_color(1);
        let empty = self.map_color(2);
        let buf = self.render(width, filled, empty);
        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // A progress bar is output only.
    }

    fn can_focus(&self) -> bool {
        false
    }

    fn set_palette_chain(&mut self, node: Option<crate::core::palette_chain::PaletteChainNode>) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_PROGRESS_BAR))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl crate::views::view::IdleView for ProgressBar {
    /// Step the marquee, rate-limited to the tick interval. Determinate bars
    /// are left alone, so adding one as an overlay widget costs nothing.
    fn idle(&mut self) {
        if self.mode != ProgressMode::Marquee {
            return;
        }
        if self.last_tick.elapsed() >= self.tick_interval {
            self.last_tick = Instant::now();
            self.tick();
        }
    }
}

/// Builder for creating progress bars with a fluent API.
pub struct ProgressBarBuilder {
    bounds: Option<Rect>,
    max: u64,
    mode: ProgressMode,
    style: ProgressStyle,
    caption: Caption,
}

impl ProgressBarBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            max: 100,
            mode: ProgressMode::Determinate,
            style: ProgressStyle::Smooth,
            caption: Caption::Percent,
        }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn max(mut self, max: u64) -> Self {
        self.max = max;
        self
    }

    #[must_use]
    pub fn marquee(mut self) -> Self {
        self.mode = ProgressMode::Marquee;
        self
    }

    #[must_use]
    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn caption(mut self, text: impl Into<String>) -> Self {
        self.caption = Caption::Text(text.into());
        self
    }

    /// Show or hide the percentage overlay. Shown by default.
    #[must_use]
    pub fn show_percent(mut self, show: bool) -> Self {
        self.caption = if show {
            Caption::Percent
        } else {
            Caption::None
        };
        self
    }

    #[must_use]
    pub fn no_caption(mut self) -> Self {
        self.caption = Caption::None;
        self
    }

    pub fn build(self) -> ProgressBar {
        let bounds = self.bounds.expect("ProgressBar bounds must be set");
        let mut bar = ProgressBar::new(bounds, self.max);
        bar.mode = self.mode;
        bar.style = self.style;
        bar.caption = self.caption;
        bar
    }

    pub fn build_boxed(self) -> Box<ProgressBar> {
        Box::new(self.build())
    }
}

impl Default for ProgressBarBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(width: i16, max: u64) -> ProgressBar {
        ProgressBar::new(Rect::new(0, 0, width, 1), max)
    }

    /// Collect the rendered glyphs, ignoring attributes.
    fn glyphs(b: &ProgressBar, width: usize) -> String {
        let attr = Attr::from_u8(0x07);
        let buf = b.render(width, attr, attr);
        buf.data.iter().map(|c| c.ch).collect::<String>()
    }

    #[test]
    fn value_and_max_are_clamped() {
        let mut b = bar(10, 50);
        b.set_value(999);
        assert_eq!(b.value(), 50);
        b.set_max(10);
        assert_eq!(b.value(), 10, "value re-clamps when max shrinks");
    }

    #[test]
    fn zero_max_does_not_divide_by_zero() {
        let b = bar(10, 0);
        assert_eq!(b.max(), 1);
        assert_eq!(b.percent(), 0);
    }

    #[test]
    fn percent_truncates_instead_of_rounding_up() {
        let mut b = bar(10, 1000);
        b.set_value(999);
        assert_eq!(b.percent(), 99, "99.9% must not read as complete");
    }

    #[test]
    fn advance_saturates() {
        let mut b = bar(10, 100);
        b.advance(u64::MAX);
        assert_eq!(b.value(), 100);
    }

    #[test]
    fn determinate_fill_is_proportional() {
        let mut b = bar(10, 100);
        b.set_style(ProgressStyle::Ascii);
        b.hide_caption();
        b.set_value(30);
        assert_eq!(glyphs(&b, 10), "###-------");
        b.set_value(100);
        assert_eq!(glyphs(&b, 10), "##########");
    }

    #[test]
    fn smooth_style_renders_a_partial_cell() {
        let mut b = bar(4, 8);
        b.hide_caption();
        b.set_value(3); // 1.5 cells of 4
        let g = glyphs(&b, 4);
        assert_eq!(g.chars().next(), Some('\u{2588}'));
        assert_eq!(
            g.chars().nth(1),
            Some('\u{258C}'),
            "half-filled second cell"
        );
    }

    #[test]
    fn caption_is_centred_over_the_track() {
        let mut b = bar(10, 100);
        b.set_style(ProgressStyle::Ascii);
        b.set_value(0);
        assert_eq!(glyphs(&b, 10), "----0%----");
    }

    #[test]
    fn marquee_has_no_percentage() {
        let mut b = bar(12, 100);
        b.set_mode(ProgressMode::Marquee);
        b.set_style(ProgressStyle::Ascii);
        b.set_value(50);
        assert_eq!(glyphs(&b, 12), "###---------", "block at the left edge");
    }

    #[test]
    fn marquee_sweeps_and_reverses() {
        let mut b = bar(8, 100);
        b.set_mode(ProgressMode::Marquee);
        let span = 8 - ProgressBar::marquee_width(8); // leading-edge travel
        for _ in 0..span {
            b.tick();
        }
        assert_eq!(b.marquee_pos, span, "reached the right end");
        b.tick();
        assert!(!b.marquee_forward, "turned around");
        assert_eq!(b.marquee_pos, span - 1);
        for _ in 0..span {
            b.tick();
        }
        assert!(b.marquee_forward, "turned around again at the left");
    }

    #[test]
    fn tick_is_inert_in_determinate_mode() {
        let mut b = bar(20, 100);
        b.tick();
        assert_eq!(b.marquee_pos, 0);
    }

    #[test]
    fn zero_width_renders_nothing() {
        let b = bar(0, 100);
        assert_eq!(glyphs(&b, 0), "");
    }

    #[test]
    fn oversized_caption_is_dropped_rather_than_truncated() {
        let mut b = bar(4, 100);
        b.set_style(ProgressStyle::Ascii);
        b.set_caption("far too long");
        assert_eq!(glyphs(&b, 4), "----");
    }

    #[test]
    fn percent_overlay_toggles() {
        let mut b = bar(10, 100);
        b.set_style(ProgressStyle::Ascii);
        assert!(b.is_percent_shown());
        b.set_show_percent(false);
        assert!(!b.is_percent_shown());
        assert_eq!(glyphs(&b, 10), "----------");
        b.set_show_percent(true);
        assert_eq!(glyphs(&b, 10), "----0%----");
    }

    #[test]
    fn disabling_percent_leaves_a_fixed_caption_alone() {
        let mut b = bar(10, 100);
        b.set_style(ProgressStyle::Ascii);
        b.set_caption("Busy");
        b.set_show_percent(false);
        assert_eq!(b.caption_text().as_deref(), Some("Busy"));
    }

    #[test]
    fn builder_can_suppress_the_percentage() {
        let b = ProgressBarBuilder::new()
            .bounds(Rect::new(0, 0, 10, 1))
            .show_percent(false)
            .build();
        assert!(!b.is_percent_shown());
        assert_eq!(b.caption_text(), None);
    }

    #[test]
    fn builder_configures_the_bar() {
        let b = ProgressBarBuilder::new()
            .bounds(Rect::new(0, 0, 10, 1))
            .max(7)
            .style(ProgressStyle::Ascii)
            .caption("Loading")
            .build();
        assert_eq!(b.max(), 7);
        assert_eq!(b.caption_text().as_deref(), Some("Loading"));
    }
}
