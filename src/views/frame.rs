use crate::core::geometry::Rect;
use crate::core::event::{Event, EventType, MB_LEFT_BUTTON};
use crate::core::draw::DrawBuffer;
use crate::core::palette::{colors, Attr, TvColor};
use crate::core::command::CM_CLOSE;
use crate::terminal::Terminal;
use super::view::{View, write_line_to_terminal};

pub struct Frame {
    bounds: Rect,
    title: String,
}

impl Frame {
    pub fn new(bounds: Rect, title: &str) -> Self {
        Self {
            bounds,
            title: title.to_string(),
        }
    }
}

impl View for Frame {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        // Frame uses white on light gray background
        let frame_attr = colors::DIALOG_FRAME_ACTIVE;

        // Top border with title - using double-line box drawing
        let mut buf = DrawBuffer::new(width);
        buf.put_char(0, '╔', frame_attr);  // Double top-left corner
        buf.put_char(width - 1, '╗', frame_attr);  // Double top-right corner
        for i in 1..width - 1 {
            buf.put_char(i, '═', frame_attr);  // Double horizontal line
        }

        // Add close button at position 2: [■]
        // In the original code, closeIcon = "[~\xFE~]" where ~ marks highlight toggle
        // For active dialog: cFrame = 0x0503, so brackets use cpDialog[3] and ■ uses cpDialog[5]
        // cpDialog[3] = Cyan on Green (but we map to White on LightGray for color scheme)
        // cpDialog[5] = Magenta on Green (but we map to LightGreen on LightGray for close button)
        // See local-only/about.png: close button is bright green (LightGreen) on gray background
        if width > 5 {
            let close_icon_attr = Attr::new(TvColor::LightGreen, TvColor::LightGray);
            buf.put_char(2, '[', frame_attr);
            buf.put_char(3, '■', close_icon_attr);  // Close icon in LightGreen on LightGray (matches Borland)
            buf.put_char(4, ']', frame_attr);
        }

        // Add title after close button
        if !self.title.is_empty() && width > self.title.len() + 8 {
            buf.move_str(6, &format!(" {} ", self.title), colors::DIALOG_TITLE);
        }
        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);

        // Middle rows - using double vertical lines
        let mut side_buf = DrawBuffer::new(width);
        side_buf.put_char(0, '║', frame_attr);  // Double vertical line
        side_buf.put_char(width - 1, '║', frame_attr);  // Double vertical line
        for i in 1..width - 1 {
            side_buf.put_char(i, ' ', frame_attr);
        }
        for y in 1..height - 1 {
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &side_buf);
        }

        // Bottom border - using double-line box drawing
        let mut bottom_buf = DrawBuffer::new(width);
        bottom_buf.put_char(0, '╚', frame_attr);  // Double bottom-left corner
        bottom_buf.put_char(width - 1, '╝', frame_attr);  // Double bottom-right corner
        for i in 1..width - 1 {
            bottom_buf.put_char(i, '═', frame_attr);  // Double horizontal line
        }
        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + height as i16 - 1, &bottom_buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Handle mouse clicks on close button
        if event.what == EventType::MouseDown {
            let mouse_pos = event.mouse.pos;

            // Check if click is on the close button [■] at position (2,3,4) on the top line
            if event.mouse.buttons & MB_LEFT_BUTTON != 0
                && mouse_pos.y == self.bounds.a.y  // Top line of frame
                && mouse_pos.x >= self.bounds.a.x + 2  // Close button starts at position 2
                && mouse_pos.x <= self.bounds.a.x + 4  // Close button ends at position 4
            {
                // Generate close command
                *event = Event::command(CM_CLOSE);
            }
        }
    }
}
