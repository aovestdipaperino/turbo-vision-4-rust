// (C) 2025 - Enzo Lombardi
// Borland-compatible status line data structures
// Matches the original Turbo Vision TStatusItem, TStatusDef architecture
//
// This module provides data structures for building status lines in a declarative way,
// matching Borland's approach while being Rust-idiomatic.

use crate::core::command::CommandId;
use crate::core::event::KeyCode;

/// Status line item - displays text and responds to keyboard shortcuts
///
/// Matches Borland: TStatusItem
///
/// In Borland, TStatusItem is a linked list node with a `next` pointer.
/// In Rust, we use Vec for type safety, but provide builder methods
/// that match Borland's ergonomics.
#[derive(Clone, Debug)]
pub struct StatusItem {
    /// Display text (use ~x~ to mark accelerator key)
    pub text: String,
    /// Keyboard shortcut
    pub key_code: KeyCode,
    /// Command to execute when clicked or shortcut pressed
    pub command: CommandId,
}

impl StatusItem {
    /// Create a new status item
    ///
    /// Matches Borland: `TStatusItem(text, keyCode, command)`
    ///
    /// # Example
    /// ```ignore
    /// let item = StatusItem::new("~F1~ Help", KB_F1, CM_HELP);
    /// ```
    pub fn new(text: &str, key_code: KeyCode, command: CommandId) -> Self {
        Self {
            text: text.to_string(),
            key_code,
            command,
        }
    }

    /// Extract the accelerator key from the text (character between ~ marks)
    pub fn get_accelerator(&self) -> Option<char> {
        let mut chars = self.text.chars();
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
}

/// Status line definition - defines which items are visible for a command set range
///
/// Matches Borland: TStatusDef
///
/// In Borland, TStatusDef is a linked list node with:
/// - `min`, `max`: command range for which this def is active
/// - `items`: pointer to first item in linked list
/// - `next`: pointer to next status def
///
/// In Rust, we use Vec for type safety and provide convenient builders.
#[derive(Clone, Debug)]
pub struct StatusDef {
    /// Minimum command ID for which this definition is active
    pub min: u16,
    /// Maximum command ID for which this definition is active
    pub max: u16,
    /// Status items to display
    pub items: Vec<StatusItem>,
}

impl StatusDef {
    /// Create a new status definition
    ///
    /// Matches Borland: `TStatusDef(min, max, items)`
    ///
    /// # Example
    /// ```ignore
    /// let def = StatusDef::new(0, 0xFFFF, vec![
    ///     StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
    ///     StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    /// ]);
    /// ```
    pub fn new(min: u16, max: u16, items: Vec<StatusItem>) -> Self {
        Self { min, max, items }
    }

    /// Create a status definition for all command ranges (default)
    ///
    /// Matches Borland: `TStatusDef(0, 0xFFFF, items)`
    pub fn default_range(items: Vec<StatusItem>) -> Self {
        Self {
            min: 0,
            max: 0xFFFF,
            items,
        }
    }

    /// Check if this definition applies to the given command set
    ///
    /// In Borland, the status line checks which definition's range
    /// matches the currently focused view's command set.
    pub fn applies_to(&self, command_set: u16) -> bool {
        command_set >= self.min && command_set <= self.max
    }

    /// Add an item to this definition
    pub fn add(&mut self, item: StatusItem) {
        self.items.push(item);
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if definition has no items
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Status line configuration - collection of status definitions
///
/// Matches Borland: linked list of TStatusDef
///
/// In Borland, status defs are linked via `next` pointers.
/// In Rust, we use Vec for type safety.
#[derive(Clone, Debug)]
pub struct StatusLine {
    /// Status definitions (evaluated in order)
    pub defs: Vec<StatusDef>,
}

impl StatusLine {
    /// Create a new status line configuration
    pub fn new(defs: Vec<StatusDef>) -> Self {
        Self { defs }
    }

    /// Create a status line with a single default definition
    pub fn single(items: Vec<StatusItem>) -> Self {
        Self {
            defs: vec![StatusDef::default_range(items)],
        }
    }

    /// Get the status definition that applies to the given command set
    ///
    /// Matches Borland: TStatusLine::update() logic
    pub fn get_def_for(&self, command_set: u16) -> Option<&StatusDef> {
        // Return first matching definition (Borland behavior)
        self.defs.iter().find(|def| def.applies_to(command_set))
    }

    /// Add a status definition
    pub fn add_def(&mut self, def: StatusDef) {
        self.defs.push(def);
    }
}

/// Builder for constructing status line configurations fluently
///
/// This provides a Borland-style builder API while being Rust-idiomatic.
///
/// # Example (Borland-style)
/// ```ignore
/// let status = StatusLineBuilder::new()
///     .add_def(0, 0xFFFF, vec![
///         StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
///         StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
///     ])
///     .build();
/// ```
pub struct StatusLineBuilder {
    defs: Vec<StatusDef>,
}

impl StatusLineBuilder {
    /// Create a new status line builder
    pub fn new() -> Self {
        Self {
            defs: Vec::new(),
        }
    }

    /// Add a status definition with command range
    ///
    /// Matches Borland: `TStatusDef(min, max, items)`
    pub fn add_def(mut self, min: u16, max: u16, items: Vec<StatusItem>) -> Self {
        self.defs.push(StatusDef::new(min, max, items));
        self
    }

    /// Add a default status definition (applies to all command sets)
    pub fn add_default_def(mut self, items: Vec<StatusItem>) -> Self {
        self.defs.push(StatusDef::default_range(items));
        self
    }

    /// Build the status line configuration
    pub fn build(self) -> StatusLine {
        StatusLine::new(self.defs)
    }
}

impl Default for StatusLineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_def() {
        let def = StatusDef::new(0, 100, vec![
            StatusItem::new("~F1~ Help", 0x3B00, 100),
            StatusItem::new("~Alt+X~ Exit", 0x2D00, 101),
        ]);

        assert!(def.applies_to(50));
        assert!(def.applies_to(0));
        assert!(def.applies_to(100));
        assert!(!def.applies_to(101));
        assert_eq!(def.len(), 2);
    }

    #[test]
    fn test_status_line_builder() {
        let status = StatusLineBuilder::new()
            .add_def(0, 100, vec![
                StatusItem::new("~F1~ Help", 0x3B00, 100),
                StatusItem::new("~F2~ Save", 0x3C00, 101),
            ])
            .add_def(101, 200, vec![
                StatusItem::new("~F1~ Help", 0x3B00, 100),
                StatusItem::new("~Esc~ Cancel", 0x011B, 102),
            ])
            .build();

        assert_eq!(status.defs.len(), 2);

        let def1 = status.get_def_for(50);
        assert!(def1.is_some());
        assert_eq!(def1.unwrap().len(), 2);

        let def2 = status.get_def_for(150);
        assert!(def2.is_some());
        assert_eq!(def2.unwrap().len(), 2);
    }

    #[test]
    fn test_accelerator() {
        let item = StatusItem::new("~F1~ Help", 0x3B00, 100);
        assert_eq!(item.get_accelerator(), Some('f'));
    }
}
