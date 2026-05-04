// (C) 2025 - Enzo Lombardi

//! StatusLine view - bottom status bar with keyboard shortcuts and context help.

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KeyCode, MB_LEFT_BUTTON};
use crate::core::draw::DrawBuffer;
use crate::core::command::CommandId;
use crate::core::command_set;
use crate::core::palette::{STATUSLINE_DISABLED, STATUSLINE_NORMAL, STATUSLINE_SELECTED, STATUSLINE_SELECTED_SHORTCUT, STATUSLINE_SHORTCUT};
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

pub struct StatusItem {
    pub text: String,
    pub key_code: KeyCode,
    pub command: CommandId,
}

impl StatusItem {
    pub fn new(text: &str, key_code: KeyCode, command: CommandId) -> Self {
        Self {
            text: text.to_string(),
            key_code,
            command,
        }
    }
}

pub struct StatusLine {
    bounds: Rect,
    items: Vec<StatusItem>,
    item_positions: Vec<(i16, i16)>, // (start_x, end_x) for each item
    selected_item: Option<usize>,    // Currently hovered/selected item
    hint_text: Option<String>,       // Context-sensitive help text
    options: u16,
    palette_chain: Option<crate::core::palette_chain::PaletteChainNode>,
}

impl StatusLine {
    pub fn new(bounds: Rect, items: Vec<StatusItem>) -> Self {
        use crate::core::state::OF_PRE_PROCESS;

        Self {
            bounds,
            items,
            item_positions: Vec::new(),
            selected_item: None,
            hint_text: None,
            options: OF_PRE_PROCESS,  // Status line processes in pre-process phase (matches Borland)
        palette_chain: None,
        }
    }

    /// Set the hint text to display on the right side of the status line
    pub fn set_hint(&mut self, hint: Option<String>) {
        self.hint_text = hint;
    }

    /// Draw the status line with optional selected item highlighting
    fn draw_select(&mut self, terminal: &mut Terminal, selected: Option<usize>) {
        let width = self.bounds.width_clamped() as usize;
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

        let mut x = 0;  // Start at position 0 (Borland starts at i=0)
        for (idx, item) in self.items.iter().enumerate() {
            if x + item.text.len() + 4 < width {  // Need space for: space + text + space + separator
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
                                break;  // Found closing tilde
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

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    /// Find which item the mouse is currently over
    fn item_mouse_is_in(&self, mouse_x: i16) -> Option<usize> {
        for (i, &(start_x, end_x)) in self.item_positions.iter().enumerate() {
            if i < self.items.len() {
                let absolute_start = self.bounds.a.x + start_x;
                let absolute_end = self.bounds.a.x + end_x;

                if mouse_x >= absolute_start && mouse_x < absolute_end {
                    return Some(i);
                }
            }
        }
        None
    }
}

impl View for StatusLine {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        // Draw with current selection (if any)
        self.draw_select(terminal, self.selected_item);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Handle mouse clicks on status items with tracking (like Borland)
        if event.what == EventType::MouseDown {
            let mouse_pos = event.mouse.pos;

            if event.mouse.buttons & MB_LEFT_BUTTON != 0 && mouse_pos.y == self.bounds.a.y {
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
                        if item.command != 0 {
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
            if mouse_pos.y == self.bounds.a.y {
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
                    *event = Event::command(item.command);
                    return;
                }
            }
        }
    }

    fn options(&self) -> u16 {
        self.options
    }

    fn set_options(&mut self, options: u16) {
        self.options = options;
    }

    fn set_palette_chain(&mut self, node: Option<crate::core::palette_chain::PaletteChainNode>) {
        self.palette_chain = node;
    }

    fn get_palette_chain(&self) -> Option<&crate::core::palette_chain::PaletteChainNode> {
        self.palette_chain.as_ref()
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_STATUSLINE))
    }
}
