// (C) 2025 - Enzo Lombardi
// Example demonstrating the new Borland-compatible menu and status data structures
//
// This example shows how to use the declarative menu and status line builders
// that match Borland Turbo Vision's architecture while being Rust-idiomatic.

use turbo_vision::core::command::*;
use turbo_vision::core::event::*;
use turbo_vision::core::menu_data::MenuItemBuilder;
use turbo_vision::core::menu_data::{Menu, MenuBuilder, MenuItem};
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::core::status_data::{StatusLine, StatusLineBuilder};

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

const KB_CTRL_INS: KeyCode = 0x0452;
const KB_SHIFT_INS: KeyCode = 0x0552;
const KB_SHIFT_DEL: KeyCode = 0x0553;

fn main() {
    println!("Menu & Status Data Structures Example");
    println!("======================================\n");

    // Example 1: Building a File menu using MenuBuilder (Borland-style)
    println!("1. Building a File menu with MenuBuilder:");
    let file_menu = MenuBuilder::new()
        .item_key("~N~ew", CM_NEW, "Ctrl+N")
        .item_key("~O~pen", CM_OPEN, "F3")
        .item_key("~S~ave", CM_SAVE, "F2")
        .item("Save ~a~s...", CM_SAVE_AS)
        .separator()
        .item_key("E~x~it", CM_QUIT, "Alt+X")
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
        MenuItemBuilder::new()
            .text("~U~ndo")
            .command(CM_UNDO)
            .key("Ctrl+Z")
            .build(),
        MenuItem::separator(),
        MenuItemBuilder::new()
            .text("Cu~t~")
            .command(CM_CUT)
            .key_code(KB_SHIFT_DEL)
            .shortcut("Shift+Del")
            .build(),
        MenuItemBuilder::new()
            .text("~C~opy")
            .command(CM_COPY)
            .key_code(KB_CTRL_INS)
            .shortcut("Ctrl+Ins")
            .build(),
        MenuItemBuilder::new()
            .text("~P~aste")
            .command(CM_PASTE)
            .key_code(KB_SHIFT_INS)
            .shortcut("Shift+Ins")
            .build(),
        MenuItem::separator(),
        MenuItemBuilder::new()
            .text("~C~lear")
            .command(CM_CLEAR)
            .build(),
    ]);

    println!("  Edit menu has {} items", edit_menu.len());
    println!();

    // Example 3: Building a nested menu (submenu)
    println!("3. Building a Help menu with submenu:");
    let help_topics_menu = MenuBuilder::new()
        .item("~I~ndex", CM_HELP_INDEX)
        .item("~K~eyboard", CM_HELP_KEYBOARD)
        .item("~C~ommands", CM_HELP_COMMANDS)
        .build();

    let help_menu = Menu::from_items(vec![
        MenuItemBuilder::new()
            .text("~C~ontents")
            .command(CM_HELP_CONTENTS)
            .key_code(KB_F1)
            .build(),
        MenuItem::submenu("~T~opics", 0, help_topics_menu, 0),
        MenuItem::separator(),
        MenuItemBuilder::new()
            .text("~A~bout")
            .command(CM_HELP_ABOUT)
            .build(),
    ]);

    println!("  Help menu has {} items", help_menu.len());
    if let MenuItem::SubMenu { text, menu, .. } = &help_menu.items[1] {
        println!("    '{}' submenu has {} items", text, menu.len());
    }
    println!();

    // Example 4: Building a status line (simple)
    println!("4. Building a simple status line:");
    let simple_status = StatusLine::single(vec![
        StatusItemBuilder::new()
            .text("~F1~ Help")
            .key("F1")
            .command(CM_HELP)
            .build(),
        StatusItemBuilder::new()
            .text("~F2~ Save")
            .key("F2")
            .command(CM_SAVE)
            .build(),
        StatusItemBuilder::new()
            .text("~F3~ Open")
            .key("F3")
            .command(CM_OPEN)
            .build(),
        StatusItemBuilder::new()
            .text("~Alt+X~ Exit")
            .key("Alt+X")
            .command(CM_QUIT)
            .build(),
    ]);

    println!(
        "  Status line has {} definition(s)",
        simple_status.defs.len()
    );
    if let Some(def) = simple_status.defs.first() {
        println!(
            "    Definition applies to command range {}-{}",
            def.min, def.max
        );
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
            StatusItemBuilder::new()
                .text("~F1~ Help")
                .key("F1")
                .command(CM_HELP)
                .build(),
            StatusItemBuilder::new()
                .text("~Alt+X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
        ])
        // Editor context (command set 100-199)
        .add_def(
            100,
            199,
            vec![
                StatusItemBuilder::new()
                    .text("~F1~ Help")
                    .key("F1")
                    .command(CM_HELP)
                    .build(),
                StatusItemBuilder::new()
                    .text("~F2~ Save")
                    .key("F2")
                    .command(CM_SAVE)
                    .build(),
                StatusItemBuilder::new()
                    .text("~F3~ Open")
                    .key("F3")
                    .command(CM_OPEN)
                    .build(),
                StatusItemBuilder::new()
                    .text("~Ctrl+Y~ Delete line")
                    .key("Ctrl+Y")
                    .command(CM_DELETE_LINE)
                    .build(),
                StatusItemBuilder::new()
                    .text("~Alt+X~ Exit")
                    .key("Alt+X")
                    .command(CM_QUIT)
                    .build(),
            ],
        )
        // Dialog context (command set 200-299)
        .add_def(
            200,
            299,
            vec![
                StatusItemBuilder::new()
                    .text("~F1~ Help")
                    .key("F1")
                    .command(CM_HELP)
                    .build(),
                StatusItemBuilder::new()
                    .text("~Tab~ Next")
                    .key("Tab")
                    .command(CM_NEXT)
                    .build(),
                StatusItemBuilder::new()
                    .text("~Esc~ Cancel")
                    .key("Esc")
                    .command(CM_CANCEL)
                    .build(),
            ],
        )
        .build();

    println!(
        "  Context-sensitive status line has {} definition(s)",
        context_status.defs.len()
    );
    for (i, def) in context_status.defs.iter().enumerate() {
        println!(
            "    Definition {} applies to command range {}-{}",
            i + 1,
            def.min,
            def.max
        );
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
    let item = MenuItemBuilder::new()
        .text("~O~pen File")
        .command(CM_OPEN)
        .key_code(KB_F3)
        .build();
    if let Some(accel) = item.get_accelerator() {
        println!("  Menu item '{}' has accelerator: '{}'", item.text(), accel);
    }

    let status_item = StatusItemBuilder::new()
        .text("~F1~ Help")
        .key("F1")
        .command(CM_HELP)
        .build();
    if let Some(accel) = status_item.get_accelerator() {
        println!(
            "  Status item '{}' has accelerator: '{}'",
            status_item.text, accel
        );
    }
    println!();

    // Example 8: Building a complete menu bar structure
    println!("8. Building a complete menu bar structure:");
    let menu_bar_menus = vec![
        (
            "~F~ile",
            MenuBuilder::new()
                .item_key("~N~ew", CM_NEW, "Ctrl+N")
                .item_key("~O~pen", CM_OPEN, "F3")
                .separator()
                .item_key("E~x~it", CM_QUIT, "Alt+X")
                .build(),
        ),
        (
            "~E~dit",
            MenuBuilder::new()
                .item_key("~U~ndo", CM_UNDO, "Ctrl+Z")
                .separator()
                .add(
                    MenuItemBuilder::new()
                        .text("Cu~t~")
                        .command(CM_CUT)
                        .key_code(KB_SHIFT_DEL)
                        .shortcut("Shift+Del")
                        .build(),
                )
                .add(
                    MenuItemBuilder::new()
                        .text("~C~opy")
                        .command(CM_COPY)
                        .key_code(KB_CTRL_INS)
                        .shortcut("Ctrl+Ins")
                        .build(),
                )
                .add(
                    MenuItemBuilder::new()
                        .text("~P~aste")
                        .command(CM_PASTE)
                        .key_code(KB_SHIFT_INS)
                        .shortcut("Shift+Ins")
                        .build(),
                )
                .build(),
        ),
        (
            "~H~elp",
            MenuBuilder::new()
                .item_key("~C~ontents", CM_HELP_CONTENTS, "F1")
                .separator()
                .item("~A~bout", CM_HELP_ABOUT)
                .build(),
        ),
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
