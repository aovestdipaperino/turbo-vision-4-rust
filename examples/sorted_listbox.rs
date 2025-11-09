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
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::sorted_listbox::SortedListBox;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::View;

const CMD_SEARCH_A: u16 = 100;
const CMD_SEARCH_B: u16 = 101;
const CMD_SEARCH_C: u16 = 102;
const CMD_TOGGLE_CASE: u16 = 103;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create sorted listbox with sample data
    let mut listbox = SortedListBox::new(Rect::new(5, 3, 35, 18), 1000);

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

    // Create status line with shortcuts
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F1~ Jump to A", 0, CMD_SEARCH_A),
            StatusItem::new("~F2~ Jump to B", 0, CMD_SEARCH_B),
            StatusItem::new("~F3~ Jump to C", 0, CMD_SEARCH_C),
            StatusItem::new("~F4~ Toggle Case", 0, CMD_TOGGLE_CASE),
            StatusItem::new("~F10~ Quit", 0, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Instructions
    let instructions = vec![
        "SortedListBox Demo",
        "",
        "Items are automatically sorted",
        "alphabetically (case-insensitive).",
        "",
        "Binary Search:",
        "- Press F1 to jump to 'A'",
        "- Press F2 to jump to 'B'",
        "- Press F3 to jump to 'C'",
        "",
        "Case Sensitivity:",
        "- Press F4 to toggle",
        "- Currently: Case-INSENSITIVE",
        "",
        "Navigation:",
        "- ↑/↓ to navigate",
        "- PgUp/PgDn for pages",
        "- Home/End for first/last",
    ];

    let mut case_sensitive = false;

    // Event loop
    app.running = true;
    while app.running {
        // Draw everything
        app.desktop.draw(&mut app.terminal);

        // Draw listbox
        listbox.draw(&mut app.terminal);

        // Draw instructions
        for (i, line) in instructions.iter().enumerate() {
            use turbo_vision::core::draw::DrawBuffer;
            use turbo_vision::core::palette::colors;
            use turbo_vision::views::view::write_line_to_terminal;

            // Update case sensitivity status line
            let display_line = if i == 12 && line.starts_with("- Currently:") {
                if case_sensitive {
                    "- Currently: Case-SENSITIVE"
                } else {
                    "- Currently: Case-INSENSITIVE"
                }
            } else {
                line
            };

            let mut buf = DrawBuffer::new(display_line.len());
            for (j, ch) in display_line.chars().enumerate() {
                buf.put_char(j, ch, colors::NORMAL);
            }
            write_line_to_terminal(&mut app.terminal, 40, 3 + i as i16, &buf);
        }

        // Draw selection info
        if let Some(selected) = listbox.get_selected_item() {
            use turbo_vision::core::draw::DrawBuffer;
            use turbo_vision::core::palette::colors;
            use turbo_vision::views::view::write_line_to_terminal;

            let info = format!("Selected: {}", selected);
            let mut buf = DrawBuffer::new(info.len());
            for (j, ch) in info.chars().enumerate() {
                buf.put_char(j, ch, colors::NORMAL);
            }
            write_line_to_terminal(&mut app.terminal, 5, 19, &buf);
        }

        // Draw status line
        if let Some(ref mut status_line) = app.status_line {
            status_line.draw(&mut app.terminal);
        }

        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            // Listbox handles navigation
            listbox.handle_event(&mut event);

            // Status line handles shortcuts
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Handle commands
            match event.what {
                EventType::Command => {
                    match event.command {
                        CM_QUIT => app.running = false,
                        CMD_SEARCH_A => {
                            // Find and focus on first item starting with "A"
                            if listbox.focus_prefix("A") {
                                // Success - will focus on "Apple" or "Apricot"
                            }
                        }
                        CMD_SEARCH_B => {
                            // Find and focus on first item starting with "B"
                            if listbox.focus_prefix("B") {
                                // Success - will focus on "Banana" or "Blueberry"
                            }
                        }
                        CMD_SEARCH_C => {
                            // Find and focus on first item starting with "C"
                            if listbox.focus_prefix("C") {
                                // Success - will focus on "Cherry" or "Coconut"
                            }
                        }
                        CMD_TOGGLE_CASE => {
                            // Toggle case sensitivity and re-sort
                            case_sensitive = !case_sensitive;
                            listbox.set_case_sensitive(case_sensitive);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
