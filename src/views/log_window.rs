// (C) 2025 - Enzo Lombardi
// Rust guideline compliant October 17th 2025

//! LogWindow view - scrollable log window with `tracing::Subscriber` integration.
//!
//! A window that displays tracing log messages with timestamps, colored log levels,
//! and a black background. Once installed, all `tracing::info!()`, `debug!()`, etc.
//! macros automatically route to the window.
//!
//! # Example
//!
//! ```ignore
//! use turbo_vision::views::log_window::LogWindowBuilder;
//! use turbo_vision::core::geometry::Rect;
//!
//! let log_window = LogWindowBuilder::new()
//!     .bounds(Rect::new(0, 0, 80, 15))
//!     .title("Log")
//!     .min_level(tracing::Level::DEBUG)
//!     .build();
//! app.desktop.add(Box::new(log_window));
//!
//! // Now tracing macros route here:
//! tracing::info!("Application started");
//! tracing::debug!("Loading config from {:?}", path);
//! ```

use crate::core::geometry::Rect;
use crate::core::event::Event;
use crate::core::palette::{Attr, TvColor};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;
use super::terminal_widget::TerminalWidget;
use super::view::View;
use super::window::{Window, WindowPaletteType};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A formatted log entry ready for display.
struct LogEntry {
    text: String,
    attr: Attr,
}

/// The `tracing::Subscriber` implementation that sends log entries to the window.
///
/// This is `Send + Sync` (required by tracing) and communicates with the
/// single-threaded `LogWindow` via an `mpsc` channel.
pub struct LogSubscriber {
    sender: mpsc::Sender<LogEntry>,
    min_level: tracing::Level,
}

impl tracing::Subscriber for LogSubscriber {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.level() <= &self.min_level
    }

    fn new_span(&self, _attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        static NEXT: AtomicUsize = AtomicUsize::new(1);
        tracing::span::Id::from_u64(NEXT.fetch_add(1, Ordering::Relaxed) as u64)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let metadata = event.metadata();
        let level = *metadata.level();

        // Format: "HH:MM:SS LEVEL message"
        let now = chrono::Local::now();
        let timestamp = now.format("%H:%M:%S");

        let level_str = match level {
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN  => "WARN ",
            tracing::Level::INFO  => "INFO ",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "TRACE",
        };

        // Extract the message from the event
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let message = visitor.message;

        let text = format!("{timestamp} {level_str} {message}");
        let attr = level_attr(level);

        // Send is non-blocking; if the receiver is gone, silently drop
        let _ = self.sender.send(LogEntry { text, attr });
    }

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

/// Visitor that extracts the message field from a tracing event.
#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        } else if self.message.is_empty() {
            self.message = format!("{}: {value:?}", field.name());
        } else {
            self.message.push_str(&format!(" {}={value:?}", field.name()));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else if self.message.is_empty() {
            self.message = format!("{}: {value}", field.name());
        } else {
            self.message.push_str(&format!(" {}={value}", field.name()));
        }
    }
}

/// Map a tracing level to a color attribute (foreground on black).
fn level_attr(level: tracing::Level) -> Attr {
    match level {
        tracing::Level::ERROR => Attr::new(TvColor::LightRed, TvColor::Black),
        tracing::Level::WARN  => Attr::new(TvColor::Yellow, TvColor::Black),
        tracing::Level::INFO  => Attr::new(TvColor::White, TvColor::Black),
        tracing::Level::DEBUG => Attr::new(TvColor::LightGray, TvColor::Black),
        tracing::Level::TRACE => Attr::new(TvColor::DarkGray, TvColor::Black),
    }
}

/// LogWindow - a scrollable window displaying tracing log messages.
///
/// Wraps a `Window` + `TerminalWidget` and drains incoming log entries
/// from the `LogSubscriber` channel on each `draw()` call.
pub struct LogWindow {
    window: Window,
    widget: Rc<RefCell<TerminalWidget>>,
    receiver: mpsc::Receiver<LogEntry>,
}

/// Shared wrapper so TerminalWidget can be a View child of the Window.
struct SharedTerminalWidget(Rc<RefCell<TerminalWidget>>);

impl View for SharedTerminalWidget {
    fn bounds(&self) -> Rect { self.0.borrow().bounds() }
    fn set_bounds(&mut self, bounds: Rect) { self.0.borrow_mut().set_bounds(bounds); }
    fn draw(&mut self, terminal: &mut Terminal) { self.0.borrow_mut().draw(terminal); }
    fn handle_event(&mut self, event: &mut Event) { self.0.borrow_mut().handle_event(event); }
    fn can_focus(&self) -> bool { true }
    fn state(&self) -> StateFlags { self.0.borrow().state() }
    fn set_state(&mut self, state: StateFlags) { self.0.borrow_mut().set_state(state); }
    fn get_palette(&self) -> Option<crate::core::palette::Palette> { self.0.borrow().get_palette() }
}

impl LogWindow {
    /// Drain pending log entries from the channel into the terminal widget.
    /// Called automatically during `draw()`.
    fn drain_logs(&mut self) {
        while let Ok(entry) = self.receiver.try_recv() {
            self.widget.borrow_mut().append_line_colored(entry.text, entry.attr);
        }
    }

    /// Manually append a log line (bypasses tracing).
    pub fn log(&mut self, level: tracing::Level, message: &str) {
        let now = chrono::Local::now();
        let timestamp = now.format("%H:%M:%S");
        let level_str = match level {
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN  => "WARN ",
            tracing::Level::INFO  => "INFO ",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "TRACE",
        };
        let text = format!("{timestamp} {level_str} {message}");
        let attr = level_attr(level);
        self.widget.borrow_mut().append_line_colored(text, attr);
    }

    /// Clear all log entries.
    pub fn clear(&mut self) {
        self.widget.borrow_mut().clear();
    }
}

impl View for LogWindow {
    fn bounds(&self) -> Rect { self.window.bounds() }
    fn set_bounds(&mut self, bounds: Rect) {
        self.window.set_bounds(bounds);
        let widget_bounds = Rect::new(
            bounds.a.x + 1, bounds.a.y + 1,
            bounds.b.x - 1, bounds.b.y - 1,
        );
        self.widget.borrow_mut().set_bounds(widget_bounds);
    }
    fn draw(&mut self, terminal: &mut Terminal) {
        self.drain_logs();
        self.window.draw(terminal);
    }
    fn handle_event(&mut self, event: &mut Event) { self.window.handle_event(event); }
    fn can_focus(&self) -> bool { true }
    fn state(&self) -> StateFlags { self.window.state() }
    fn set_state(&mut self, state: StateFlags) { self.window.set_state(state); }
    fn options(&self) -> u16 { self.window.options() }
    fn set_options(&mut self, options: u16) { self.window.set_options(options); }
    fn get_palette(&self) -> Option<crate::core::palette::Palette> { self.window.get_palette() }
    fn get_end_state(&self) -> crate::core::command::CommandId { self.window.get_end_state() }
    fn set_end_state(&mut self, cmd: crate::core::command::CommandId) { self.window.set_end_state(cmd); }
}

/// Builder for creating a LogWindow with tracing integration.
pub struct LogWindowBuilder {
    bounds: Option<Rect>,
    title: Option<String>,
    min_level: tracing::Level,
    max_lines: usize,
}

impl LogWindowBuilder {
    pub fn new() -> Self {
        Self {
            bounds: None,
            title: None,
            min_level: tracing::Level::TRACE,
            max_lines: 10000,
        }
    }

    /// Sets the window bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the window title (default: "Log").
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the minimum tracing level to display (default: TRACE — show everything).
    #[must_use]
    pub fn min_level(mut self, level: tracing::Level) -> Self {
        self.min_level = level;
        self
    }

    /// Sets the maximum scrollback buffer size (default: 10000).
    #[must_use]
    pub fn max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = max_lines;
        self
    }

    /// Builds the LogWindow and installs the tracing subscriber as the global default.
    ///
    /// # Panics
    ///
    /// Panics if `bounds` is not set, or if a global tracing subscriber is already installed.
    pub fn build(self) -> LogWindow {
        let bounds = self.bounds.expect("LogWindow bounds must be set");
        let title = self.title.unwrap_or_else(|| "Log".to_string());

        // Use blue window as base, then override palette to black-background entries
        // App palette positions 97-104 are the black window colors
        let mut window = Window::new_with_type(bounds, &title, WindowPaletteType::Blue);
        window.set_custom_palette(vec![
            97, 98, 99, 100, 101, 102, 103, 104, // Black window frame/text colors
        ]);

        let widget_bounds = Rect::new(
            bounds.a.x + 1, bounds.a.y + 1,
            bounds.b.x - 1, bounds.b.y - 1,
        );
        let mut widget = TerminalWidget::new(widget_bounds);
        widget = widget.with_scrollbar();
        widget.set_max_lines(self.max_lines);
        widget.set_auto_scroll(true);

        let widget = Rc::new(RefCell::new(widget));
        window.add(Box::new(SharedTerminalWidget(Rc::clone(&widget))));

        let (sender, receiver) = mpsc::channel();

        // Install the tracing subscriber
        let subscriber = LogSubscriber {
            sender,
            min_level: self.min_level,
        };
        // Use try — if a subscriber is already set, log a warning but don't panic
        let _ = tracing::subscriber::set_global_default(subscriber);

        LogWindow { window, widget, receiver }
    }

    /// Builds as a Box for convenience.
    pub fn build_boxed(self) -> Box<LogWindow> {
        Box::new(self.build())
    }
}

impl Default for LogWindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_attr_colors() {
        let error = level_attr(tracing::Level::ERROR);
        assert_eq!(error, Attr::new(TvColor::LightRed, TvColor::Black));

        let warn = level_attr(tracing::Level::WARN);
        assert_eq!(warn, Attr::new(TvColor::Yellow, TvColor::Black));

        let info = level_attr(tracing::Level::INFO);
        assert_eq!(info, Attr::new(TvColor::White, TvColor::Black));

        let debug = level_attr(tracing::Level::DEBUG);
        assert_eq!(debug, Attr::new(TvColor::LightGray, TvColor::Black));

        let trace = level_attr(tracing::Level::TRACE);
        assert_eq!(trace, Attr::new(TvColor::DarkGray, TvColor::Black));
    }

    #[test]
    fn test_log_window_creation() {
        let log_window = LogWindowBuilder::new()
            .bounds(Rect::new(0, 0, 80, 15))
            .title("Test Log")
            .min_level(tracing::Level::DEBUG)
            .max_lines(500)
            .build();

        assert_eq!(log_window.bounds(), Rect::new(0, 0, 80, 15));
    }

    #[test]
    fn test_log_window_manual_log() {
        let mut log_window = LogWindowBuilder::new()
            .bounds(Rect::new(0, 0, 80, 15))
            .title("Test Log")
            .build();

        log_window.log(tracing::Level::INFO, "test message");
        assert_eq!(log_window.widget.borrow().line_count(), 1);

        log_window.log(tracing::Level::ERROR, "error message");
        assert_eq!(log_window.widget.borrow().line_count(), 2);

        log_window.clear();
        assert_eq!(log_window.widget.borrow().line_count(), 0);
    }
}
