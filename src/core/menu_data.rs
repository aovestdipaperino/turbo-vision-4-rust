// (C) 2025 - Enzo Lombardi
// Borland-compatible menu data structures
// Matches the original Turbo Vision TMenuItem, TSubMenu, TMenu architecture
//
// This module provides data structures for building menus in a declarative way,
// matching Borland's approach while being Rust-idiomatic.

use crate::core::command::CommandId;
use crate::core::event::KeyCode;

/// Menu item - can be a regular command, a submenu, or a separator
///
/// Matches Borland: TMenuItem
///
/// In Borland, TMenuItem is a linked list node with a `next` pointer.
/// In Rust, we use Vec for type safety, but provide builder methods
/// that match Borland's ergonomics.
#[derive(Clone, Debug)]
pub enum MenuItem {
    /// Regular menu item that executes a command
    /// Matches Borland: TMenuItem with command
    Regular {
        /// Display text (use ~x~ to mark accelerator key)
        text: String,
        /// Command to execute when selected
        command: CommandId,
        /// Keyboard shortcut
        key_code: KeyCode,
        /// Help context ID
        help_ctx: u16,
        /// Whether item is enabled
        enabled: bool,
        /// Optional shortcut text to display (e.g., "Ctrl+O", "F3")
        shortcut: Option<String>,
    },
    /// Submenu item that opens a nested menu
    /// Matches Borland: TMenuItem with subMenu
    SubMenu {
        /// Display text (use ~x~ to mark accelerator key)
        text: String,
        /// Keyboard shortcut to open submenu
        key_code: KeyCode,
        /// Help context ID
        help_ctx: u16,
        /// Nested menu
        menu: Menu,
    },
    /// Separator line
    /// Matches Borland: TMenuItem with null name
    Separator,
}

impl MenuItem {
    /// Create a regular menu item
    ///
    /// Matches Borland: `TMenuItem(name, command, keyCode, helpCtx)`
    ///
    /// # Example
    /// ```ignore
    /// let item = MenuItem::new("~O~pen", CM_OPEN, KB_F3, hcOpen);
    /// ```
    pub fn new(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16) -> Self {
        Self::Regular {
            text: text.to_string(),
            command,
            key_code,
            help_ctx,
            enabled: true,
            shortcut: None,
        }
    }

    /// Create a menu item with display shortcut
    ///
    /// # Example
    /// ```ignore
    /// let item = MenuItem::with_shortcut("~O~pen", CM_OPEN, KB_F3, "F3", hcOpen);
    /// ```
    pub fn with_shortcut(text: &str, command: CommandId, key_code: KeyCode, shortcut: &str, help_ctx: u16) -> Self {
        Self::Regular {
            text: text.to_string(),
            command,
            key_code,
            help_ctx,
            enabled: true,
            shortcut: Some(shortcut.to_string()),
        }
    }

    /// Create a disabled menu item
    pub fn new_disabled(text: &str, command: CommandId, key_code: KeyCode, help_ctx: u16) -> Self {
        Self::Regular {
            text: text.to_string(),
            command,
            key_code,
            help_ctx,
            enabled: false,
            shortcut: None,
        }
    }

    /// Create a submenu item
    ///
    /// Matches Borland: `TMenuItem(name, keyCode, subMenu, helpCtx)`
    ///
    /// # Example
    /// ```ignore
    /// let item = MenuItem::submenu("~F~ile", KB_ALT_F, file_menu, hcFile);
    /// ```
    pub fn submenu(text: &str, key_code: KeyCode, menu: Menu, help_ctx: u16) -> Self {
        Self::SubMenu {
            text: text.to_string(),
            key_code,
            help_ctx,
            menu,
        }
    }

    /// Create a separator
    ///
    /// Matches Borland: `newLine()`
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Check if this item is selectable (not a separator and not disabled)
    pub fn is_selectable(&self) -> bool {
        match self {
            Self::Regular { enabled, .. } => *enabled,
            Self::SubMenu { .. } => true,
            Self::Separator => false,
        }
    }

    /// Extract the accelerator key from the text (character between ~ marks)
    pub fn get_accelerator(&self) -> Option<char> {
        let text = match self {
            Self::Regular { text, .. } | Self::SubMenu { text, .. } => text,
            Self::Separator => return None,
        };

        let mut chars = text.chars();
        while let Some(ch) = chars.next() {
            if ch == '~' {
                // Next char is the accelerator
                if let Some(accel) = chars.next() {
                    return Some(accel.to_ascii_lowercase());
                }
            }
        }
        None
    }

    /// Get the display text (with ~ markers)
    pub fn text(&self) -> &str {
        match self {
            Self::Regular { text, .. } | Self::SubMenu { text, .. } => text,
            Self::Separator => "",
        }
    }

    /// Get the command (for Regular items only)
    pub fn command(&self) -> Option<CommandId> {
        match self {
            Self::Regular { command, .. } => Some(*command),
            _ => None,
        }
    }

    /// Get the shortcut display text (for Regular items only)
    pub fn shortcut(&self) -> Option<&str> {
        match self {
            Self::Regular { shortcut, .. } => shortcut.as_deref(),
            _ => None,
        }
    }
}

/// Menu - a collection of menu items
///
/// Matches Borland: TMenu
///
/// In Borland, TMenu has:
/// - `items`: pointer to first item in linked list
/// - `deflt`: pointer to default item
///
/// In Rust, we use Vec for type safety and provide convenient builders.
#[derive(Clone, Debug)]
pub struct Menu {
    /// Menu items
    pub items: Vec<MenuItem>,
    /// Index of default item (if any)
    pub default_index: Option<usize>,
}

impl Menu {
    /// Create an empty menu
    ///
    /// Matches Borland: `TMenu()`
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            default_index: None,
        }
    }

    /// Create a menu from items
    ///
    /// Matches Borland: `TMenu(itemList)`
    pub fn from_items(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            default_index: None,
        }
    }

    /// Create a menu with a default item
    ///
    /// Matches Borland: `TMenu(itemList, deflt)`
    pub fn with_default(items: Vec<MenuItem>, default_index: usize) -> Self {
        Self {
            items,
            default_index: Some(default_index),
        }
    }

    /// Add an item to the menu
    ///
    /// Matches Borland: appending to TMenuItem linked list
    pub fn add(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Set the default item by index
    pub fn set_default(&mut self, index: usize) {
        if index < self.items.len() {
            self.default_index = Some(index);
        }
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if menu is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing menus fluently
///
/// This provides a Borland-style builder API while being Rust-idiomatic.
///
/// # Example (Borland-style)
/// ```ignore
/// let menu = MenuBuilder::new()
///     .item("~O~pen", CM_OPEN, KB_F3)
///     .item("~S~ave", CM_SAVE, KB_F2)
///     .separator()
///     .item("E~x~it", CM_QUIT, KB_ALT_X)
///     .build();
/// ```
pub struct MenuBuilder {
    items: Vec<MenuItem>,
    help_ctx: u16,
}

impl MenuBuilder {
    /// Create a new menu builder
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            help_ctx: 0,
        }
    }

    /// Set the default help context for subsequent items
    pub fn help_context(mut self, help_ctx: u16) -> Self {
        self.help_ctx = help_ctx;
        self
    }

    /// Add a regular menu item
    pub fn item(mut self, text: &str, command: CommandId, key_code: KeyCode) -> Self {
        self.items.push(MenuItem::new(text, command, key_code, self.help_ctx));
        self
    }

    /// Add a menu item with shortcut display
    pub fn item_with_shortcut(mut self, text: &str, command: CommandId, key_code: KeyCode, shortcut: &str) -> Self {
        self.items.push(MenuItem::with_shortcut(text, command, key_code, shortcut, self.help_ctx));
        self
    }

    /// Add a disabled menu item
    pub fn item_disabled(mut self, text: &str, command: CommandId, key_code: KeyCode) -> Self {
        self.items.push(MenuItem::new_disabled(text, command, key_code, self.help_ctx));
        self
    }

    /// Add a submenu
    pub fn submenu(mut self, text: &str, key_code: KeyCode, menu: Menu) -> Self {
        self.items.push(MenuItem::submenu(text, key_code, menu, self.help_ctx));
        self
    }

    /// Add a separator
    pub fn separator(mut self) -> Self {
        self.items.push(MenuItem::separator());
        self
    }

    /// Build the menu
    pub fn build(self) -> Menu {
        Menu::from_items(self.items)
    }
}

impl Default for MenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_builder() {
        let menu = MenuBuilder::new()
            .item("~O~pen", 100, 0x3D00)
            .item("~S~ave", 101, 0x3C00)
            .separator()
            .item("E~x~it", 102, 0x2D00)
            .build();

        assert_eq!(menu.len(), 4);
        assert!(matches!(menu.items[0], MenuItem::Regular { .. }));
        assert!(matches!(menu.items[2], MenuItem::Separator));
    }

    #[test]
    fn test_accelerator() {
        let item = MenuItem::new("~O~pen", 100, 0x3D00, 0);
        assert_eq!(item.get_accelerator(), Some('o'));
    }
}
