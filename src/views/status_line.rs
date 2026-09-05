// (C) 2025 - Enzo Lombardi

//! StatusLine view - bottom status bar with keyboard shortcuts and context help.

use super::view::{View, ViewCore, write_line_to_terminal};
use crate::core::command_set;
use crate::core::draw::DrawBuffer;
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::geometry::Rect;
use crate::core::palette::{
    STATUSLINE_DISABLED, STATUSLINE_NORMAL, STATUSLINE_SELECTED, STATUSLINE_SELECTED_SHORTCUT,
    STATUSLINE_SHORTCUT,
};
use crate::core::state::Options;
use crate::terminal::Terminal;

pub use crate::core::status_data::StatusItem;

pub struct StatusLine {
    core: ViewCore,
    items: Vec<StatusItem>,
    item_positions: Vec<(i16, i16)>, // (start_x, end_x) for each item
    selected_item: Option<usize>,    // Currently hovered/selected item
    hint_text: Option<String>,       // Context-sensitive help text
    /// Optional help-context-ranged item sets (Borland: TStatusDef chain).
    /// When set, update() swaps `items` based on the current help context.
    defs: Vec<crate::core::status_data::StatusDef>,
    current_help_ctx: u16,
    /// Optional right-aligned mode marker, re-queried on every draw
    right_indicator: Option<fn() -> Option<String>>,
}

impl StatusLine {
    pub fn new(bounds: Rect, items: Vec<StatusItem>) -> Self {
        Self {
            core: ViewCore {
                bounds,
                options: Options::PRE_PROCESS, // Status line processes in pre-process phase (matches Borland)
                palette_chain: None,
                ..ViewCore::default()
            },
            items,
            item_positions: Vec::new(),
            selected_item: None,
            hint_text: None,
            defs: Vec::new(),
            current_help_ctx: 0,
            right_indicator: None,
        }
    }

    /// Set a right-aligned marker drawn at the far end of the status line.
    ///
    /// The function is called on every draw and returns the text to show, or
    /// `None` to show nothing, so a mode marker tracks its flag with no
    /// rebuild.
    ///
    /// # Example
    /// ```ignore
    /// status_line.set_right_indicator(|| block_edit_mode().then(|| "▭ Block".to_string()));
    /// ```
    pub fn set_right_indicator(&mut self, indicator: fn() -> Option<String>) {
        self.right_indicator = Some(indicator);
    }

    /// Set the hint text to display on the right side of the status line
    /// Create a status line whose item sets switch with the help context.
    ///
    /// Matches Borland: `TStatusLine(bounds, TStatusDef chain)` — `update()`
    /// picks the first def whose `[min, max]` range contains the current
    /// help context.
    pub fn with_defs(bounds: Rect, defs: Vec<crate::core::status_data::StatusDef>) -> Self {
        let items = defs
            .first()
            .map(|d| Self::convert_items(&d.items))
            .unwrap_or_default();
        let mut line = Self::new(bounds, items);
        line.defs = defs;
        line
    }

    /// Switch the visible item set to the def matching `help_ctx`.
    ///
    /// Matches Borland: TStatusLine::update()/findItems(). No-op when the
    /// status line was built from a flat item list or the context is
    /// unchanged.
    pub fn update(&mut self, help_ctx: u16) {
        if self.defs.is_empty() || help_ctx == self.current_help_ctx {
            return;
        }
        self.current_help_ctx = help_ctx;
        if let Some(def) = self
            .defs
            .iter()
            .find(|d| d.min <= help_ctx && help_ctx <= d.max)
        {
            self.items = Self::convert_items(&def.items);
            self.item_positions.clear();
        }
    }

    /// Convert declarative status_data items into the view's item type.
    fn convert_items(items: &[crate::core::status_data::StatusItem]) -> Vec<StatusItem> {
        items
            .iter()
            .map(|i| StatusItem {
                text: i.text.clone(),
                key_code: i.key_code,
                command: i.command,
            })
            .collect()
    }

    pub fn set_hint(&mut self, hint: Option<String>) {
        self.hint_text = hint;
    }

    /// Draw the status line with optional selected item highlighting
    fn draw_select(&mut self, terminal: &mut Terminal, selected: Option<usize>) {
        let width = self.core.bounds.width_clamped() as usize;
        let mut buf = DrawBuffer::new(width);

        // StatusLine palette indices:
        // 1: Normal, 2: Shortcut, 3: Selected, 4: Selected shortcut, 5: Disabled
        let normal_attr = self.map_color(STATUSLINE_NORMAL);
        let shortcut_attr = self.map_color(STATUSLINE_SHORTCUT);
        let selected_attr = self.map_color(STATUSLINE_SELECTED);
        let selected_shortcut_attr = self.map_color(STATUSLINE_SELECTED_SHORTCUT);
        let disabled_attr = self.map_color(STATUSLINE_DISABLED);

        buf.move_char(0, ' ', normal_attr, width);

        // Clear previous item positions
        self.item_positions.clear();

        let mut x = 0; // Start at position 0 (Borland starts at i=0)
        for (idx, item) in self.items.iter().enumerate() {
            if x + item.text.len() + 4 < width {
                // Need space for: space + text + space + separator
                // Hit area starts at the leading space (matches Borland tstatusl.cc:204)
                let start_x = x as i16;

                // Determine color based on selection AND command-enable state.
                // Disabled items grey out (text + shortcut both render in
                // disabled_attr) so the user can see at a glance which shortcuts
                // are currently active. Mirrors Borland's tstatusl.cc:87-96.
                let is_selected = selected == Some(idx);
                let is_enabled = command_set::command_enabled(item.command);
                let item_normal = if !is_enabled {
                    disabled_attr
                } else if is_selected {
                    selected_attr
                } else {
                    normal_attr
                };
                let item_shortcut = if !is_enabled {
                    disabled_attr
                } else if is_selected {
                    selected_shortcut_attr
                } else {
                    shortcut_attr
                };

                // Draw leading space (Borland: b.moveChar(i, ' ', color, 1))
                buf.put_char(x, ' ', item_normal);
                x += 1;

                // Parse ~X~ for highlighting - everything between tildes is highlighted
                let mut chars = item.text.chars();
                while let Some(ch) = chars.next() {
                    if ch == '~' {
                        // Read all characters until closing ~ in highlight color
                        while let Some(shortcut_ch) = chars.next() {
                            if shortcut_ch == '~' {
                                break; // Found closing tilde
                            }
                            buf.put_char(x, shortcut_ch, item_shortcut);
                            x += 1;
                        }
                    } else {
                        buf.put_char(x, ch, item_normal);
                        x += 1;
                    }
                }

                // Draw trailing space (Borland: b.moveChar(i+l+1, ' ', color, 1))
                buf.put_char(x, ' ', item_normal);
                x += 1;

                // Hit area ends after the trailing space (matches Borland inc=2 spacing)
                let end_x = x as i16;
                self.item_positions.push((start_x, end_x));

                // Separator is always drawn in normal color, never highlighted
                buf.move_str(x, "│ ", normal_attr);
                x += 2;
            }
        }

        // Display hint text if available. Renders as many characters
        // as fit and ellipsises the tail on narrow terminals. No
        // leading separator — the hint starts where the items end so
        // the whole status row reads like one continuous line.
        if let Some(ref hint) = self.hint_text {
            if x + 1 < width {
                let avail = width - x;
                let chars: Vec<char> = hint.chars().collect();
                if chars.len() <= avail {
                    for (i, ch) in chars.iter().enumerate() {
                        buf.put_char(x + i, *ch, normal_attr);
                    }
                } else if avail >= 4 {
                    let take = avail - 3;
                    for (i, ch) in chars.iter().take(take).enumerate() {
                        buf.put_char(x + i, *ch, normal_attr);
                    }
                    for (i, ch) in "...".chars().enumerate() {
                        buf.put_char(x + take + i, ch, normal_attr);
                    }
                } else {
                    for (i, ch) in chars.iter().take(avail).enumerate() {
                        buf.put_char(x + i, *ch, normal_attr);
                    }
                }
            }
        }

        // Right-aligned mode marker (e.g. block-edit mode). Drawn last so it
        // wins over a long hint that would otherwise reach the right edge.
        if let Some(text) = self.right_indicator.and_then(|f| f()) {
            let len = text.chars().count();
            if len < width {
                let start = width - len - 1;
                for (i, ch) in text.chars().enumerate() {
                    buf.put_char(start + i, ch, shortcut_attr);
                }
            }
        }

        write_line_to_terminal(terminal, self.core.bounds.a.x, self.core.bounds.a.y, &buf);
    }

    /// Find which item the mouse is currently over
    fn item_mouse_is_in(&self, mouse_x: i16) -> Option<usize> {
        for (i, &(start_x, end_x)) in self.item_positions.iter().enumerate() {
            if i < self.items.len() {
                let absolute_start = self.core.bounds.a.x + start_x;
                let absolute_end = self.core.bounds.a.x + end_x;

                if mouse_x >= absolute_start && mouse_x < absolute_end {
                    return Some(i);
                }
            }
        }
        None
    }
}

impl View for StatusLine {
    fn core(&self) -> &ViewCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut ViewCore {
        &mut self.core
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Draw with current selection (if any)
        self.draw_select(terminal, self.selected_item);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Handle mouse clicks on status items with tracking (like Borland)
        if event.what == EventType::MouseDown {
            let mouse_pos = event.mouse.pos;

            if event.mouse.buttons & MB_LEFT_BUTTON != 0 && mouse_pos.y == self.core.bounds.a.y {
                // Track mouse movement while button is held down
                // Initial selection
                let selected_item = self.item_mouse_is_in(mouse_pos.x);
                if selected_item.is_some() {
                    self.selected_item = selected_item;
                    // Note: In full implementation, we'd redraw here with selection
                    // For now, we'll skip the redraw to avoid terminal borrow issues
                }

                // Clear the event since we're handling it
                event.clear();

                // If an item was selected, generate command
                if let Some(idx) = selected_item {
                    if idx < self.items.len() {
                        let item = &self.items[idx];
                        // Only act on enabled commands - disabled items are drawn
                        // greyed out and must not fire. Matches Borland:
                        // TStatusLine::handleEvent() checks commandEnabled().
                        if item.command != 0 && command_set::command_enabled(item.command) {
                            *event = Event::command(item.command);
                        }
                    }
                }

                // Reset selection
                self.selected_item = None;
                return;
            }
        }

        // Handle mouse move to show hover effect
        if event.what == EventType::MouseMove {
            let mouse_pos = event.mouse.pos;
            if mouse_pos.y == self.core.bounds.a.y {
                let hovered_item = self.item_mouse_is_in(mouse_pos.x);
                if hovered_item != self.selected_item {
                    self.selected_item = hovered_item;
                    // Note: Ideally we'd redraw here to show hover effect
                    // But without access to terminal in handle_event, we defer to next draw cycle
                }
            } else if self.selected_item.is_some() {
                self.selected_item = None;
            }
        }

        // Handle keyboard shortcuts
        if event.what == EventType::Keyboard {
            for item in &self.items {
                if event.key_code == item.key_code {
                    // Disabled commands don't fire; the event passes through
                    // unhandled. Matches Borland: TStatusLine::handleEvent()
                    // only converts the key when commandEnabled(command).
                    if command_set::command_enabled(item.command) {
                        *event = Event::command(item.command);
                        return;
                    }
                }
            }
        }
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_STATUSLINE))
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
    use crate::core::command::CommandId;
    use crate::core::event::KeyCode;
    use crate::core::geometry::Point;
    use crate::core::status_data::StatusItemBuilder;

    const TEST_KEY: KeyCode = 0x1234;

    fn make_status_line(command: CommandId) -> StatusLine {
        StatusLine::new(
            Rect::new(0, 0, 80, 1),
            vec![
                StatusItemBuilder::new()
                    .text("~F9~ Test")
                    .key_code(TEST_KEY)
                    .command(command)
                    .build(),
            ],
        )
    }

    #[test]
    fn test_disabled_status_key_passes_through_unhandled() {
        const TEST_CMD: CommandId = 520;
        command_set::disable_command(TEST_CMD);

        let mut status = make_status_line(TEST_CMD);
        let mut event = Event::keyboard(TEST_KEY);
        status.handle_event(&mut event);

        assert_eq!(
            event.what,
            EventType::Keyboard,
            "disabled shortcut must pass through unhandled"
        );
    }

    #[test]
    fn test_enabled_status_key_fires_command() {
        const TEST_CMD: CommandId = 521;
        command_set::enable_command(TEST_CMD);

        let mut status = make_status_line(TEST_CMD);
        let mut event = Event::keyboard(TEST_KEY);
        status.handle_event(&mut event);

        assert_eq!(event.what, EventType::Command);
        assert_eq!(event.command, TEST_CMD);
    }

    #[test]
    fn test_disabled_status_item_click_does_not_fire() {
        const TEST_CMD: CommandId = 522;
        command_set::disable_command(TEST_CMD);

        let mut status = make_status_line(TEST_CMD);
        // Simulate the hit area normally computed during draw()
        status.item_positions.push((0, 9));

        let mut event = Event::mouse(
            EventType::MouseDown,
            Point::new(3, 0),
            MB_LEFT_BUTTON,
            false,
        );
        status.handle_event(&mut event);

        assert_ne!(
            event.what,
            EventType::Command,
            "clicking a disabled status item must not fire its command"
        );
    }

    #[test]
    fn test_enabled_status_item_click_fires() {
        const TEST_CMD: CommandId = 523;
        command_set::enable_command(TEST_CMD);

        let mut status = make_status_line(TEST_CMD);
        status.item_positions.push((0, 9));

        let mut event = Event::mouse(
            EventType::MouseDown,
            Point::new(3, 0),
            MB_LEFT_BUTTON,
            false,
        );
        status.handle_event(&mut event);

        assert_eq!(event.what, EventType::Command);
        assert_eq!(event.command, TEST_CMD);
    }

    #[test]
    fn test_status_defs_switch_with_help_context() {
        use crate::core::status_data::StatusDef;

        let defs = vec![
            StatusDef::new(
                0,
                99,
                vec![
                    StatusItemBuilder::new()
                        .text("~Alt+X~ Exit")
                        .key("Alt+X")
                        .command(1)
                        .build(),
                ],
            ),
            StatusDef::new(
                100,
                199,
                vec![
                    StatusItemBuilder::new()
                        .text("~F2~ Save")
                        .key("F2")
                        .command(302)
                        .build(),
                    StatusItemBuilder::new()
                        .text("~F3~ Open")
                        .key("F3")
                        .command(301)
                        .build(),
                ],
            ),
        ];
        let mut line = StatusLine::with_defs(Rect::new(0, 24, 80, 25), defs);
        assert_eq!(line.items.len(), 1);

        // Entering the editor context (100..=199) swaps the item set
        line.update(150);
        assert_eq!(line.items.len(), 2);
        assert_eq!(line.items[0].text, "~F2~ Save");

        // Back to the default range
        line.update(5);
        assert_eq!(line.items.len(), 1);

        // Unknown context keeps the current set
        line.update(999);
        assert_eq!(line.items.len(), 1);
    }
}
