// (C) 2025 - Enzo Lombardi
// Desklogo Example - Custom Desktop Background
// Port of the Borland Turbo Vision desklogo example
// Demonstrates how to customize the desktop background with a pattern

use turbo_vision::app::Application;
use turbo_vision::views::{
    dialog::DialogBuilder,
    button::ButtonBuilder,
    static_text::StaticTextBuilder,
    menu_bar::{MenuBar, SubMenu},
    status_line::{StatusLine, StatusItem},
    View,
};
use turbo_vision::core::command::{CM_QUIT, CM_OK};
use turbo_vision::core::event::{Event, EventType};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::palette::{Attr, TvColor};
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;

// Custom command for About dialog
const CM_ABOUT: u16 = 100;

// The Turbo Vision logo pattern (23 rows x 80 columns)
// ASCII art logo pattern
const LOGO_LINES: [&str; 23] = [
    "                                                                                ",
    "                                                                                ",
    "    ████████╗██╗   ██╗██████╗ ██████╗  ██████╗                                 ",
    "    ╚══██╔══╝██║   ██║██╔══██╗██╔══██╗██╔═══██╗                                ",
    "       ██║   ██║   ██║██████╔╝██████╔╝██║   ██║                                ",
    "       ██║   ██║   ██║██╔══██╗██╔══██╗██║   ██║                                ",
    "       ██║   ╚██████╔╝██║  ██║██████╔╝╚██████╔╝                                ",
    "       ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚═════╝  ╚═════╝                                 ",
    "                                                                                ",
    "    ██╗   ██╗██╗███████╗██╗ ██████╗ ███╗   ██╗                                 ",
    "    ██║   ██║██║██╔════╝██║██╔═══██╗████╗  ██║                                 ",
    "    ██║   ██║██║███████╗██║██║   ██║██╔██╗ ██║                                 ",
    "    ╚██╗ ██╔╝██║╚════██║██║██║   ██║██║╚██╗██║                                 ",
    "     ╚████╔╝ ██║███████║██║╚██████╔╝██║ ╚████║                                 ",
    "      ╚═══╝  ╚═╝╚══════╝╚═╝ ╚═════╝ ╚═╝  ╚═══╝                                 ",
    "                                                                                ",
    "                     Rust Edition - 2025                                       ",
    "                                                                                ",
    "                   A Modern TUI Framework                                      ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
    "                                                                                ",
];

// Custom Desktop Background with Logo Pattern
struct LogoBackground {
    bounds: Rect,
    state: StateFlags,
}

impl LogoBackground {
    fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            state: 0,
        }
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
        let color = Attr::new(TvColor::Black, TvColor::Cyan);

        for i in 0..height {
            let mut buf = DrawBuffer::new(width);

            for j in 0..width {
                let ch = if i < LOGO_LINES.len() {
                    // Use character from logo pattern
                    LOGO_LINES[i].chars().nth(j).unwrap_or(' ')
                } else {
                    // Fill remaining area with spaces
                    ' '
                };
                buf.move_char(j, ch, color, 1);
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

fn create_menu_bar(width: u16) -> MenuBar {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // About menu
    let about_menu_items = vec![
        MenuItem::with_shortcut("~A~bout", CM_ABOUT, 0, "Alt+A", 0),
    ];
    let about_menu = SubMenu::new("~A~bout", Menu::from_items(about_menu_items));

    menu_bar.add_submenu(about_menu);
    menu_bar
}

fn create_status_line(width: u16, height: u16) -> StatusLine {
    use turbo_vision::core::event::KB_ALT_X;

    let status_items = vec![
        StatusItem::new("~Alt+X~ Exit", KB_ALT_X, CM_QUIT),
    ];

    StatusLine::new(Rect::new(0, height as i16 - 1, width as i16, height as i16), status_items)
}

fn show_about_dialog(app: &mut Application) {
    use turbo_vision::core::state::OF_CENTERED;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(0, 0, 35, 12))
        .title("About")
        .build();
    dialog.set_options(dialog.options() | OF_CENTERED);

    // Static text with centered content
    let text = StaticTextBuilder::new()
        .bounds(Rect::new(1, 2, 34, 7))
        .text("\nTurbo Vision Example\n\n\
         Modifying the desk top\n\n\
         Borland Technical Support")
        .centered(true)
        .build();
    dialog.add(Box::new(text));

    // OK button
    let ok_button = ButtonBuilder::new()
        .bounds(Rect::new(3, 9, 32, 11))
        .title("  ~O~K  ")
        .command(CM_OK)
        .default(true)
        .build();
    dialog.add(Box::new(ok_button));

    dialog.set_initial_focus();
    dialog.execute(app);
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create custom desktop background with logo
    let logo_bg = LogoBackground::new(Rect::new(0, 1, width as i16, height as i16 - 1));
    app.desktop.add(Box::new(logo_bg));

    // Create menu bar
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // Create status line
    let status_line = create_status_line(width, height);
    app.set_status_line(status_line);

    // Main event loop
    app.running = true;
    while app.running {
        app.draw();
        app.terminal.flush()?;

        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
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
    }

    Ok(())
}
