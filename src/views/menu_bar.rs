// (C) 2025 - Enzo Lombardi

//! MenuBar view - horizontal top menu bar with dropdown submenus.
// MenuBar - Horizontal menu bar
//
// Matches Borland: TMenuBar (menubar.h, tmenubar.cc)
//
// A MenuBar displays a horizontal bar of menu items at the top of the screen.
// Clicking on a menu opens a dropdown with that menu's items.
//
// Borland inheritance: TView → TMenuView → TMenuBar
// Rust composition: View + MenuViewer → MenuBar

use super::menu_box::MenuBox;
use super::menu_viewer::{MenuViewer, MenuViewerState};
use super::view::{View, write_line_to_terminal};
use crate::core::draw::DrawBuffer;
use crate::core::event::{
    Event, EventType, KB_ALT_A, KB_ALT_B, KB_ALT_C, KB_ALT_D, KB_ALT_E, KB_ALT_F, KB_ALT_G, KB_ALT_H, KB_ALT_I, KB_ALT_J, KB_ALT_K, KB_ALT_L, KB_ALT_M, KB_ALT_N, KB_ALT_O, KB_ALT_P, KB_ALT_Q,
    KB_ALT_R, KB_ALT_S, KB_ALT_T, KB_ALT_U, KB_ALT_V, KB_ALT_W, KB_ALT_X, KB_ALT_Y, KB_ALT_Z, KB_ENTER, KB_ESC, KB_ESC_ESC, KB_F1, KB_F10, KB_LEFT, KB_RIGHT, KeyCode, MB_LEFT_BUTTON,
};
use crate::core::geometry::{Point, Rect};
use crate::core::menu_data::{Menu, MenuItem};
use crate::core::state::StateFlags;
use crate::terminal::Terminal;

// MenuBar palette indices (matches Borland TMenuView)
const MENU_NORMAL: u8 = 1; // Normal item text
const MENU_SELECTED: u8 = 2; // Selected item text
const MENU_DISABLED: u8 = 3; // Disabled item text
const MENU_SHORTCUT: u8 = 4; // Shortcut/accelerator text

/// SubMenu represents a top-level menu with dropdown items
pub struct SubMenu {
    pub name: String,
    pub menu: Menu,
}

impl SubMenu {
    pub fn new(name: &str, menu: Menu) -> Self {
        Self { name: name.to_string(), menu }
    }
}

/// Extract the hotkey character from a menu name with ~X~ markers
///
/// Given a string like "~F~ile" or "~W~indow", returns the character between the tildes.
/// Returns None if no valid hotkey marker is found.
///
/// Examples:
/// - "~F~ile" -> Some('F')
/// - "~W~indow" -> Some('W')
/// - "No hotkey" -> None
fn extract_hotkey(text: &str) -> Option<char> {
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '~' {
            // Found opening ~, next character is the hotkey
            if let Some(hotkey_ch) = chars.next() {
                // Verify it's followed by closing ~
                if chars.next() == Some('~') {
                    return Some(hotkey_ch);
                }
            }
        }
    }
    None
}

/// MenuBar - Horizontal menu bar at top of screen
///
/// Matches Borland: TMenuBar
pub struct MenuBar {
    bounds: Rect,
    submenus: Vec<SubMenu>,
    menu_positions: Vec<i16>,       // X positions of each menu for dropdown placement
    active_menu_idx: Option<usize>, // Which submenu is currently open
    menu_state: MenuViewerState,    // State for dropdown menu items
    state: StateFlags,
    owner: Option<*const dyn View>,
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
            owner: None,
        }
    }

    pub fn add_submenu(&mut self, submenu: SubMenu) {
        self.submenus.push(submenu);
        self.menu_positions.push(0); // Will be updated during draw
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

    /// Find a submenu index by matching Alt+Letter hotkey with ~X~ markers in menu names
    ///
    /// Scans all submenus for a name containing ~X~ where X matches the Alt+Letter keypress.
    /// For example, "~F~ile" matches KB_ALT_F, "~W~indow" matches KB_ALT_W, etc.
    ///
    /// Returns Some(index) if a matching menu is found, None otherwise.
    fn find_menu_by_hotkey(&self, key_code: KeyCode) -> Option<usize> {
        // Map Alt+Letter keycodes to their corresponding character
        // Use lowercase for comparison since menu hotkeys are case-insensitive
        let hotkey_char = match key_code {
            KB_ALT_A => 'a',
            KB_ALT_B => 'b',
            KB_ALT_C => 'c',
            KB_ALT_D => 'd',
            KB_ALT_E => 'e',
            KB_ALT_F => 'f',
            KB_ALT_G => 'g',
            KB_ALT_H => 'h',
            KB_ALT_I => 'i',
            KB_ALT_J => 'j',
            KB_ALT_K => 'k',
            KB_ALT_L => 'l',
            KB_ALT_M => 'm',
            KB_ALT_N => 'n',
            KB_ALT_O => 'o',
            KB_ALT_P => 'p',
            KB_ALT_Q => 'q',
            KB_ALT_R => 'r',
            KB_ALT_S => 's',
            KB_ALT_T => 't',
            KB_ALT_U => 'u',
            KB_ALT_V => 'v',
            KB_ALT_W => 'w',
            KB_ALT_X => 'x',
            KB_ALT_Y => 'y',
            KB_ALT_Z => 'z',
            _ => return None, // Not an Alt+Letter key
        };

        // Search through all submenus for a matching hotkey
        for (idx, submenu) in self.submenus.iter().enumerate() {
            // Extract the hotkey character from between ~X~ markers
            if let Some(menu_hotkey) = extract_hotkey(&submenu.name) {
                if menu_hotkey.to_ascii_lowercase() == hotkey_char {
                    return Some(idx);
                }
            }
        }

        None
    }

    /// Show a cascading submenu for the currently selected item
    /// Returns Some(command) if a command was selected, None if cancelled
    pub fn check_cascading_submenu(&mut self, terminal: &mut Terminal) -> Option<u16> {
        self.show_cascading_submenu(terminal)
    }

    /// Show a cascading submenu for the currently selected item (internal)
    fn show_cascading_submenu(&mut self, terminal: &mut Terminal) -> Option<u16> {
        // Get the current selected item
        let current_item = self.menu_state.get_current_item()?;

        // Check if it's a SubMenu item
        if let MenuItem::SubMenu { menu, .. } = current_item {
            // Calculate position for the cascading menu
            let current_idx = self.menu_state.current?;
            let menu_idx = self.active_menu_idx?;

            // Position submenu to the right of the dropdown
            let dropdown_x = self.menu_positions.get(menu_idx).copied().unwrap_or(0);
            let item_y = self.bounds.a.y + 2 + current_idx as i16; // +1 for bar, +1 for top border

            // Calculate dropdown width (similar to draw_dropdown logic)
            let parent_menu = &self.submenus[menu_idx].menu;
            let mut max_text_width = 10;
            for item in &parent_menu.items {
                match item {
                    MenuItem::Regular { text, shortcut, .. } => {
                        let text_len = text.replace('~', "").len();
                        max_text_width = max_text_width.max(text_len);
                        if let Some(s) = shortcut {
                            max_text_width = max_text_width.max(text_len + s.len() + 2);
                        }
                    }
                    MenuItem::SubMenu { text, .. } => {
                        let text_len = text.replace('~', "").len();
                        max_text_width = max_text_width.max(text_len + 3);
                    }
                    MenuItem::Separator => {}
                }
            }
            let dropdown_width = max_text_width + 4;

            let submenu_x = dropdown_x + dropdown_width as i16 - 1;
            let position = Point::new(submenu_x, item_y);

            // Create and execute the cascading menu
            let mut menu_box = MenuBox::new(position, menu.clone());
            let command = menu_box.execute(terminal);

            return Some(command);
        }

        None
    }

    /// Draw the dropdown menu
    fn draw_dropdown(&self, terminal: &mut Terminal, menu_idx: usize) {
        if menu_idx >= self.submenus.len() || menu_idx >= self.menu_positions.len() {
            return;
        }

        let menu_x = self.menu_positions[menu_idx];
        let menu_y = self.bounds.a.y + 1;
        let menu = &self.submenus[menu_idx].menu;

        let normal_attr = self.map_color(MENU_NORMAL);
        let selected_attr = self.map_color(MENU_SELECTED);
        let disabled_attr = self.map_color(MENU_DISABLED);
        let shortcut_attr = self.map_color(MENU_SHORTCUT);

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
        top_buf.put_char(0, '┌', normal_attr);
        for i in 1..dropdown_width - 1 {
            top_buf.put_char(i, '─', normal_attr);
        }
        top_buf.put_char(dropdown_width - 1, '┐', normal_attr);
        write_line_to_terminal(terminal, menu_x, menu_y, &top_buf);

        // Draw menu items
        for (i, item) in menu.items.iter().enumerate() {
            let mut item_buf = DrawBuffer::new(dropdown_width);
            let is_selected = Some(i) == self.menu_state.current;

            match item {
                MenuItem::Separator => {
                    item_buf.put_char(0, '├', normal_attr);
                    for j in 1..dropdown_width - 1 {
                        item_buf.put_char(j, '─', normal_attr);
                    }
                    item_buf.put_char(dropdown_width - 1, '┤', normal_attr);
                }
                MenuItem::Regular { text, enabled, shortcut, .. } => {
                    let attr = if is_selected && *enabled {
                        selected_attr
                    } else if !enabled {
                        disabled_attr
                    } else {
                        normal_attr
                    };

                    // Borders and fill
                    item_buf.put_char(0, '│', normal_attr);
                    for j in 1..dropdown_width - 1 {
                        item_buf.put_char(j, ' ', attr);
                    }

                    // Draw text with accelerator
                    let mut x = 1;
                    let mut chars = text.chars();
                    while let Some(ch) = chars.next() {
                        if x >= dropdown_width - 1 {
                            break;
                        }
                        if ch == '~' {
                            let item_shortcut_attr = if is_selected && *enabled {
                                selected_attr
                            } else if !enabled {
                                disabled_attr
                            } else {
                                shortcut_attr
                            };
                            while let Some(sc) = chars.next() {
                                if sc == '~' {
                                    break;
                                }
                                if x < dropdown_width - 1 {
                                    item_buf.put_char(x, sc, item_shortcut_attr);
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
                                item_buf.put_char(shortcut_x + i, ch, shortcut_attr);
                            }
                        }
                    }

                    item_buf.put_char(dropdown_width - 1, '│', normal_attr);
                }
                MenuItem::SubMenu { text, .. } => {
                    let attr = if is_selected { selected_attr } else { normal_attr };

                    item_buf.put_char(0, '│', normal_attr);
                    for j in 1..dropdown_width - 1 {
                        item_buf.put_char(j, ' ', attr);
                    }

                    // Draw text
                    let mut x = 1;
                    for ch in text.replace('~', "").chars() {
                        if x >= dropdown_width - 2 {
                            break;
                        }
                        item_buf.put_char(x, ch, attr);
                        x += 1;
                    }

                    // Draw arrow
                    item_buf.put_char(dropdown_width - 2, '►', attr);
                    item_buf.put_char(dropdown_width - 1, '│', normal_attr);
                }
            }

            write_line_to_terminal(terminal, menu_x, menu_y + 1 + i as i16, &item_buf);
        }

        // Draw bottom border
        let mut bottom_buf = DrawBuffer::new(dropdown_width);
        bottom_buf.put_char(0, '└', normal_attr);
        for i in 1..dropdown_width - 1 {
            bottom_buf.put_char(i, '─', normal_attr);
        }
        bottom_buf.put_char(dropdown_width - 1, '┘', normal_attr);
        write_line_to_terminal(terminal, menu_x, menu_y + 1 + dropdown_height, &bottom_buf);

        // Draw shadow
        let shadow_bounds = crate::core::geometry::Rect::new(menu_x, menu_y, menu_x + dropdown_width as i16, menu_y + dropdown_height + 2);
        crate::views::view::draw_shadow_bounds(terminal, shadow_bounds);
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

        let normal_attr = self.map_color(MENU_NORMAL);
        let selected_attr = self.map_color(MENU_SELECTED);
        let shortcut_attr = self.map_color(MENU_SHORTCUT);

        buf.move_char(0, ' ', normal_attr, width);

        // Draw menu names and track their positions
        let mut x: usize = 1;
        for (i, submenu) in self.submenus.iter().enumerate() {
            // Store the starting position of this menu
            if i < self.menu_positions.len() {
                self.menu_positions[i] = x as i16;
            }

            let attr = if self.active_menu_idx == Some(i) { selected_attr } else { normal_attr };

            // Parse ~X~ for highlighting
            buf.put_char(x, ' ', attr);
            x += 1;

            let mut chars = submenu.name.chars();
            while let Some(ch) = chars.next() {
                if ch == '~' {
                    // Read all characters until closing ~ in shortcut color
                    let menu_shortcut_attr = if self.active_menu_idx == Some(i) { selected_attr } else { shortcut_attr };
                    while let Some(shortcut_ch) = chars.next() {
                        if shortcut_ch == '~' {
                            break;
                        }
                        buf.put_char(x, shortcut_ch, menu_shortcut_attr);
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
                if let Some(menu_idx) = self.active_menu_idx {
                    let mouse_pos = event.mouse.pos;

                    // Calculate dropdown bounds
                    let (dropdown_bounds, item_count) = if menu_idx < self.menu_positions.len() {
                        if let Some(menu) = self.menu_state.get_menu() {
                            let menu_x = self.menu_positions[menu_idx];
                            let menu_y = self.bounds.a.y + 1;
                            let item_count = menu.items.len();

                            // Dropdown bounds: top border + items + bottom border
                            let bounds = Rect::new(
                                menu_x,
                                menu_y,
                                menu_x + 20, // Approximate width
                                menu_y + 1 + item_count as i16 + 1,
                            );
                            (Some(bounds), item_count)
                        } else {
                            (None, 0)
                        }
                    } else {
                        (None, 0)
                    };

                    if let Some(bounds) = dropdown_bounds {
                        if bounds.contains(mouse_pos) {
                            // Find which item was clicked
                            for i in 0..item_count {
                                let item_rect = self.get_item_rect(i);
                                if item_rect.contains(mouse_pos) {
                                    self.menu_state.current = Some(i);

                                    // Don't execute or show submenu here
                                    // That will be handled by check_cascading_submenu() or on MouseUp
                                    event.clear();
                                    return;
                                }
                            }
                        } else {
                            // Clicked outside dropdown - close
                            self.close_menu();
                            event.clear();
                        }
                    }
                }
            }
            EventType::MouseUp => {
                if let Some(menu_idx) = self.active_menu_idx {
                    let mouse_pos = event.mouse.pos;

                    // Calculate dropdown bounds (same as MouseDown)
                    let (dropdown_bounds, item_count) = if menu_idx < self.menu_positions.len() {
                        if let Some(menu) = self.menu_state.get_menu() {
                            let menu_x = self.menu_positions[menu_idx];
                            let menu_y = self.bounds.a.y + 1;
                            let item_count = menu.items.len();

                            let bounds = Rect::new(menu_x, menu_y, menu_x + 20, menu_y + 1 + item_count as i16 + 1);
                            (Some(bounds), item_count)
                        } else {
                            (None, 0)
                        }
                    } else {
                        (None, 0)
                    };

                    if let Some(bounds) = dropdown_bounds {
                        if bounds.contains(mouse_pos) {
                            // Check if mouse up is on currently selected item
                            for i in 0..item_count {
                                let item_rect = self.get_item_rect(i);
                                if item_rect.contains(mouse_pos) && self.menu_state.current == Some(i) {
                                    // Check if it's a submenu - don't clear so check_cascading_submenu can handle it
                                    if let Some(item) = self.menu_state.get_current_item() {
                                        if matches!(item, MenuItem::SubMenu { .. }) {
                                            // Don't clear - will be handled by check_cascading_submenu
                                            return;
                                        }
                                    }

                                    // If it's a regular item, execute it
                                    let command = self
                                        .menu_state
                                        .get_current_item()
                                        .and_then(|item| if let MenuItem::Regular { command, enabled: true, .. } = item { Some(*command) } else { None });

                                    if let Some(cmd) = command {
                                        self.close_menu();
                                        *event = Event::command(cmd);
                                        return;
                                    }
                                    break;
                                }
                            }
                        }
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
                    // Special case: F10 always opens first menu, F1 always opens last (Help)
                    let menu_to_open = match event.key_code {
                        KB_F10 if !self.submenus.is_empty() => Some(0),
                        KB_F1 if !self.submenus.is_empty() => Some(self.submenus.len() - 1),
                        _ => {
                            // Dynamically match Alt+Letter based on ~X~ hotkeys in menu names
                            // This extracts the hotkey character from menu names like "~F~ile", "~W~indow", etc.
                            self.find_menu_by_hotkey(event.key_code)
                        }
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
                            // Just move to next menu, don't open submenus automatically
                            self.open_menu((menu_idx + 1) % self.submenus.len());
                            event.clear();
                        }
                        KB_ENTER => {
                            // Leave event uncleared if it's a submenu, so check_cascading_submenu can handle it
                            if let Some(item) = self.menu_state.get_current_item() {
                                if matches!(item, MenuItem::SubMenu { .. }) {
                                    // Don't clear - will be handled by check_cascading_submenu
                                    return;
                                }
                            }

                            // Execute current item (if it's a regular item)
                            let command = self
                                .menu_state
                                .get_current_item()
                                .and_then(|item| if let MenuItem::Regular { command, enabled: true, .. } = item { Some(*command) } else { None });

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
