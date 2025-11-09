// (C) 2025 - Enzo Lombardi

//! Label view - static text display with optional linked control focus.

use super::view::{write_line_to_terminal, View};
use crate::core::draw::DrawBuffer;
use crate::core::event::Event;
use crate::core::geometry::Rect;
use crate::core::palette::{LABEL_NORMAL, LABEL_SHORTCUT};
use crate::terminal::Terminal;

pub struct Label {
    bounds: Rect,
    text: String,
    link: Option<usize>, // Index of linked control in parent group
    owner: Option<*const dyn View>,
}

impl Label {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            link: None,
            owner: None,
        }
    }

    /// Set the linked control index
    /// Matches Borland: TLabel constructor takes TView* aLink parameter
    /// When label is clicked, focus transfers to the linked control
    pub fn set_link(&mut self, link_index: usize) {
        self.link = Some(link_index);
    }
}

impl View for Label {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);

        // Label palette indices:
        // 1: Normal, 2: Selected, 3: Shortcut
        let normal_attr = self.map_color(LABEL_NORMAL);
        let shortcut_attr = self.map_color(LABEL_SHORTCUT);

        buf.move_char(0, ' ', normal_attr, width);
        buf.move_str_with_shortcut(0, &self.text, normal_attr, shortcut_attr);

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {
        // Labels don't handle events directly
        // Focus linking is handled by Group
    }

    /// Return the linked control index for this label
    /// Matches Borland: TLabel::link field
    fn label_link(&self) -> Option<usize> {
        self.link
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_LABEL))
    }
}
