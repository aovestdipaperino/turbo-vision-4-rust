// (C) 2025 - Enzo Lombardi

//! MenuBox view - popup menu container for dropdown menus.
// MenuBox - Popup menu container
//
// Matches Borland: TMenuBox (menubox.h, tmenubox.cc)
//
// A MenuBox is a popup/dropdown menu that displays menu items in a
// vertical list with borders and optional shadows.
//
// Borland inheritance: TView → TMenuView → TMenuBox
// Rust composition: View + MenuViewer → MenuBox

use crate::core::geometry::{Rect, Point};
use crate::core::event::{Event, EventType, KB_ENTER, KB_ESC, KB_ESC_ESC, MB_LEFT_BUTTON};
use crate::core::draw::DrawBuffer;
use crate::core::state::{StateFlags, SF_SHADOW};
use crate::core::menu_data::{Menu, MenuItem};
use crate::core::command::CommandId;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal, draw_shadow};
use super::menu_viewer::{MenuViewer, MenuViewerState};

/// MenuBox - Popup menu container
///
/// Displays a vertical menu with borders, shadows, and selection highlighting.
/// Matches Borland: TMenuBox
pub struct MenuBox {
    bounds: Rect,
    menu_state: MenuViewerState,
    state: StateFlags,
    owner: Option<*const dyn View>,
}

impl MenuBox {
    /// Create a new menu box
    ///
    /// Matches Borland: TMenuBox(bounds, menu, parentMenu)
    ///
    /// The bounds will be adjusted to fit the menu content.
    pub fn new(position: Point, menu: Menu) -> Self {
        // Calculate required size for menu
        let bounds = Self::calculate_bounds(position, &menu);

        Self {
            bounds,
            menu_state: MenuViewerState::with_menu(menu),
            state: SF_SHADOW, // MenuBox has shadow by default
            owner: None,
        }
    }

    /// Calculate bounds for menu based on content
    ///
    /// Matches Borland: getRect() in tmenubox.cc
    fn calculate_bounds(position: Point, menu: &Menu) -> Rect {
        let mut width = 10; // Minimum width
        let mut height = 2; // Top and bottom borders

        // Calculate maximum width needed
        for item in &menu.items {
            let item_width = match item {
                MenuItem::Regular { text, shortcut, .. } => {
                    let text_len = text.replace('~', "").len();
                    let shortcut_len = shortcut.as_ref().map(|s| s.len() + 2).unwrap_or(0);
                    text_len + shortcut_len + 6 // Padding
                }
                MenuItem::SubMenu { text, .. } => {
                    let text_len = text.replace('~', "").len();
                    text_len + 6 + 3 // Padding + submenu arrow
                }
                MenuItem::Separator => 4, // Just borders
            };
            width = width.max(item_width);
            height += 1;
        }

        Rect::new(
            position.x,
            position.y,
            position.x + width as i16,
            position.y + height as i16,
        )
    }

    /// Get the command from the currently selected item
    pub fn get_selected_command(&self) -> Option<CommandId> {
        self.menu_state.get_current_item().and_then(|item| item.command())
    }

    /// Execute the menu modally
    ///
    /// Matches Borland: TMenuView::execute()
    /// Returns the selected command, or 0 if cancelled
    pub fn execute(&mut self, terminal: &mut Terminal) -> CommandId {
        loop {
            // Draw the menu
            self.draw(terminal);
            let _ = terminal.flush();

            // Get event
            if let Ok(Some(mut event)) = terminal.poll_event(std::time::Duration::from_millis(50)) {
                // Handle the event
                self.handle_event(&mut event);

                // Check for selection or cancellation
                match event.what {
                    EventType::Command => {
                        // Command event - return the command
                        return event.command;
                    }
                    EventType::Nothing => {
                        // Event was handled, continue
                    }
                    _ => {
                        // Other events - continue
                    }
                }
            }
        }
    }
}

impl View for MenuBox {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        if height < 2 || width < 4 {
            return; // Too small to draw
        }

        let menu = match self.menu_state.get_menu() {
            Some(m) => m,
            None => return,
        };

        // MenuBox palette indices (same as MenuBar):
        // 1: Normal, 2: Selected, 3: Disabled, 4: Shortcut
        let normal_attr = self.map_color(1);
        let selected_attr = self.map_color(2);
        let disabled_attr = self.map_color(3);
        let shortcut_attr = self.map_color(4);

        // Draw top border
        let mut buf = DrawBuffer::new(width);
        buf.put_char(0, '┌', normal_attr);
        for i in 1..width - 1 {
            buf.put_char(i, '─', normal_attr);
        }
        buf.put_char(width - 1, '┐', normal_attr);
        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);

        // Draw menu items
        let mut y = 1;
        for (idx, item) in menu.items.iter().enumerate() {
            if y >= height - 1 {
                break; // No more room
            }

            let mut buf = DrawBuffer::new(width);
            let is_selected = Some(idx) == self.menu_state.current;

            match item {
                MenuItem::Separator => {
                    // Draw separator line
                    buf.put_char(0, '├', normal_attr);
                    for i in 1..width - 1 {
                        buf.put_char(i, '─', normal_attr);
                    }
                    buf.put_char(width - 1, '┤', normal_attr);
                }
                MenuItem::Regular { text, enabled, shortcut, .. } => {
                    let color = if is_selected {
                        if *enabled {
                            selected_attr
                        } else {
                            selected_attr // Disabled but selected
                        }
                    } else if *enabled {
                        normal_attr
                    } else {
                        disabled_attr
                    };

                    // Left border
                    buf.put_char(0, '│', normal_attr);

                    // Fill with spaces
                    for i in 1..width - 1 {
                        buf.put_char(i, ' ', color);
                    }

                    // Draw text with accelerator highlighting
                    let mut x = 2;
                    let mut chars = text.chars();
                    while let Some(ch) = chars.next() {
                        if ch == '~' {
                            // Read accelerator
                            if let Some(accel_ch) = chars.next() {
                                if accel_ch == '~' {
                                    break; // End of accelerator
                                }
                                let accel_color = if is_selected {
                                    selected_attr
                                } else {
                                    shortcut_attr
                                };
                                buf.put_char(x, accel_ch, accel_color);
                                x += 1;
                            }
                        } else {
                            buf.put_char(x, ch, color);
                            x += 1;
                        }
                    }

                    // Draw shortcut right-aligned
                    if let Some(shortcut_text) = shortcut {
                        let shortcut_x = width - shortcut_text.len() - 2;
                        for (i, ch) in shortcut_text.chars().enumerate() {
                            buf.put_char(shortcut_x + i, ch, color);
                        }
                    }

                    // Right border
                    buf.put_char(width - 1, '│', normal_attr);
                }
                MenuItem::SubMenu { text, .. } => {
                    let color = if is_selected {
                        selected_attr
                    } else {
                        normal_attr
                    };

                    // Left border
                    buf.put_char(0, '│', normal_attr);

                    // Fill with spaces
                    for i in 1..width - 1 {
                        buf.put_char(i, ' ', color);
                    }

                    // Draw text
                    let mut x = 2;
                    let mut chars = text.chars();
                    while let Some(ch) = chars.next() {
                        if ch == '~' {
                            if let Some(accel_ch) = chars.next() {
                                if accel_ch == '~' {
                                    break;
                                }
                                let accel_color = if is_selected {
                                    selected_attr
                                } else {
                                    shortcut_attr
                                };
                                buf.put_char(x, accel_ch, accel_color);
                                x += 1;
                            }
                        } else {
                            buf.put_char(x, ch, color);
                            x += 1;
                        }
                    }

                    // Draw submenu arrow
                    buf.put_char(width - 2, '►', color);

                    // Right border
                    buf.put_char(width - 1, '│', normal_attr);
                }
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &buf);
            y += 1;
        }

        // Draw bottom border
        let mut buf = DrawBuffer::new(width);
        buf.put_char(0, '└', normal_attr);
        for i in 1..width - 1 {
            buf.put_char(i, '─', normal_attr);
        }
        buf.put_char(width - 1, '┘', normal_attr);
        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &buf);

        // Draw shadow
        if self.state & SF_SHADOW != 0 {
            draw_shadow(terminal, self.bounds, crate::core::state::SHADOW_ATTR);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Try standard menu navigation first
        if self.handle_menu_event(event) {
            return;
        }

        // Handle MenuBox-specific events
        match event.what {
            EventType::Keyboard => {
                match event.key_code {
                    KB_ENTER => {
                        // Activate current item
                        if let Some(item) = self.menu_state.get_current_item() {
                            match item {
                                MenuItem::Regular { command, enabled: true, .. } => {
                                    *event = Event::command(*command);
                                }
                                _ => {
                                    event.clear();
                                }
                            }
                        }
                    }
                    KB_ESC | KB_ESC_ESC => {
                        // Cancel menu - return 0 to signal cancellation
                        *event = Event::command(0);
                    }
                    _ => {}
                }
            }
            EventType::MouseDown => {
                let mouse_pos = event.mouse.pos;

                if event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    // Check if clicked outside menu - cancel
                    if !self.bounds.contains(mouse_pos) {
                        *event = Event::command(0); // Cancel
                        return;
                    }

                    // Track which item is under the mouse
                    if let Some(menu) = self.menu_state.get_menu() {
                        for (idx, _item) in menu.items.iter().enumerate() {
                            let item_rect = self.get_item_rect(idx);
                            if item_rect.contains(mouse_pos) {
                                // Update selection to clicked item
                                self.menu_state.current = Some(idx);
                                break;
                            }
                        }
                    }
                    event.clear();
                }
            }
            EventType::MouseUp => {
                let mouse_pos = event.mouse.pos;

                if event.mouse.buttons & MB_LEFT_BUTTON != 0 {
                    // Check if clicked outside menu - cancel
                    if !self.bounds.contains(mouse_pos) {
                        *event = Event::command(0); // Cancel
                        return;
                    }
                }

                // Execute the currently selected item on mouse up
                if let Some(item) = self.menu_state.get_current_item() {
                    if let MenuItem::Regular { command, enabled: true, .. } = item {
                        *event = Event::command(*command);
                        return;
                    }
                }
                event.clear();
            }
            _ => {}
        }
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<crate::core::palette::Palette> {
        use crate::core::palette::{Palette, palettes};
        Some(Palette::from_slice(palettes::CP_MENU_BAR))
    }
}

impl MenuViewer for MenuBox {
    fn menu_state(&self) -> &MenuViewerState {
        &self.menu_state
    }

    fn menu_state_mut(&mut self) -> &mut MenuViewerState {
        &mut self.menu_state
    }

    fn get_item_rect(&self, item_index: usize) -> Rect {
        // Items start at y=1 (after top border)
        // Each item is 1 row tall
        Rect::new(
            self.bounds.a.x,
            self.bounds.a.y + 1 + item_index as i16,
            self.bounds.b.x,
            self.bounds.a.y + 2 + item_index as i16,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::menu_data::MenuBuilder;

    #[test]
    fn test_menubox_creation() {
        let menu = MenuBuilder::new()
            .item("~O~pen", 100, 0)
            .item("~S~ave", 101, 0)
            .build();

        let menubox = MenuBox::new(Point::new(10, 5), menu);

        assert_eq!(menubox.bounds.a.x, 10);
        assert_eq!(menubox.bounds.a.y, 5);
        assert!(menubox.bounds.width() >= 10); // At least minimum width
        assert_eq!(menubox.bounds.height(), 4); // 2 items + 2 borders
    }

    #[test]
    fn test_menubox_with_separators() {
        let menu = MenuBuilder::new()
            .item("Item 1", 100, 0)
            .separator()
            .item("Item 2", 101, 0)
            .build();

        let menubox = MenuBox::new(Point::new(0, 0), menu);

        assert_eq!(menubox.bounds.height(), 5); // 2 items + 1 separator + 2 borders
    }

    #[test]
    fn test_menubox_get_item_rect() {
        let menu = MenuBuilder::new()
            .item("Item 1", 100, 0)
            .item("Item 2", 101, 0)
            .build();

        let menubox = MenuBox::new(Point::new(10, 5), menu);

        let rect0 = menubox.get_item_rect(0);
        assert_eq!(rect0.a.y, 6); // Position 5 + 1 (border)

        let rect1 = menubox.get_item_rect(1);
        assert_eq!(rect1.a.y, 7); // Position 5 + 2 (border + item)
    }

    #[test]
    fn test_menubox_selection() {
        let menu = MenuBuilder::new()
            .item("Item 1", 100, 0)
            .item("Item 2", 101, 0)
            .build();

        let mut menubox = MenuBox::new(Point::new(0, 0), menu);

        assert_eq!(menubox.current_item(), Some(0));

        menubox.menu_state_mut().select_next();
        assert_eq!(menubox.current_item(), Some(1));

        assert_eq!(menubox.get_selected_command(), Some(101));
    }
}
