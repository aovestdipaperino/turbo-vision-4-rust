// (C) 2025 - Enzo Lombardi
// File Browser Example
//
// Demonstrates:
// - FileList for browsing files
// - DirListBox for directory tree navigation
// - Side-by-side directory tree and file list

use std::env;
use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::event::{EventType, KB_ESC_ESC, KB_TAB};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::dir_listbox::DirListBox;
use turbo_vision::views::file_list::FileList;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::view::View;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (w, h) = app.terminal.size();
    let (w, h) = (w as i16, h as i16);
    let current_dir = env::current_dir()?;

    // Split screen: DirListBox on left, FileList on right
    let split_x = w / 2;

    // Directory tree on left
    let dir_bounds = Rect::new(0, 0, split_x, h - 1);
    let mut dir_list = DirListBox::new(dir_bounds, &current_dir);
    dir_list.set_focus(true);

    // File list on right
    let file_bounds = Rect::new(split_x, 0, w, h - 1);
    let mut file_list = FileList::new(file_bounds, &current_dir);
    file_list.refresh();

    // Kind of status line at bottom
    let status_bounds = Rect::new(0, h - 1, w, h);
    let mut status = StaticTextBuilder::new()
        .bounds(status_bounds)
        .text(" File Browser Demo | TAB: Switch panels | Enter: Navigate | ESC ESC: Exit")
        .build();

    // Event loop
    let mut focused_left = true;

    loop {
        // Draw everything
        dir_list.draw(&mut app.terminal);
        file_list.draw(&mut app.terminal);
        status.draw(&mut app.terminal);

        // Update cursor
        if focused_left {
            dir_list.update_cursor(&mut app.terminal);
        } else {
            file_list.update_cursor(&mut app.terminal);
        }

        let _ = app.terminal.flush();

        // Handle events
        if let Ok(Some(mut event)) = app.terminal.poll_event(Duration::from_millis(50)) {
            // Handle TAB to switch focus
            if event.what == EventType::Keyboard && event.key_code == KB_TAB {
                focused_left = !focused_left;
                dir_list.set_focus(focused_left);
                file_list.set_focus(!focused_left);
                event.clear();
            }

            // Handle ESC ESC to exit
            if event.what == EventType::Keyboard && event.key_code == KB_ESC_ESC {
                break;
            }

            // Let focused panel handle the event
            if focused_left {
                dir_list.handle_event(&mut event);

                // Sync file list with directory list
                if dir_list.current_path() != file_list.current_path() {
                    let _ = file_list.change_dir(dir_list.current_path());
                }
            } else {
                file_list.handle_event(&mut event);

                // Sync directory list with file list (if directory changed)
                if file_list.current_path() != dir_list.current_path() {
                    let _ = dir_list.change_dir(file_list.current_path());
                }
            }
        }
    }

    Ok(())
}
