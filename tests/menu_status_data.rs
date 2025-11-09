// (C) 2025 - Enzo Lombardi
// Example demonstrating the new Borland-compatible menu and status data structures
//
// This example shows how to use the declarative menu and status line builders
// that match Borland Turbo Vision's architecture while being Rust-idiomatic.

use turbo_vision::core::menu_data::{Menu, MenuItem, MenuBuilder};
use turbo_vision::core::status_data::{StatusItem, StatusLine, StatusLineBuilder};
use turbo_vision::core::command::*;
use turbo_vision::core::event::*;

// Define some example commands and key codes for demonstration
const CM_HELP: u16 = 1000;
const CM_HELP_INDEX: u16 = 1001;
const CM_HELP_KEYBOARD: u16 = 1002;
const CM_HELP_COMMANDS: u16 = 1003;
const CM_HELP_CONTENTS: u16 = 1004;
const CM_HELP_ABOUT: u16 = 1005;
const CM_NEXT: u16 = 1006;
const CM_DELETE_LINE: u16 = 1007;
const CM_CLEAR: u16 = 1008;

const KB_CTRL_N: KeyCode = 0x310E;
const KB_CTRL_Z: KeyCode = 0x2C1A;
const KB_CTRL_Y: KeyCode = 0x1519;
const KB_CTRL_INS: KeyCode = 0x0452;
const KB_SHIFT_INS: KeyCode = 0x0552;
const KB_SHIFT_DEL: KeyCode = 0x0553;

fn main() {
    println!("Menu & Status Data Structures Example");
    println!("======================================\n");

    // Example 1: Building a File menu using MenuBuilder (Borland-style)
    println!("1. Building a File menu with MenuBuilder:");
    let file_menu = MenuBuilder::new()
        .item_with_shortcut("~N~ew", CM_NEW, KB_CTRL_N, "Ctrl+N")
        .item_with_shortcut("~O~pen", CM_OPEN, KB_F3, "F3")
        .item_with_shortcut("~S~ave", CM_SAVE, KB_F2, "F2")
        .item("Save ~a~s...", CM_SAVE_AS, 0)
        .separator()
        .item_with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X")
        .build();

    println!("  File menu has {} items", file_menu.len());
    for (i, item) in file_menu.items.iter().enumerate() {
        match item {
            MenuItem::Regular { text, command, .. } => {
                println!("    [{}] {} -> Command {}", i, text, command);
            }
            MenuItem::SubMenu { text, .. } => {
                println!("    [{}] {} -> (submenu)", i, text);
            }
            MenuItem::Separator => {
                println!("    [{}] --------", i);
            }
        }
    }
    println!();

    // Example 2: Building menus manually (direct construction)
    println!("2. Building an Edit menu manually:");
    let edit_menu = Menu::from_items(vec![
        MenuItem::with_shortcut("~U~ndo", CM_UNDO, KB_CTRL_Z, "Ctrl+Z", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("Cu~t~", CM_CUT, KB_SHIFT_DEL, "Shift+Del", 0),
        MenuItem::with_shortcut("~C~opy", CM_COPY, KB_CTRL_INS, "Ctrl+Ins", 0),
        MenuItem::with_shortcut("~P~aste", CM_PASTE, KB_SHIFT_INS, "Shift+Ins", 0),
        MenuItem::separator(),
        MenuItem::new("~C~lear", CM_CLEAR, 0, 0),
    ]);

    println!("  Edit menu has {} items", edit_menu.len());
    println!();

    // Example 3: Building a nested menu (submenu)
    println!("3. Building a Help menu with submenu:");
    let help_topics_menu = MenuBuilder::new()
        .item("~I~ndex", CM_HELP_INDEX, 0)
        .item("~K~eyboard", CM_HELP_KEYBOARD, 0)
        .item("~C~ommands", CM_HELP_COMMANDS, 0)
        .build();

    let help_menu = Menu::from_items(vec![
        MenuItem::new("~C~ontents", CM_HELP_CONTENTS, KB_F1, 0),
        MenuItem::submenu("~T~opics", 0, help_topics_menu, 0),
        MenuItem::separator(),
        MenuItem::new("~A~bout", CM_HELP_ABOUT, 0, 0),
    ]);

    println!("  Help menu has {} items", help_menu.len());
    if let MenuItem::SubMenu { text, menu, .. } = &help_menu.items[1] {
        println!("    '{}' submenu has {} items", text, menu.len());
    }
    println!();

    // Example 4: Building a status line (simple)
    println!("4. Building a simple status line:");
    let simple_status = StatusLine::single(vec![
        StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
        StatusItem::new("~F2~ Save", KB_F2, CM_SAVE),
        StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ]);

    println!("  Status line has {} definition(s)", simple_status.defs.len());
    if let Some(def) = simple_status.defs.first() {
        println!("    Definition applies to command range {}-{}", def.min, def.max);
        println!("    Has {} items:", def.items.len());
        for item in &def.items {
            println!("      - {}", item.text);
        }
    }
    println!();

    // Example 5: Building a context-sensitive status line
    println!("5. Building a context-sensitive status line:");
    let context_status = StatusLineBuilder::new()
        // Default status (all contexts)
        .add_default_def(vec![
            StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        ])
        // Editor context (command set 100-199)
        .add_def(100, 199, vec![
            StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
            StatusItem::new("~F2~ Save", KB_F2, CM_SAVE),
            StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
            StatusItem::new("~Ctrl+Y~ Delete line", KB_CTRL_Y, CM_DELETE_LINE),
            StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
        ])
        // Dialog context (command set 200-299)
        .add_def(200, 299, vec![
            StatusItem::new("~F1~ Help", KB_F1, CM_HELP),
            StatusItem::new("~Tab~ Next", KB_TAB, CM_NEXT),
            StatusItem::new("~Esc~ Cancel", KB_ESC, CM_CANCEL),
        ])
        .build();

    println!("  Context-sensitive status line has {} definition(s)", context_status.defs.len());
    for (i, def) in context_status.defs.iter().enumerate() {
        println!("    Definition {} applies to command range {}-{}", i + 1, def.min, def.max);
        println!("      Has {} items", def.items.len());
    }
    println!();

    // Example 6: Testing context switching
    println!("6. Testing context-sensitive status line:");
    println!("  In default context (command 50):");
    if let Some(def) = context_status.get_def_for(50) {
        println!("    -> {} items", def.items.len());
    }

    println!("  In editor context (command 150):");
    if let Some(def) = context_status.get_def_for(150) {
        println!("    -> {} items", def.items.len());
    }

    println!("  In dialog context (command 250):");
    if let Some(def) = context_status.get_def_for(250) {
        println!("    -> {} items", def.items.len());
    }
    println!();

    // Example 7: Testing accelerator extraction
    println!("7. Testing accelerator key extraction:");
    let item = MenuItem::new("~O~pen File", CM_OPEN, KB_F3, 0);
    if let Some(accel) = item.get_accelerator() {
        println!("  Menu item '{}' has accelerator: '{}'", item.text(), accel);
    }

    let status_item = StatusItem::new("~F1~ Help", KB_F1, CM_HELP);
    if let Some(accel) = status_item.get_accelerator() {
        println!("  Status item '{}' has accelerator: '{}'", status_item.text, accel);
    }
    println!();

    // Example 8: Building a complete menu bar structure
    println!("8. Building a complete menu bar structure:");
    let menu_bar_menus = vec![
        ("~F~ile", MenuBuilder::new()
            .item_with_shortcut("~N~ew", CM_NEW, KB_CTRL_N, "Ctrl+N")
            .item_with_shortcut("~O~pen", CM_OPEN, KB_F3, "F3")
            .separator()
            .item_with_shortcut("E~x~it", CM_QUIT, KB_ALT_X, "Alt+X")
            .build()),
        ("~E~dit", MenuBuilder::new()
            .item_with_shortcut("~U~ndo", CM_UNDO, KB_CTRL_Z, "Ctrl+Z")
            .separator()
            .item_with_shortcut("Cu~t~", CM_CUT, KB_SHIFT_DEL, "Shift+Del")
            .item_with_shortcut("~C~opy", CM_COPY, KB_CTRL_INS, "Ctrl+Ins")
            .item_with_shortcut("~P~aste", CM_PASTE, KB_SHIFT_INS, "Shift+Ins")
            .build()),
        ("~H~elp", MenuBuilder::new()
            .item_with_shortcut("~C~ontents", CM_HELP_CONTENTS, KB_F1, "F1")
            .separator()
            .item("~A~bout", CM_HELP_ABOUT, 0)
            .build()),
    ];

    println!("  Menu bar has {} top-level menus:", menu_bar_menus.len());
    for (name, menu) in &menu_bar_menus {
        println!("    {} - {} items", name, menu.len());
    }
    println!();

    println!("✅ All menu and status data structures are working correctly!");
    println!("\nThese data structures provide:");
    println!("  • Borland-compatible API for easy porting");
    println!("  • Rust-idiomatic builder patterns");
    println!("  • Type-safe construction (no raw pointers!)");
    println!("  • Declarative menu/status definition");
    println!("  • Context-sensitive status lines");
    println!("  • Automatic accelerator key extraction");
}
