// (C) 2025 - Enzo Lombardi
// Example demonstrating Group::broadcast() with owner parameter
//
// This example creates a standalone Group (not boxed in a Dialog) to directly
// demonstrate the broadcast() method with owner parameter.
//
// Visual demonstration:
// - 4 buttons in a group showing "Broadcasts received" counters
// - Click any button: ALL siblings increment their counter EXCEPT the clicked one
// - This proves Group::broadcast() skips the owner correctly

use std::cell::Cell;
use turbo_vision::app::Application;
use turbo_vision::core::command::CommandId;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{Event, EventType, KB_ALT_X, KB_CTRL_C, KB_ESC_ESC};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::colors;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::View;
use turbo_vision::views::group::Group;

// Custom commands
const CMD_BROADCAST_TEST: CommandId = 200;
const CMD_BUTTON_BASE: CommandId = 201;

/// Custom button that tracks broadcasts received
struct BroadcastButton {
    bounds: Rect,
    label: String,
    command: CommandId,
    broadcast_count: Cell<u32>,
    click_count: Cell<u32>,
}

impl BroadcastButton {
    fn new(bounds: Rect, label: &str, command: CommandId) -> Self {
        Self {
            bounds,
            label: label.to_string(),
            command,
            broadcast_count: Cell::new(0),
            click_count: Cell::new(0),
        }
    }
}

impl View for BroadcastButton {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width_clamped() as usize;
        let height = self.bounds.height();

        for y in 0..height {
            let mut buf = DrawBuffer::new(width);

            if y == 0 {
                // Label
                let text = format!("{:^width$}", self.label, width = width);
                buf.move_str(0, &text, colors::BUTTON_NORMAL);
            } else if y == 1 {
                // Click count
                let text = format!("{:^width$}", format!("Clicks: {}", self.click_count.get()), width = width);
                buf.move_str(0, &text, colors::DIALOG_NORMAL);
            } else if y == 2 {
                // Broadcast count (this is what we're testing!)
                let text = format!("{:^width$}", format!("RX: {}", self.broadcast_count.get()), width = width);
                buf.move_str(0, &text, colors::MENU_NORMAL);
            }

            turbo_vision::views::view::write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        use turbo_vision::core::event::MB_LEFT_BUTTON;

        match event.what {
            EventType::MouseDown => {
                let mouse_pos = event.mouse.pos;
                if event.mouse.buttons & MB_LEFT_BUTTON != 0 && mouse_pos.x >= self.bounds.a.x && mouse_pos.x < self.bounds.b.x && mouse_pos.y >= self.bounds.a.y && mouse_pos.y < self.bounds.b.y {
                    self.click_count.set(self.click_count.get() + 1);
                    *event = Event::command(self.command);
                }
            }
            EventType::Broadcast => {
                if event.command == CMD_BROADCAST_TEST {
                    // Received a broadcast from a sibling!
                    self.broadcast_count.set(self.broadcast_count.get() + 1);
                    // Don't clear - let it continue to other siblings
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create a group directly (not inside a dialog)
    // This lets us call group.broadcast() directly
    let group_width = 70;
    let group_height = 20;
    let group_x = (width - group_width) / 2;
    let group_y = (height - group_height) / 2;

    let mut group = Group::new(Rect::new(group_x, group_y, group_x + group_width, group_y + group_height));

    // Create 4 buttons in a 2x2 grid
    let button_width = 18;
    let button_height = 3;

    for row in 0..2 {
        for col in 0..2 {
            let button_id = row * 2 + col;
            let button_x = 14 + col * (button_width + 5);
            let button_y = 5 + row * (button_height + 2);

            let button = BroadcastButton::new(
                Rect::new(button_x, button_y, button_x + button_width, button_y + button_height),
                &format!("Button {}", button_id + 1),
                CMD_BUTTON_BASE + button_id as u16,
            );
            group.add(Box::new(button));
        }
    }

    // Event loop
    loop {
        // Draw
        app.desktop.draw(&mut app.terminal);

        // Draw title
        let mut title_buf = DrawBuffer::new(group_width as usize);
        let title = "Broadcast Demo - Click any button";
        title_buf.move_str((group_width as usize - title.len()) / 2, title, colors::MENU_SELECTED);
        turbo_vision::views::view::write_line_to_terminal(&mut app.terminal, group_x, group_y - 3, &title_buf);

        // Draw info
        let mut info_buf = DrawBuffer::new(group_width as usize);
        let info = "Click button → broadcasts to siblings (owner skipped)";
        info_buf.move_str((group_width as usize - info.len()) / 2, info, colors::MENU_NORMAL);
        turbo_vision::views::view::write_line_to_terminal(&mut app.terminal, group_x, group_y - 2, &info_buf);

        let mut info_buf = DrawBuffer::new(group_width as usize);
        let info = "Exit: ESC-ESC, ALt+X or CTRL+C";
        info_buf.move_str((group_width as usize - info.len()) / 2, info, colors::MENU_NORMAL);
        turbo_vision::views::view::write_line_to_terminal(&mut app.terminal, group_x, group_y - 1, &info_buf);

        group.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        // Poll events
        if let Some(mut event) = app.terminal.poll_event(std::time::Duration::from_millis(50)).ok().flatten() {
            // Check for ESC ESC, ALT+X or CTRL+C
            if event.what == EventType::Keyboard && matches!(event.key_code, KB_ESC_ESC | KB_ALT_X | KB_CTRL_C) {
                break;
            }

            // Let group handle event
            group.handle_event(&mut event);

            // Check if a button was clicked
            if event.what == EventType::Command && event.command >= CMD_BUTTON_BASE && event.command < CMD_BUTTON_BASE + 4 {
                // Determine which button was clicked
                let owner_index = (event.command - CMD_BUTTON_BASE) as usize;

                // Create broadcast event
                let mut broadcast_event = Event::broadcast(CMD_BROADCAST_TEST);

                // THIS IS THE KEY LINE: Call Group::broadcast() with owner parameter
                // The owner button will NOT receive this broadcast!
                group.broadcast(&mut broadcast_event, Some(owner_index));

                // Clear the command event
                event.clear();
            }
        }
    }

    Ok(())
}
