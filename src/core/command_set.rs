// (C) 2025 - Enzo Lombardi
//! Command Set System
//!
//! Implements Borland Turbo Vision's command enable/disable system.
//! Commands are stored as a bitfield allowing efficient enable/disable operations.
//!
//! ## Architecture
//!
//! - **CommandSet**: Bitfield storing enabled/disabled state for up to 65,536 commands
//! - **Global State**: Stored in Application, accessible via View trait methods
//! - **Automatic Updates**: Buttons auto-enable/disable when command set changes
//! - **Broadcast Pattern**: cmCommandSetChanged notifies all views of changes
//!
//! ## Reference
//!
//! Based on Borland's implementation:
//! - `/include/tv/cmdset.h` (lines 14-84)
//! - `/classes/tcommand.cc` (lines 1-268)

use crate::core::command::CommandId;
use std::cell::RefCell;

/// Maximum number of commands supported (65,536)
/// Matches Borland: MAX_COMMANDS = 32 * 2048
pub const MAX_COMMANDS: usize = 32 * 2048;

// Global command set - matches Borland's TView::curCommandSet (tview.cc:67)
// Uses thread_local to avoid Sync requirement while maintaining global accessibility
thread_local! {
    static GLOBAL_COMMAND_SET: RefCell<CommandSet> = RefCell::new(CommandSet::with_all_enabled());
    static COMMAND_SET_CHANGED: RefCell<bool> = RefCell::new(false);
}

/// Check if a command is currently enabled (global query)
/// Matches Borland: TView::commandEnabled(ushort command) (tview.cc:142-147)
pub fn command_enabled(command: CommandId) -> bool {
    GLOBAL_COMMAND_SET.with(|cs| cs.borrow().has(command))
}

/// Enable a command in the global command set
/// Matches Borland: TView::enableCommand(ushort command) (tview.cc:384-389)
pub fn enable_command(command: CommandId) {
    GLOBAL_COMMAND_SET.with(|cs| {
        let mut set = cs.borrow_mut();
        if !set.has(command) {
            COMMAND_SET_CHANGED.with(|changed| *changed.borrow_mut() = true);
        }
        set.enable_command(command);
    });
}

/// Disable a command in the global command set
/// Matches Borland: TView::disableCommand(ushort command) (tview.cc:161-166)
pub fn disable_command(command: CommandId) {
    GLOBAL_COMMAND_SET.with(|cs| {
        let mut set = cs.borrow_mut();
        if set.has(command) {
            COMMAND_SET_CHANGED.with(|changed| *changed.borrow_mut() = true);
        }
        set.disable_command(command);
    });
}

/// Check if command set has changed (needs broadcast)
/// Matches Borland: TView::commandSetChanged (tview.cc:51)
pub fn command_set_changed() -> bool {
    COMMAND_SET_CHANGED.with(|changed| *changed.borrow())
}

/// Clear the command set changed flag
/// Called after broadcasting CM_COMMAND_SET_CHANGED
pub fn clear_command_set_changed() {
    COMMAND_SET_CHANGED.with(|changed| *changed.borrow_mut() = false);
}

/// Initialize the global command set with specific disabled commands
/// Matches Borland: initCommands() (tview.cc:58-68)
pub fn init_command_set() {
    use crate::core::command::CM_CLOSE;

    GLOBAL_COMMAND_SET.with(|cs| {
        let mut set = cs.borrow_mut();
        *set = CommandSet::with_all_enabled();
        set.disable_command(CM_CLOSE);
    });
    COMMAND_SET_CHANGED.with(|changed| *changed.borrow_mut() = false);
}

/// Number of 32-bit words needed to store command bits
const COMMANDS_COUNT: usize = MAX_COMMANDS / 32;

/// Command set bitfield for tracking enabled/disabled commands
///
/// Matches Borland's TCommandSet (cmdset.h:14-84)
/// Uses a bitfield array where each command ID is a bit position
#[derive(Clone, PartialEq)]
pub struct CommandSet {
    /// Bitfield storage: 2048 words * 32 bits = 65,536 command bits
    /// Matches Borland: uint32 *cmds
    cmds: Box<[u32; COMMANDS_COUNT]>,
}

impl CommandSet {
    /// Create a new command set with all commands disabled
    ///
    /// Matches Borland: TCommandSet::TCommandSet() (tcommand.cc:41-48)
    pub fn new() -> Self {
        Self {
            cmds: Box::new([0; COMMANDS_COUNT]),
        }
    }

    /// Create a command set with all commands enabled
    ///
    /// Matches Borland: TCommandSet::enableAllCommands() (tcommand.cc:132-137)
    pub fn with_all_enabled() -> Self {
        Self {
            cmds: Box::new([0xFFFFFFFF; COMMANDS_COUNT]),
        }
    }

    /// Check if a command is enabled
    ///
    /// Matches Borland: TCommandSet::has(int cmd) (tcommand.cc:108-112)
    pub fn has(&self, command: CommandId) -> bool {
        let cmd = command as usize;
        if cmd >= MAX_COMMANDS {
            // Commands >= MAX_COMMANDS are always enabled
            return true;
        }
        let word_index = cmd / 32;
        let bit_mask = 1u32 << (cmd & 0x1F);
        (self.cmds[word_index] & bit_mask) != 0
    }

    /// Enable a single command
    ///
    /// Matches Borland: TCommandSet::enableCmd(int cmd) (tcommand.cc:139-145)
    pub fn enable_command(&mut self, command: CommandId) {
        let cmd = command as usize;
        if cmd >= MAX_COMMANDS {
            return;
        }
        let word_index = cmd / 32;
        let bit_mask = 1u32 << (cmd & 0x1F);
        self.cmds[word_index] |= bit_mask;
    }

    /// Disable a single command
    ///
    /// Matches Borland: TCommandSet::disableCmd(int cmd) (tcommand.cc:180-186)
    pub fn disable_command(&mut self, command: CommandId) {
        let cmd = command as usize;
        if cmd >= MAX_COMMANDS {
            return;
        }
        let word_index = cmd / 32;
        let bit_mask = 1u32 << (cmd & 0x1F);
        self.cmds[word_index] &= !bit_mask;
    }

    /// Enable a range of commands (inclusive)
    ///
    /// Matches Borland: TCommandSet::enableCmd(int cmdStart, int cmdEnd) (tcommand.cc:147-179)
    pub fn enable_range(&mut self, cmd_start: CommandId, cmd_end: CommandId) {
        let start = cmd_start as usize;
        let end = cmd_end as usize;

        if end >= MAX_COMMANDS || end <= start {
            return;
        }

        let word_start = start / 32;
        let word_end = end / 32;

        // Both in the same word
        if word_start == word_end {
            for bit in (start & 0x1F)..=(end & 0x1F) {
                self.cmds[word_start] |= 1u32 << bit;
            }
            return;
        }

        // Set partial bits in first word
        for bit in (start & 0x1F)..32 {
            self.cmds[word_start] |= 1u32 << bit;
        }

        // Set all bits in middle words
        for word in (word_start + 1)..word_end {
            self.cmds[word] = 0xFFFFFFFF;
        }

        // Set partial bits in last word
        for bit in 0..=(end & 0x1F) {
            self.cmds[word_end] |= 1u32 << bit;
        }
    }

    /// Disable a range of commands (inclusive)
    ///
    /// Matches Borland: TCommandSet::disableCmd(int cmdStart, int cmdEnd) (tcommand.cc:188-220)
    pub fn disable_range(&mut self, cmd_start: CommandId, cmd_end: CommandId) {
        let start = cmd_start as usize;
        let end = cmd_end as usize;

        if end >= MAX_COMMANDS || end <= start {
            return;
        }

        let word_start = start / 32;
        let word_end = end / 32;

        // Both in the same word
        if word_start == word_end {
            for bit in (start & 0x1F)..=(end & 0x1F) {
                self.cmds[word_start] &= !(1u32 << bit);
            }
            return;
        }

        // Clear partial bits in first word
        for bit in (start & 0x1F)..32 {
            self.cmds[word_start] &= !(1u32 << bit);
        }

        // Clear all bits in middle words
        for word in (word_start + 1)..word_end {
            self.cmds[word] = 0;
        }

        // Clear partial bits in last word
        for bit in 0..=(end & 0x1F) {
            self.cmds[word_end] &= !(1u32 << bit);
        }
    }

    /// Enable all commands in another command set
    ///
    /// Matches Borland: TCommandSet::enableCmd(const TCommandSet&) (tcommand.cc:222-228)
    pub fn enable_set(&mut self, other: &CommandSet) {
        for i in 0..COMMANDS_COUNT {
            self.cmds[i] |= other.cmds[i];
        }
    }

    /// Disable all commands in another command set
    ///
    /// Matches Borland: TCommandSet::disableCmd(const TCommandSet&) (tcommand.cc:230-236)
    pub fn disable_set(&mut self, other: &CommandSet) {
        for i in 0..COMMANDS_COUNT {
            self.cmds[i] &= !other.cmds[i];
        }
    }

    /// Enable all commands
    ///
    /// Matches Borland: TCommandSet::enableAllCommands() (tcommand.cc:132-137)
    pub fn enable_all(&mut self) {
        self.cmds.fill(0xFFFFFFFF);
    }

    /// Check if command set is empty (all commands disabled)
    ///
    /// Matches Borland: TCommandSet::isEmpty() (tcommand.cc:114-125)
    pub fn is_empty(&self) -> bool {
        self.cmds.iter().all(|&word| word == 0)
    }

    /// Perform bitwise AND with another command set
    ///
    /// Matches Borland: TCommandSet::operator&=(const TCommandSet&) (tcommand.cc:259-268)
    pub fn intersect(&mut self, other: &CommandSet) {
        for i in 0..COMMANDS_COUNT {
            self.cmds[i] &= other.cmds[i];
        }
    }

    /// Perform bitwise OR with another command set
    ///
    /// Matches Borland: TCommandSet::operator|=(const TCommandSet&) (tcommand.cc:249-257)
    pub fn union(&mut self, other: &CommandSet) {
        for i in 0..COMMANDS_COUNT {
            self.cmds[i] |= other.cmds[i];
        }
    }
}

impl Default for CommandSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_disable_single() {
        let mut cs = CommandSet::new();
        assert!(!cs.has(10));

        cs.enable_command(10);
        assert!(cs.has(10));

        cs.disable_command(10);
        assert!(!cs.has(10));
    }

    #[test]
    fn test_enable_range() {
        let mut cs = CommandSet::new();
        cs.enable_range(10, 20);

        assert!(!cs.has(9));
        assert!(cs.has(10));
        assert!(cs.has(15));
        assert!(cs.has(20));
        assert!(!cs.has(21));
    }

    #[test]
    fn test_enable_all() {
        let mut cs = CommandSet::new();
        cs.enable_all();

        assert!(cs.has(0));
        assert!(cs.has(100));
        assert!(cs.has(65535));
    }

    #[test]
    fn test_is_empty() {
        let mut cs = CommandSet::new();
        assert!(cs.is_empty());

        cs.enable_command(50);
        assert!(!cs.is_empty());
    }

    #[test]
    fn test_commands_default_disabled() {
        // New command set has all commands disabled by default
        let cs = CommandSet::new();
        assert!(!cs.has(0));
        assert!(!cs.has(100));
        assert!(!cs.has(1000));
        assert!(!cs.has(60000));
        assert!(!cs.has(65535)); // Maximum u16 value
    }
}
