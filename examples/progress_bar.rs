// (C) 2025 - Enzo Lombardi
// ProgressBar example - determinate, marquee, and the percentage toggle.
//
// Four bars are added as overlay widgets so they animate from the idle loop:
//   1. Smooth determinate bar with the percentage shown.
//   2. The same bar with the percentage suppressed.
//   3. ASCII determinate bar with a fixed caption.
//   4. Marquee bar, which steps itself via IdleView.
//
// Press Alt-X to exit.

use std::time::Instant;
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{Event, KB_ALT_X};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Palette;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::progress_bar::{ProgressBar, ProgressBarBuilder, ProgressStyle};
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::{IdleView, View, ViewCore};

/// A caption plus a bar, driving the bar's value from wall-clock time.
struct DemoRow {
    label: StaticText,
    bar: ProgressBar,
    /// Determinate bars advance one unit per `step_ms`; marquee bars ignore it.
    step_ms: u128,
    last_step: Instant,
    animate_value: bool,
}

impl DemoRow {
    fn new(y: i16, label: &str, bar: ProgressBar, animate_value: bool) -> Self {
        Self {
            label: StaticText::new(Rect::new(4, y, 28, y + 1), label),
            bar,
            step_ms: 60,
            last_step: Instant::now(),
            animate_value,
        }
    }
}

impl View for DemoRow {
    fn core(&self) -> &ViewCore {
        self.bar.core()
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        self.bar.core_mut()
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.label.draw(terminal);
        self.bar.draw(terminal);
    }

    fn handle_event(&mut self, _event: &mut Event) {}

    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn get_palette(&self) -> Option<Palette> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl IdleView for DemoRow {
    fn idle(&mut self) {
        if self.animate_value && self.last_step.elapsed().as_millis() >= self.step_ms {
            self.last_step = Instant::now();
            if self.bar.value() >= self.bar.max() {
                self.bar.reset();
            } else {
                self.bar.advance(1);
            }
        }
        // Steps the marquee; a no-op for determinate bars.
        self.bar.idle();
    }
}

fn bar_at(y: i16) -> Rect {
    Rect::new(30, y, 70, y + 1)
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let with_percent = ProgressBarBuilder::new().bounds(bar_at(4)).max(100).build();

    let without_percent = ProgressBarBuilder::new()
        .bounds(bar_at(6))
        .max(100)
        .show_percent(false)
        .build();

    let captioned = ProgressBarBuilder::new()
        .bounds(bar_at(8))
        .max(100)
        .style(ProgressStyle::Ascii)
        .caption("Copying files")
        .build();

    let marquee = ProgressBarBuilder::new()
        .bounds(bar_at(10))
        .marquee()
        .build();

    app.add_overlay_widget(Box::new(DemoRow::new(
        4,
        "Smooth, percentage on:",
        with_percent,
        true,
    )));
    app.add_overlay_widget(Box::new(DemoRow::new(
        6,
        "Smooth, percentage off:",
        without_percent,
        true,
    )));
    app.add_overlay_widget(Box::new(DemoRow::new(
        8,
        "ASCII, fixed caption:",
        captioned,
        true,
    )));
    app.add_overlay_widget(Box::new(DemoRow::new(10, "Marquee:", marquee, false)));

    let (w, h) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, h as i16 - 1, w as i16, h as i16),
        vec![StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT)],
    ));

    app.run();
    Ok(())
}
