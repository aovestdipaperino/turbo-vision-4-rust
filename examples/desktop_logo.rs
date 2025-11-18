// (C) 2025 - Enzo Lombardi
// Desklogo Example - Custom Desktop Background
// Port of the Borland Turbo Vision desklogo example
// Demonstrates how to customize the desktop background with a pattern

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{Event, EventType};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::palette::{Attr, TvColor, Palette};
use turbo_vision::core::state::StateFlags;
use turbo_vision::helpers::msgbox::{MF_ABOUT, MF_OK_BUTTON, message_box};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{
    View, IdleView,
    menu_bar::{MenuBar, SubMenu},
    status_line::{StatusItem, StatusLine},
};
use std::time::Instant;

// Custom command for About dialog
const CM_ABOUT: u16 = 100;

// Animated Crab Widget for Status Bar
struct CrabWidget {
    bounds: Rect,
    state: StateFlags,
    position: usize,      // Current position (0-9)
    direction: i8,        // 1 for right, -1 for left
    last_update: Instant,
}

impl CrabWidget {
    fn new(x: i16, y: i16) -> Self {
        Self {
            bounds: Rect::new(x, y, x + 10, y + 1),
            state: 0,
            position: 0,
            direction: 1,
            last_update: Instant::now(),
        }
    }
}

impl View for CrabWidget {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let mut buf = DrawBuffer::new(10);
        // Use status line colors (reverse video)
        let color = Attr::new(TvColor::Black, TvColor::LightGray);

        // Fill with spaces
        for i in 0..10 {
            buf.move_char(i, ' ', color, 1);
        }

        // Place the crab at current position
        buf.move_char(self.position, '🦀', color, 1);

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {}
    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn get_palette(&self) -> Option<Palette> {
        None
    }
}

impl IdleView for CrabWidget {
    fn idle(&mut self) {
        // Update animation every 100ms
        if self.last_update.elapsed().as_millis() > 100 {
            // Move the crab
            if self.direction > 0 {
                self.position += 1;
                if self.position >= 9 {
                    self.direction = -1;
                }
            } else {
                if self.position > 0 {
                    self.position -= 1;
                }
                if self.position == 0 {
                    self.direction = 1;
                }
            }
            self.last_update = Instant::now();
        }
    }
}

// The Turbo Vision logo pattern (23 rows x 80 columns)
// ASCII art logo pattern
const LOGO_LINES: [&str; 13] = [
    "████████╗██╗   ██╗██████╗ ██████╗  ██████╗ ",
    "╚══██╔══╝██║   ██║██╔══██╗██╔══██╗██╔═══██╗",
    "   ██║   ██║   ██║██████╔╝██████╔╝██║   ██║",
    "   ██║   ██║   ██║██╔══██╗██╔══██╗██║   ██║",
    "   ██║   ╚██████╔╝██║  ██║██████╔╝╚██████╔╝",
    "   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚═════╝  ╚═════╝ ",
    "                                           ",
    "██╗   ██╗██╗███████╗██╗ ██████╗ ███╗   ██╗ ",
    "██║   ██║██║██╔════╝██║██╔═══██╗████╗  ██║ ",
    "██║   ██║██║███████╗██║██║   ██║██╔██╗ ██║ ",
    "╚██╗ ██╔╝██║╚════██║██║██║   ██║██║╚██╗██║ ",
    " ╚████╔╝ ██║███████║██║╚██████╔╝██║ ╚████║ ",
    "  ╚═══╝  ╚═╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝ ",
];

// Custom Desktop Background with Logo Pattern
struct LogoBackground {
    bounds: Rect,
    state: StateFlags,
}

impl LogoBackground {
    fn new(bounds: Rect) -> Self {
        Self { bounds, state: 0 }
    }
}

impl View for LogoBackground {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn state(&self) -> StateFlags {
        self.state
    }

    fn set_state(&mut self, state: StateFlags) {
        self.state = state;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;
        // Use cyan background for desktop
        let color = Attr::new(TvColor::LightGray, TvColor::DarkGray);

        // Calculate logo dimensions
        let logo_width = LOGO_LINES
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let logo_height = LOGO_LINES.len();

        // Calculate center position
        let x_offset = (width.saturating_sub(logo_width)) / 2;
        let y_offset = (height.saturating_sub(logo_height)) / 2;

        for i in 0..height {
            let mut buf = DrawBuffer::new(width);

            // Fill the entire line with spaces first
            for j in 0..width {
                buf.move_char(j, ' ', color, 1);
            }

            // Draw logo if we're in the logo area
            if i >= y_offset && i < y_offset + logo_height {
                let logo_line_idx = i - y_offset;
                let logo_line = LOGO_LINES[logo_line_idx];

                // Draw each character of the logo at the centered position
                for (j, ch) in logo_line.chars().enumerate() {
                    if x_offset + j < width {
                        buf.move_char(x_offset + j, ch, color, 1);
                    }
                }
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {}
    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

fn create_menu_bar(width: i16) -> MenuBar {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width, 1));

    // About menu
    let about_menu_items = vec![MenuItem::with_shortcut("~A~bout", CM_ABOUT, 0, "Alt+A", 0)];
    let about_menu = SubMenu::new("~A~bout", Menu::from_items(about_menu_items));

    menu_bar.add_submenu(about_menu);
    menu_bar
}

fn create_status_line(width: i16, height: i16) -> StatusLine {
    use turbo_vision::core::event::KB_ALT_X;

    let status_items = vec![StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT)];

    StatusLine::new(
        Rect::new(0, height - 1, width, height),
        status_items,
    )
}

fn show_about_dialog(app: &mut Application) {
    let message = "Turbo Vision Example\n\n\
                   Modifying the desk top\n\n\
                   Borland Technical Support";

    message_box(app, message, MF_ABOUT | MF_OK_BUTTON);
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar (this adjusts desktop bounds)
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // Create status line (this adjusts desktop bounds again)
    let status_line = create_status_line(width, height);
    app.set_status_line(status_line);

    // Create custom desktop background with logo using desktop's bounds
    let desktop_bounds = app.desktop.bounds();
    let logo_bg = LogoBackground::new(Rect::new(
        0,
        0,
        desktop_bounds.width(),
        desktop_bounds.height(),
    ));
    app.desktop.add(Box::new(logo_bg));

    // Create animated crab widget on the right side of the status bar
    // Add it as an overlay widget so it continues animating even during modal dialogs
    let crab_widget = CrabWidget::new(width - 11, height - 1);
    app.add_overlay_widget(Box::new(crab_widget));

    // Main event loop
    app.running = true;
    while app.running {
        app.draw();
        app.terminal.flush()?;

        if let Ok(Some(mut event)) = app
            .terminal
            .poll_event(std::time::Duration::from_millis(50))
        {
            // Menu bar handles events first
            if let Some(ref mut menu_bar) = app.menu_bar {
                menu_bar.handle_event(&mut event);

                // Check for cascading submenu
                if event.what == EventType::Keyboard || event.what == EventType::MouseUp {
                    if let Some(command) = menu_bar.check_cascading_submenu(&mut app.terminal) {
                        if command != 0 {
                            event = Event::command(command);
                        }
                    }
                }
            }

            // Status line handles events
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Desktop handles events
            app.desktop.handle_event(&mut event);

            // Handle commands
            if event.what == EventType::Command {
                match event.command {
                    CM_QUIT => app.running = false,
                    CM_ABOUT => show_about_dialog(&mut app),
                    _ => {}
                }
            }
        }

        // Idle processing - updates crab animation and other idle tasks
        app.idle();
    }

    Ok(())
}
