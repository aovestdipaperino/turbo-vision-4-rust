// (C) 2025 - Enzo Lombardi

//! StaticText view - multi-line static text display with word wrapping.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::STATIC_TEXT_NORMAL;
use crate::terminal::Terminal;

pub struct StaticText {
    core: ViewCore,
    text: String,
    centered: bool,
}

impl StaticText {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            text: text.to_string(),
            centered: false,
        }
    }

    pub fn new_centered(bounds: Rect, text: &str) -> Self {
        Self {
            core: ViewCore {
                bounds,
                palette_chain: None,
                ..ViewCore::default()
            },
            text: text.to_string(),
            centered: true,
        }
    }
}

impl View for StaticText {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.core.bounds.width_clamped() as usize;
        let lines: Vec<&str> = self.text.split('\n').collect();

        // StaticText palette color index 1 = normal text
        let text_attr = self.map_color(STATIC_TEXT_NORMAL);

        for (i, line) in lines.iter().enumerate() {
            if i >= self.core.bounds.height_clamped() as usize {
                break;
            }
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', text_attr, width);

            // Calculate starting position based on centering
            // Strip control characters and ~ shortcut markers for display width calculation
            let start_pos = if self.centered {
                let display_len = line
                    .chars()
                    .filter(|c| !c.is_control() && *c != '~')
                    .count();
                if width > display_len {
                    (width - display_len) / 2
                } else {
                    0
                }
            } else {
                0
            };

            // For now, use same color for shortcuts (no separate shortcut color in StaticText palette)
            buf.move_str_with_shortcut(start_pos, line, text_attr, text_attr);
            write_line_to_terminal(
                terminal,
                0,
                i as i16,
                &buf,
            );
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Static text doesn't handle events
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_STATIC_TEXT))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Builder for creating static text views with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::views::static_text::StaticTextBuilder;
/// use turbo_vision::core::geometry::Rect;
///
/// // Create left-aligned static text
/// let text = StaticTextBuilder::new()
///     .bounds(Rect::new(2, 2, 40, 4))
///     .text("Hello, World!")
///     .build();
///
/// // Create centered static text
/// let text = StaticTextBuilder::new()
///     .bounds(Rect::new(2, 6, 40, 8))
///     .text("Centered Text")
///     .centered(true)
///     .build();
/// ```
pub struct StaticTextBuilder {
    bounds: Option<Rect>,
    text: Option<String>,
    centered: bool,
}

impl StaticTextBuilder {
    /// Creates a new StaticTextBuilder with default values.
    pub fn new() -> Self {
        Self {
            bounds: None,
            text: None,
            centered: false,
        }
    }

    /// Sets the static text bounds (required).
    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the text to display (required).
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets whether the text should be centered (default: false).
    #[must_use]
    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    /// Builds the StaticText.
    ///
    /// # Panics
    ///
    /// Panics if required fields (bounds, text) are not set.
    pub fn build(self) -> StaticText {
        let bounds = self.bounds.expect("StaticText bounds must be set");
        let text = self.text.expect("StaticText text must be set");

        StaticText {
            core: ViewCore::new(bounds),
            text,
            centered: self.centered,
        }
    }

    /// Builds the StaticText as a Box.
    pub fn build_boxed(self) -> Box<StaticText> {
        Box::new(self.build())
    }
}

impl Default for StaticTextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
