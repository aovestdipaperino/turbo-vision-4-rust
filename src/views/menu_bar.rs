// MenuBar - Horizontal menu bar
//
// Matches Borland: TMenuBar (menubar.h, tmenubar.cc)
//
// A MenuBar displays a horizontal bar of menu items at the top of the screen.
// Clicking on a menu opens a dropdown with that menu's items.
//
// Borland inheritance: TView → TMenuView → TMenuBar
// Rust composition: View + MenuViewer → MenuBar

use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, KB_ALT_F, KB_ALT_H, KB_ENTER, KB_ESC, KB_LEFT, KB_RIGHT, KB_ESC_F, KB_ESC_H, KB_ESC_E, KB_ESC_S, KB_ESC_V, KB_ESC_ESC, MB_LEFT_BUTTON};
use crate::core::draw::DrawBuffer;
use crate::core::palette::colors;
use crate::core::state::StateFlags;
use crate::core::menu_data::{Menu, MenuItem};
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal, draw_shadow};
use super::menu_viewer::{MenuViewer, MenuViewerState};

/// SubMenu represents a top-level menu with dropdown items
pub struct SubMenu {
    pub name: String,
    pub menu: Menu,
}

impl SubMenu {
    pub fn new(name: &str, menu: Menu) -> Self {
        Self {
            name: name.to_string(),
            menu,
        }
    }
}

/// MenuBar - Horizontal menu bar at top of screen
///
/// Matches Borland: TMenuBar
pub struct MenuBar {
    bounds: Rect,
    submenus: Vec<SubMenu>,
    menu_positions: Vec<i16>,  // X positions of each menu for dropdown placement
    active_menu_idx: Option<usize>,  // Which submenu is currently open
    menu_state: MenuViewerState,  // State for dropdown menu items
    state: StateFlags,
}

impl MenuBar {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            submenus: Vec::new(),
            menu_positions: Vec::new(),
            active_menu_idx: None,
            menu_state: MenuViewerState::new(),
            state: 0,
        }
    }

    pub fn add_submenu(&mut self, submenu: SubMenu) {
        self.submenus.push(submenu);
        self.menu_positions.push(0);  // Will be updated during draw
    }

    /// Open a specific submenu by index
    fn open_menu(&mut self, menu_idx: usize) {
        if menu_idx < self.submenus.len() {
            self.active_menu_idx = Some(menu_idx);
            self.menu_state.set_menu(self.submenus[menu_idx].menu.clone());
        }
    }

    /// Close the currently open menu
    fn close_menu(&mut self) {
        self.active_menu_idx = None;
        self.menu_state = MenuViewerState::new();
    }

    /// Draw the dropdown menu
    fn draw_dropdown(&self, terminal: &mut Terminal, menu_idx: usize) {
        if menu_idx >= self.submenus.len() || menu_idx >= self.menu_positions.len() {
            return;
        }

        let menu_x = self.menu_positions[menu_idx];
        let menu_y = self.bounds.a.y + 1;
        let menu = &self.submenus[menu_idx].menu;

        // Calculate dropdown width
        let mut max_text_width = 12;
        let mut max_shortcut_width = 0;
        for item in &menu.items {
            match item {
                MenuItem::Regular { text, shortcut, .. } => {
                    let text_len = text.replace('~', "").len();
                    max_text_width = max_text_width.max(text_len);
                    if let Some(s) = shortcut {
                        max_shortcut_width = max_shortcut_width.max(s.len());
                    }
                }
                MenuItem::SubMenu { text, .. } => {
                    let text_len = text.replace('~', "").len();
                    max_text_width = max_text_width.max(text_len + 3); // +3 for arrow
                }
                MenuItem::Separator => {}
            }
        }

        let dropdown_width = if max_shortcut_width > 0 {
            max_text_width + 2 + max_shortcut_width + 2
        } else {
            max_text_width + 2
        };
        let dropdown_height = menu.items.len() as i16;

        // Draw top border
        let mut top_buf = DrawBuffer::new(dropdown_width);
        top_buf.put_char(0, '┌', colors::MENU_NORMAL);
        for i in 1..dropdown_width - 1 {
            top_buf.put_char(i, '─', colors::MENU_NORMAL);
        }
        top_buf.put_char(dropdown_width - 1, '┐', colors::MENU_NORMAL);
        write_line_to_terminal(terminal, menu_x, menu_y, &top_buf);

        // Draw menu items
        for (i, item) in menu.items.iter().enumerate() {
            let mut item_buf = DrawBuffer::new(dropdown_width);
            let is_selected = Some(i) == self.menu_state.current;

            match item {
                MenuItem::Separator => {
                    item_buf.put_char(0, '├', colors::MENU_NORMAL);
                    for j in 1..dropdown_width - 1 {
                        item_buf.put_char(j, '─', colors::MENU_NORMAL);
                    }
                    item_buf.put_char(dropdown_width - 1, '┤', colors::MENU_NORMAL);
                }
                MenuItem::Regular { text, enabled, shortcut, .. } => {
                    let attr = if is_selected && *enabled {
                        colors::MENU_SELECTED
                    } else if !enabled {
                        colors::MENU_DISABLED
                    } else {
                        colors::MENU_NORMAL
                    };

                    // Borders and fill
                    item_buf.put_char(0, '│', colors::MENU_NORMAL);
                    for j in 1..dropdown_width - 1 {
                        item_buf.put_char(j, ' ', attr);
                    }

                    // Draw text with accelerator
                    let mut x = 1;
                    let mut chars = text.chars();
                    while let Some(ch) = chars.next() {
                        if x >= dropdown_width - 1 { break; }
                        if ch == '~' {
                            let shortcut_attr = if is_selected && *enabled {
                                colors::MENU_SELECTED
                            } else if !enabled {
                                colors::MENU_DISABLED
                            } else {
                                colors::MENU_SHORTCUT
                            };
                            while let Some(sc) = chars.next() {
                                if sc == '~' { break; }
                                if x < dropdown_width - 1 {
                                    item_buf.put_char(x, sc, shortcut_attr);
                                    x += 1;
                                }
                            }
                        } else {
                            item_buf.put_char(x, ch, attr);
                            x += 1;
                        }
                    }

                    // Draw shortcut right-aligned
                    if let Some(shortcut_text) = shortcut {
                        let shortcut_x = dropdown_width.saturating_sub(shortcut_text.len() + 1);
                        for (i, ch) in shortcut_text.chars().enumerate() {
                            if shortcut_x + i < dropdown_width - 1 {
                                item_buf.put_char(shortcut_x + i, ch, attr);
                            }
                        }
                    }

                    item_buf.put_char(dropdown_width - 1, '│', colors::MENU_NORMAL);
                }
                MenuItem::SubMenu { text, .. } => {
                    let attr = if is_selected {
                        colors::MENU_SELECTED
                    } else {
                        colors::MENU_NORMAL
                    };

                    item_buf.put_char(0, '│', colors::MENU_NORMAL);
                    for j in 1..dropdown_width - 1 {
                        item_buf.put_char(j, ' ', attr);
                    }

                    // Draw text
                    let mut x = 1;
                    for ch in text.replace('~', "").chars() {
                        if x >= dropdown_width - 2 { break; }
                        item_buf.put_char(x, ch, attr);
                        x += 1;
                    }

                    // Draw arrow
                    item_buf.put_char(dropdown_width - 2, '►', attr);
                    item_buf.put_char(dropdown_width - 1, '│', colors::MENU_NORMAL);
                }
            }

            write_line_to_terminal(terminal, menu_x, menu_y + 1 + i as i16, &item_buf);
        }

        // Draw bottom border
        let mut bottom_buf = DrawBuffer::new(dropdown_width);
        bottom_buf.put_char(0, '└', colors::MENU_NORMAL);
        for i in 1..dropdown_width - 1 {
            bottom_buf.put_char(i, '─', colors::MENU_NORMAL);
        }
        bottom_buf.put_char(dropdown_width - 1, '┘', colors::MENU_NORMAL);
        write_line_to_terminal(terminal, menu_x, menu_y + 1 + dropdown_height, &bottom_buf);

        // Draw shadow
        let shadow_bounds = crate::core::geometry::Rect::new(
            menu_x,
            menu_y,
            menu_x + dropdown_width as i16,
            menu_y + dropdown_height + 2,
        );
        draw_shadow(terminal, shadow_bounds, crate::core::state::SHADOW_ATTR);
    }
}

impl View for MenuBar {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', colors::MENU_NORMAL, width);

        // Draw menu names and track their positions
        let mut x: usize = 1;
        for (i, submenu) in self.submenus.iter().enumerate() {
            // Store the starting position of this menu
            if i < self.menu_positions.len() {
                self.menu_positions[i] = x as i16;
            }

            let attr = if self.active_menu_idx == Some(i) {
                colors::MENU_SELECTED
            } else {
                colors::MENU_NORMAL
            };

            // Parse ~X~ for highlighting
            buf.put_char(x, ' ', attr);
            x += 1;

            let mut chars = submenu.name.chars();
            while let Some(ch) = chars.next() {
                if ch == '~' {
                    // Read all characters until closing ~ in shortcut color
                    let shortcut_attr = if self.active_menu_idx == Some(i) {
                        colors::MENU_SELECTED
                    } else {
                        colors::MENU_SHORTCUT
                    };
                    while let Some(shortcut_ch) = chars.next() {
                        if shortcut_ch == '~' {
                            break;
                        }
                        buf.put_char(x, shortcut_ch, shortcut_attr);
                        x += 1;
                    }
                } else {
                    buf.put_char(x, ch, attr);
                    x += 1;
                }
            }

            buf.put_char(x, ' ', attr);
            x += 1;
        }

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);

        // Draw dropdown if active
        if let Some(idx) = self.active_menu_idx {
            self.draw_dropdown(terminal, idx);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        match event.what {
            EventType::MouseDown if event.mouse.buttons & MB_LEFT_BUTTON != 0 => {
                let mouse_pos = event.mouse.pos;

                // Click on menu bar - toggle/switch menus
                if mouse_pos.y == self.bounds.a.y {
                    for (i, &menu_x) in self.menu_positions.iter().enumerate() {
                        if i < self.submenus.len() {
                            let menu_width = self.submenus[i].name.replace('~', "").len() as i16 + 2;
                            if mouse_pos.x >= menu_x && mouse_pos.x < menu_x + menu_width {
                                if self.active_menu_idx == Some(i) {
                                    self.close_menu();
                                } else {
                                    self.open_menu(i);
                                }
                                event.clear();
                                return;
                            }
                        }
                    }
                    // Clicked on bar but not on a menu - close
                    if self.active_menu_idx.is_some() {
                        self.close_menu();
                        event.clear();
                        return;
                    }
                }

                // Click on dropdown - select item
                if self.active_menu_idx.is_some() {
                    // Try standard menu event handling first
                    if self.handle_menu_event(event) {
                        // If an item was clicked, execute it
                        let command = self.menu_state.get_current_item().and_then(|item| {
                            if let MenuItem::Regular { command, enabled: true, .. } = item {
                                Some(*command)
                            } else {
                                None
                            }
                        });

                        if let Some(cmd) = command {
                            self.close_menu();
                            *event = Event::command(cmd);
                            return;
                        }
                    } else {
                        // Clicked outside - close
                        self.close_menu();
                        event.clear();
                    }
                }
            }
            EventType::MouseMove => {
                if let Some(menu_idx) = self.active_menu_idx {
                    let mouse_pos = event.mouse.pos;

                    // Hover over dropdown items
                    if mouse_pos.y > self.bounds.a.y {
                        self.handle_menu_event(event);
                    }

                    // Hover over different menu on bar - switch
                    if mouse_pos.y == self.bounds.a.y {
                        for (i, &menu_x) in self.menu_positions.iter().enumerate() {
                            if i < self.submenus.len() && i != menu_idx {
                                let menu_width = self.submenus[i].name.replace('~', "").len() as i16 + 2;
                                if mouse_pos.x >= menu_x && mouse_pos.x < menu_x + menu_width {
                                    self.open_menu(i);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            EventType::Keyboard => {
                // Hot keys to open specific menus
                if self.active_menu_idx.is_none() {
                    let menu_to_open = match event.key_code {
                        KB_ALT_F | KB_ESC_F | crate::core::event::KB_F1 if !self.submenus.is_empty() => Some(0),
                        KB_ESC_E if self.submenus.len() > 1 => Some(1),
                        KB_ESC_S if self.submenus.len() > 2 => Some(2),
                        KB_ESC_V if self.submenus.len() > 3 => Some(3),
                        KB_ALT_H | KB_ESC_H if self.submenus.len() > 1 => Some(self.submenus.len() - 1),
                        _ => None,
                    };

                    if let Some(idx) = menu_to_open {
                        self.open_menu(idx);
                        event.clear();
                        return;
                    }
                }

                // Handle dropdown navigation
                if let Some(menu_idx) = self.active_menu_idx {
                    match event.key_code {
                        KB_ESC | KB_ESC_ESC => {
                            self.close_menu();
                            event.clear();
                        }
                        KB_LEFT => {
                            // Previous menu
                            let prev = if menu_idx > 0 { menu_idx - 1 } else { self.submenus.len() - 1 };
                            self.open_menu(prev);
                            event.clear();
                        }
                        KB_RIGHT => {
                            // Next menu
                            self.open_menu((menu_idx + 1) % self.submenus.len());
                            event.clear();
                        }
                        KB_ENTER => {
                            // Execute current item
                            let command = self.menu_state.get_current_item().and_then(|item| {
                                if let MenuItem::Regular { command, enabled: true, .. } = item {
                                    Some(*command)
                                } else {
                                    None
                                }
                            });

                            if let Some(cmd) = command {
                                self.close_menu();
                                *event = Event::command(cmd);
                                return;
                            }
                            event.clear();
                        }
                        _ => {
                            // Use MenuViewer trait for Up/Down/Accelerators
                            self.handle_menu_event(event);
                        }
                    }
                }
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
}

// Implement MenuViewer trait for dropdown menu items
impl MenuViewer for MenuBar {
    fn menu_state(&self) -> &MenuViewerState {
        &self.menu_state
    }

    fn menu_state_mut(&mut self) -> &mut MenuViewerState {
        &mut self.menu_state
    }

    fn get_item_rect(&self, item_index: usize) -> crate::core::geometry::Rect {
        if let Some(menu_idx) = self.active_menu_idx {
            if menu_idx < self.menu_positions.len() {
                let menu_x = self.menu_positions[menu_idx];
                let menu_y = self.bounds.a.y + 1;
                // Items start at menu_y + 1 (after top border), each is 1 row
                return crate::core::geometry::Rect::new(
                    menu_x,
                    menu_y + 1 + item_index as i16,
                    menu_x + 20, // Approximate width
                    menu_y + 2 + item_index as i16,
                );
            }
        }
        crate::core::geometry::Rect::new(0, 0, 0, 0)
    }
}