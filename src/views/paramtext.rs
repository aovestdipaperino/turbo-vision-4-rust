// (C) 2025 - Enzo Lombardi

//! ParamText view - parametrized text display with dynamic string substitution.

use super::view::{write_line_to_terminal, View};
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::PARAM_TEXT_NORMAL;
use crate::terminal::Terminal;

/// ParamText - Static text with parameter substitution
/// Displays text with placeholders like "File: %s" or "Total: %d items"
pub struct ParamText {
    bounds: Rect,
    template: String,
    text: String,
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
}

impl ParamText {
    /// Create a new parameterized text control
    /// The template string can contain placeholders:
    /// - %s for string substitution
    /// - %d for numeric substitution
    /// - %% for a literal %
    pub fn new(bounds: Rect, template: &str) -> Self {
        Self {
            bounds,
            template: template.to_string(),
            text: template.to_string(),
            owner: None,
            owner_type: super::view::OwnerType::None,
        }
    }

    /// Set the template text
    pub fn set_template(&mut self, template: &str) {
        self.template = template.to_string();
        self.text = template.to_string();
    }

    /// Set a string parameter (replaces first %s)
    pub fn set_param_str(&mut self, value: &str) {
        self.text = self.template.replacen("%s", value, 1);
    }

    /// Set multiple string parameters
    pub fn set_params_str(&mut self, values: &[&str]) {
        let mut result = self.template.clone();
        for value in values {
            result = result.replacen("%s", value, 1);
        }
        self.text = result;
    }

    /// Set a numeric parameter (replaces first %d)
    pub fn set_param_num(&mut self, value: i64) {
        let value_str = value.to_string();
        self.text = self.template.replacen("%d", &value_str, 1);
    }

    /// Set text with both string and numeric parameters
    /// Example: template = "File: %s, Size: %d bytes"
    ///          set_params("test.txt", &[1024])
    pub fn set_params(&mut self, str_params: &[&str], num_params: &[i64]) {
        let mut result = self.template.clone();

        // Replace string parameters
        for value in str_params {
            result = result.replacen("%s", value, 1);
        }

        // Replace numeric parameters
        for value in num_params {
            let value_str = value.to_string();
            result = result.replacen("%d", &value_str, 1);
        }

        // Replace %% with %
        result = result.replace("%%", "%");

        self.text = result;
    }

    /// Get the current displayed text
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Get the template
    pub fn get_template(&self) -> &str {
        &self.template
    }
}

impl View for ParamText {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width_clamped() as usize;
        let height = self.bounds.height_clamped() as usize;

        // ParamText palette indices:
        // 1: Normal text
        let normal_attr = self.map_color(PARAM_TEXT_NORMAL);

        // Split text into lines
        let lines: Vec<&str> = self.text.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if i >= height {
                break;
            }

            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', normal_attr, width);

            // Truncate line if too long
            let display_text = if line.len() > width {
                &line[..width]
            } else {
                line
            };

            buf.move_str(0, display_text, normal_attr);
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }

        // Fill remaining lines with spaces
        for i in lines.len()..height {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', normal_attr, width);
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // ParamText doesn't handle events
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_STATIC_TEXT))
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paramtext_creation() {
        let param_text = ParamText::new(Rect::new(0, 0, 20, 1), "Hello %s");
        assert_eq!(param_text.get_template(), "Hello %s");
        assert_eq!(param_text.get_text(), "Hello %s");
    }

    #[test]
    fn test_paramtext_set_param_str() {
        let mut param_text = ParamText::new(Rect::new(0, 0, 20, 1), "Hello %s");
        param_text.set_param_str("World");
        assert_eq!(param_text.get_text(), "Hello World");
    }

    #[test]
    fn test_paramtext_set_param_num() {
        let mut param_text = ParamText::new(Rect::new(0, 0, 20, 1), "Count: %d");
        param_text.set_param_num(42);
        assert_eq!(param_text.get_text(), "Count: 42");
    }

    #[test]
    fn test_paramtext_multiple_params() {
        let mut param_text = ParamText::new(Rect::new(0, 0, 40, 1), "File: %s, Size: %d bytes");
        param_text.set_params(&["test.txt"], &[1024]);
        assert_eq!(param_text.get_text(), "File: test.txt, Size: 1024 bytes");
    }

    #[test]
    fn test_paramtext_multiple_strings() {
        let mut param_text = ParamText::new(Rect::new(0, 0, 40, 1), "From %s to %s");
        param_text.set_params_str(&["Alice", "Bob"]);
        assert_eq!(param_text.get_text(), "From Alice to Bob");
    }

    #[test]
    fn test_paramtext_escape_percent() {
        let mut param_text = ParamText::new(Rect::new(0, 0, 30, 1), "Progress: %d%%");
        param_text.set_params(&[], &[75]);
        assert_eq!(param_text.get_text(), "Progress: 75%");
    }

    #[test]
    fn test_paramtext_set_template() {
        let mut param_text = ParamText::new(Rect::new(0, 0, 20, 1), "Hello %s");
        param_text.set_param_str("World");
        assert_eq!(param_text.get_text(), "Hello World");

        param_text.set_template("Goodbye %s");
        param_text.set_param_str("Moon");
        assert_eq!(param_text.get_text(), "Goodbye Moon");
    }

    #[test]
    fn test_paramtext_complex() {
        let mut param_text = ParamText::new(
            Rect::new(0, 0, 60, 1),
            "User: %s, Files: %d, Size: %d MB (%d%%)",
        );
        param_text.set_params(&["admin"], &[150, 2048, 95]);
        assert_eq!(
            param_text.get_text(),
            "User: admin, Files: 150, Size: 2048 MB (95%)"
        );
    }
}

/// Builder for creating param texts with a fluent API.
pub struct ParamTextBuilder {
    bounds: Option<Rect>,
    template: Option<String>,
}

impl ParamTextBuilder {
    pub fn new() -> Self {
        Self { bounds: None, template: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn template(mut self, template: impl Into<String>) -> Self {
        self.template = Some(template.into());
        self
    }

    pub fn build(self) -> ParamText {
        let bounds = self.bounds.expect("ParamText bounds must be set");
        let template = self.template.expect("ParamText template must be set");
        ParamText::new(bounds, &template)
    }

    pub fn build_boxed(self) -> Box<ParamText> {
        Box::new(self.build())
    }
}

impl Default for ParamTextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
