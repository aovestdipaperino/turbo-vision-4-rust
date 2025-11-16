// (C) 2025 - Enzo Lombardi

//! Label view - static text display with optional linked control focus.

use super::view::{write_line_to_terminal, View, ViewId};
use super::group::Group;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType};
use crate::core::geometry::Rect;
use crate::core::palette::{LABEL_NORMAL, LABEL_SHORTCUT};
use crate::core::state::OF_POST_PROCESS;
use crate::terminal::Terminal;

pub struct Label {
    bounds: Rect,
    text: String,
    link: Option<ViewId>, // ID of linked control
    owner: Option<*const dyn View>,
    owner_type: super::view::OwnerType,
    state: u16,
    options: u16,
}

impl Label {
    pub fn new(bounds: Rect, text: &str) -> Self {
        Self {
            bounds,
            text: text.to_string(),
            link: None,
            owner: None,
            owner_type: super::view::OwnerType::Dialog, // Labels default to Dialog context
            state: 0,
            options: OF_POST_PROCESS, // Labels need PostProcess to handle keyboard shortcuts
        }
    }

    /// Set the linked control by its ViewId
    /// Matches Borland: TLabel constructor takes TView* aLink parameter
    /// When label is clicked, focus transfers to the linked control
    pub fn set_link(&mut self, view_id: ViewId) {
        self.link = Some(view_id);
    }

    /// Extract the hotkey character from the label text
    /// Returns the uppercase character following the first '~', or None if no hotkey
    /// Matches Borland: hotKey() function
    fn get_hotkey(&self) -> Option<char> {
        let mut chars = self.text.chars();
        while let Some(ch) = chars.next() {
            if ch == '~' {
                // Next character is the hotkey
                if let Some(hotkey) = chars.next() {
                    return Some(hotkey.to_uppercase().next().unwrap_or(hotkey));
                }
            }
        }
        None
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
        let width = self.bounds.width_clamped() as usize;
        let mut buf = DrawBuffer::new(width);

        // Label palette indices:
        // 1: Normal, 2: Selected, 3: Shortcut
        let normal_attr = self.map_color(LABEL_NORMAL);
        let shortcut_attr = self.map_color(LABEL_SHORTCUT);

        buf.move_char(0, ' ', normal_attr, width);
        buf.move_str_with_shortcut(0, &self.text, normal_attr, shortcut_attr);

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Labels handle keyboard shortcuts in PostProcess phase
        // Matches Borland: TLabel::handleEvent() with ofPostProcess flag
        if event.what == EventType::Keyboard {
            // Check if we have a linked control and a hotkey
            if let (Some(link_id), Some(hotkey)) = (self.link, self.get_hotkey()) {
                // Check if the pressed key matches our Alt+letter shortcut
                // The key code for Alt+letter is stored in the high byte (scan code)
                // We need to check if it matches KB_ALT_{LETTER}
                use crate::core::event::*;

                let alt_code = match hotkey {
                    'A' => Some(KB_ALT_A),
                    'B' => Some(KB_ALT_B),
                    'C' => Some(KB_ALT_C),
                    'D' => Some(KB_ALT_D),
                    'E' => Some(KB_ALT_E),
                    'F' => Some(KB_ALT_F),
                    'G' => Some(KB_ALT_G),
                    'H' => Some(KB_ALT_H),
                    'I' => Some(KB_ALT_I),
                    'J' => Some(KB_ALT_J),
                    'K' => Some(KB_ALT_K),
                    'L' => Some(KB_ALT_L),
                    'M' => Some(KB_ALT_M),
                    'N' => Some(KB_ALT_N),
                    'O' => Some(KB_ALT_O),
                    'P' => Some(KB_ALT_P),
                    'Q' => Some(KB_ALT_Q),
                    'R' => Some(KB_ALT_R),
                    'S' => Some(KB_ALT_S),
                    'T' => Some(KB_ALT_T),
                    'U' => Some(KB_ALT_U),
                    'V' => Some(KB_ALT_V),
                    'W' => Some(KB_ALT_W),
                    'X' => Some(KB_ALT_X),
                    'Y' => Some(KB_ALT_Y),
                    'Z' => Some(KB_ALT_Z),
                    _ => None,
                };

                if let Some(expected_code) = alt_code {
                    if event.key_code == expected_code {
                        // Hotkey matched! Focus the linked control
                        // Matches Borland: TLabel calls link->select()
                        if let Some(owner_ptr) = self.owner {
                            // SAFETY: We assume the owner is a Group, which is the case
                            // for labels added to dialogs/windows
                            unsafe {
                                let group = &mut *(owner_ptr as *mut Group);
                                if group.focus_by_view_id(link_id) {
                                    event.clear(); // Event consumed
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Return the linked control ViewId for this label
    /// Matches Borland: TLabel::link field
    fn label_link(&self) -> Option<ViewId> {
        self.link
    }

    fn state(&self) -> u16 {
        self.state
    }

    fn set_state(&mut self, state: u16) {
        self.state = state;
    }

    fn options(&self) -> u16 {
        self.options
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_owner_type(&self) -> super::view::OwnerType {
        self.owner_type
    }

    fn set_owner_type(&mut self, owner_type: super::view::OwnerType) {
        self.owner_type = owner_type;
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{palettes, Palette};
        Some(Palette::from_slice(palettes::CP_LABEL))
    }
}

/// Builder for creating labels with a fluent API.
pub struct LabelBuilder {
    bounds: Option<Rect>,
    text: Option<String>,
    link: Option<ViewId>,
}

impl LabelBuilder {
    pub fn new() -> Self {
        Self { bounds: None, text: None, link: None }
    }

    #[must_use]
    pub fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    #[must_use]
    pub fn link(mut self, view_id: ViewId) -> Self {
        self.link = Some(view_id);
        self
    }

    pub fn build(self) -> Label {
        let bounds = self.bounds.expect("Label bounds must be set");
        let text = self.text.expect("Label text must be set");
        let mut label = Label::new(bounds, &text);
        if let Some(link) = self.link {
            label.link = Some(link);
        }
        label
    }

    pub fn build_boxed(self) -> Box<Label> {
        Box::new(self.build())
    }
}

impl Default for LabelBuilder {
    fn default() -> Self {
        Self::new()
    }
}
