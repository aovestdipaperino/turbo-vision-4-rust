// (C) 2025 - Enzo Lombardi

//! Menu data structures - declarative menu building with Borland-compatible API.
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

/// Builder for creating regular menu items with a fluent API.
///
/// This builder focuses on the Regular variant of MenuItem where most
/// configuration options exist (enabled state, shortcut display).
///
/// # Examples
///
/// ```ignore
/// use turbo_vision::core::menu_data::MenuItemBuilder;
/// use turbo_vision::core::command::CM_OPEN;
/// use turbo_vision::core::event::KB_F3;
///
/// // Create a simple menu item
/// let item = MenuItemBuilder::new()
///     .text("~O~pen")
///     .command(CM_OPEN)
///     .key_code(KB_F3)
///     .build();
///
/// // Create a menu item with shortcut display
/// let item = MenuItemBuilder::new()
///     .text("~S~ave")
///     .command(CM_SAVE)
///     .key_code(KB_F2)
///     .shortcut("F2")
///     .build();
///
/// // Create a disabled menu item
/// let item = MenuItemBuilder::new()
///     .text("~P~rint")
///     .command(CM_PRINT)
///     .key_code(KB_CTRL_P)
///     .enabled(false)
///     .build();
/// ```
pub struct MenuItemBuilder {
    text: Option<String>,
    command: Option<CommandId>,
    key_code: KeyCode,
    help_ctx: u16,
    enabled: bool,
    shortcut: Option<String>,
}

impl MenuItemBuilder {
    /// Creates a new MenuItemBuilder with default values.
    pub fn new() -> Self {
        Self {
            text: None,
            command: None,
            key_code: 0,
            help_ctx: 0,
            enabled: true,
            shortcut: None,
        }
    }

    /// Sets the menu item text (required).
    /// Use ~x~ to mark the accelerator key.
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets the command to execute (required).
    #[must_use]
    pub fn command(mut self, command: CommandId) -> Self {
        self.command = Some(command);
        self
    }

    /// Sets the keyboard shortcut key code.
    #[must_use]
    pub fn key_code(mut self, key_code: KeyCode) -> Self {
        self.key_code = key_code;
        self
    }

    /// Sets the help context ID.
    #[must_use]
    pub fn help_ctx(mut self, help_ctx: u16) -> Self {
        self.help_ctx = help_ctx;
        self
    }

    /// Sets whether the menu item is enabled (default: true).
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the shortcut display text (e.g., "F3", "Ctrl+O").
    #[must_use]
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Builds the MenuItem::Regular variant.
    ///
    /// # Panics
    ///
    /// Panics if required fields (text, command) are not set.
    pub fn build(self) -> MenuItem {
        let text = self.text.expect("MenuItem text must be set");
        let command = self.command.expect("MenuItem command must be set");

        MenuItem::Regular {
            text,
            command,
            key_code: self.key_code,
            help_ctx: self.help_ctx,
            enabled: self.enabled,
            shortcut: self.shortcut,
        }
    }
}

impl Default for MenuItemBuilder {
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

    #[test]
    fn test_menu_item_builder() {
        let item = MenuItemBuilder::new()
            .text("~O~pen")
            .command(100)
            .key_code(0x3D00)
            .build();

        assert_eq!(item.text(), "~O~pen");
        assert_eq!(item.command(), Some(100));
        assert!(item.is_selectable());
    }

    #[test]
    fn test_menu_item_builder_with_shortcut() {
        let item = MenuItemBuilder::new()
            .text("~S~ave")
            .command(101)
            .key_code(0x3C00)
            .shortcut("F2")
            .build();

        assert_eq!(item.shortcut(), Some("F2"));
    }

    #[test]
    fn test_menu_item_builder_disabled() {
        let item = MenuItemBuilder::new()
            .text("~P~rint")
            .command(102)
            .key_code(0)
            .enabled(false)
            .build();

        assert!(!item.is_selectable());
    }
}
