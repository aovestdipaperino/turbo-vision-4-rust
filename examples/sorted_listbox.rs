// (C) 2025 - Enzo Lombardi
// Demonstration of SortedListBox with binary search
//
// This example shows:
// - Automatic sorting of items
// - Binary search with find_exact()
// - Prefix search with find_prefix() and focus_prefix()
// - Case-sensitive vs case-insensitive sorting

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{EventType, KB_ALT_A, KB_ALT_B, KB_ALT_C, KB_ALT_T, KB_ALT_X};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Attr;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::View;
use turbo_vision::views::sorted_listbox::SortedListBox;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::view::write_line_to_terminal;

const CMD_SEARCH_A: u16 = 100;
const CMD_SEARCH_B: u16 = 101;
const CMD_SEARCH_C: u16 = 102;
const CMD_TOGGLE_CASE: u16 = 103;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let status_line = setup_status_line(&app);
    app.set_status_line(status_line);

    let mut listbox = setup_listbox();
    let instructions = get_instructions();
    let mut case_sensitive = false;

    // Event loop
    app.running = true;
    while app.running {
        draw_screen(&mut app, &mut listbox, &instructions, case_sensitive);

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            handle_event(&mut app, &mut listbox, &mut event, &mut case_sensitive);
        }
    }

    Ok(())
}

/// Create and configure the status line at the bottom of the screen
fn setup_status_line(app: &Application) -> StatusLine {
    let (w, h) = app.terminal.size();

    StatusLine::new(
        Rect::new(0, h - 1, w, h),
        vec![
            StatusItem::new("~Alt+X~ Quit", KB_ALT_X, CM_QUIT),
            StatusItem::new("~Alt+A~ Jump to A", KB_ALT_A, CMD_SEARCH_A),
            StatusItem::new("~Alt+B~ Jump to B", KB_ALT_B, CMD_SEARCH_B),
            StatusItem::new("~Alt+C~ Jump to C", KB_ALT_C, CMD_SEARCH_C),
            StatusItem::new("~Alt+T~ Toggle Case", KB_ALT_T, CMD_TOGGLE_CASE),
        ],
    )
}

/// Create and populate the sorted listbox with sample data
fn setup_listbox() -> SortedListBox {
    use turbo_vision::views::sorted_listbox::SortedListBoxBuilder;

    let mut listbox = SortedListBoxBuilder::new().bounds(Rect::new(5, 3, 35, 18)).on_select_command(1000).build();

    // Add items in random order - they'll be automatically sorted
    listbox.add_item("Zebra".to_string());
    listbox.add_item("Apple".to_string());
    listbox.add_item("Banana".to_string());
    listbox.add_item("Apricot".to_string());
    listbox.add_item("Mango".to_string());
    listbox.add_item("Cherry".to_string());
    listbox.add_item("Blueberry".to_string());
    listbox.add_item("Avocado".to_string());
    listbox.add_item("Coconut".to_string());
    listbox.add_item("Date".to_string());
    listbox.add_item("Elderberry".to_string());
    listbox.add_item("Fig".to_string());
    listbox.add_item("Grape".to_string());
    listbox.add_item("Kiwi".to_string());

    listbox
}

/// Get the instruction text to display
fn get_instructions() -> Vec<&'static str> {
    vec![
        "SortedListBox Demo",
        "",
        "Items are automatically sorted",
        "alphabetically (case-insensitive).",
        "",
        "Binary Search:",
        "- Press Alt+A to jump to 'A'",
        "- Press Alt+B to jump to 'B'",
        "- Press Alt+C to jump to 'C'",
        "",
        "Case Sensitivity:",
        "- Press Alt+T to toggle",
        "- Currently: Case-INSENSITIVE",
        "",
        "Navigation:",
        "- ↑/↓ to navigate",
        "- PgUp/PgDn for pages",
        "- Home/End for first/last",
    ]
}

/// Draw all screen elements
fn draw_screen(app: &mut Application, listbox: &mut SortedListBox, instructions: &[&str], case_sensitive: bool) {
    // Draw base elements
    app.desktop.draw(&mut app.terminal);
    listbox.draw(&mut app.terminal);

    // Draw instructions panel
    draw_instructions_panel(&mut app.terminal, instructions, case_sensitive);

    // Draw selection info
    draw_selection_info(&mut app.terminal, listbox);

    // Draw status line
    if let Some(ref mut status_line) = app.status_line {
        status_line.draw(&mut app.terminal);
    }

    let _ = app.terminal.flush();
}

/// Draw the instructions panel with black background and white text
fn draw_instructions_panel(terminal: &mut Terminal, instructions: &[&str], case_sensitive: bool) {
    const INSTR_WIDTH: usize = 40;
    const INSTR_X: i16 = 40;
    const INSTR_Y: i16 = 3;

    for (i, line) in instructions.iter().enumerate() {
        // Update case sensitivity status line dynamically
        let display_line = if i == 12 && line.starts_with("- Currently:") {
            if case_sensitive { "- Currently: Case-SENSITIVE" } else { "- Currently: Case-INSENSITIVE" }
        } else {
            line
        };

        let mut buf = DrawBuffer::new(INSTR_WIDTH);
        let chars: Vec<char> = display_line.chars().collect();
        for j in 0..INSTR_WIDTH {
            let ch = if j < chars.len() { chars[j] } else { ' ' };
            buf.put_char(j, ch, Attr::from_u8(0x0F)); // White on black
        }
        write_line_to_terminal(terminal, INSTR_X, INSTR_Y + i as i16, &buf);
    }
}

/// Draw the selection info with cyan background and white text
fn draw_selection_info(terminal: &mut Terminal, listbox: &SortedListBox) {
    const LISTBOX_WIDTH: usize = 30; // Width of the listbox (35 - 5 = 30)
    const SELECTION_X: i16 = 5;
    const SELECTION_Y: i16 = 20;

    if let Some(selected) = listbox.get_selected_item() {
        let info = format!("Selected: {}", selected);
        let mut buf = DrawBuffer::new(LISTBOX_WIDTH);
        let chars: Vec<char> = info.chars().collect();

        for j in 0..LISTBOX_WIDTH {
            let ch = if j < chars.len() { chars[j] } else { ' ' };
            buf.put_char(j, ch, Attr::from_u8(0x3F)); // White on cyan
        }
        write_line_to_terminal(terminal, SELECTION_X, SELECTION_Y, &buf);
    }
}

/// Handle all events (keyboard, commands, etc.)
fn handle_event(app: &mut Application, listbox: &mut SortedListBox, event: &mut turbo_vision::core::event::Event, case_sensitive: &mut bool) {
    // Listbox handles navigation
    listbox.handle_event(event);

    // Status line handles shortcuts
    if let Some(ref mut status_line) = app.status_line {
        status_line.handle_event(event);
    }

    // Handle commands
    if event.what == EventType::Command {
        match event.command {
            CM_QUIT => app.running = false,
            CMD_SEARCH_A => {
                listbox.focus_prefix("A");
            }
            CMD_SEARCH_B => {
                listbox.focus_prefix("B");
            }
            CMD_SEARCH_C => {
                listbox.focus_prefix("C");
            }
            CMD_TOGGLE_CASE => {
                *case_sensitive = !*case_sensitive;
                listbox.set_case_sensitive(*case_sensitive);
            }
            _ => {}
        }
    }
}
