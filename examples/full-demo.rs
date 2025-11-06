// (C) 2025 - Enzo Lombardi
// Full Demo - Turbo Vision Feature Demonstration
// Port of the classic Borland TV demo application

use turbo_vision::app::Application;
use turbo_vision::views::{
    dialog::Dialog,
    window::Window,
    button::Button,
    static_text::StaticText,
    menu_bar::{MenuBar, SubMenu},
    status_line::{StatusLine, StatusItem},
    View,
};
use turbo_vision::core::command::{CM_QUIT, CM_OK, CM_CLOSE, CM_NEXT, CM_PREV, CM_ZOOM};
use turbo_vision::core::event::{Event, EventType, KB_F1, KB_F3, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::palette::{colors, Attr, TvColor};
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;

// Custom commands
const CM_ABOUT: u16 = 100;
const CM_ASCII_TABLE: u16 = 101;
const CM_CALCULATOR: u16 = 102;
const CM_CALENDAR: u16 = 103;
const CM_PUZZLE: u16 = 104;
const CM_OPEN: u16 = 105;
const CM_CHDIR: u16 = 106;
const CM_TILE: u16 = 107;
const CM_CASCADE: u16 = 108;

fn create_menu_bar(width: u16) -> MenuBar {
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));

    // System menu (using ☼ symbol as in original Borland TV)
    let system_menu_items = vec![
        MenuItem::with_shortcut("~A~bout...", CM_ABOUT, 0, "", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~A~scii Table", CM_ASCII_TABLE, 0, "", 0),
        MenuItem::with_shortcut("Ca~l~culator", CM_CALCULATOR, 0, "", 0),
        MenuItem::with_shortcut("Ca~l~endar", CM_CALENDAR, 0, "", 0),
        MenuItem::with_shortcut("~P~uzzle", CM_PUZZLE, 0, "", 0),
    ];
    let system_menu = SubMenu::new("~☼~", Menu::from_items(system_menu_items));

    // File menu
    let file_menu_items = vec![
        MenuItem::with_shortcut("~O~pen...", CM_OPEN, 0, "F3", 0),
        MenuItem::with_shortcut("~C~hange Dir...", CM_CHDIR, 0, "", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ];
    let file_menu = SubMenu::new("~F~ile", Menu::from_items(file_menu_items));

    // Windows menu
    let windows_menu_items = vec![
        MenuItem::with_shortcut("~Z~oom", 0, 0, "F5", 0),
        MenuItem::with_shortcut("~N~ext", CM_NEXT, 0, "F6", 0),
        MenuItem::with_shortcut("~C~lose", CM_CLOSE, 0, "Alt+F3", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("~T~ile", CM_TILE, 0, "", 0),
        MenuItem::with_shortcut("C~a~scade", CM_CASCADE, 0, "", 0),
    ];
    let windows_menu = SubMenu::new("~W~indows", Menu::from_items(windows_menu_items));

    menu_bar.add_submenu(system_menu);
    menu_bar.add_submenu(file_menu);
    menu_bar.add_submenu(windows_menu);

    menu_bar
}

fn create_status_line(width: u16, height: u16) -> StatusLine {
    StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F1~ Help", KB_F1, CM_ABOUT),
            StatusItem::new("~F3~ Open", KB_F3, CM_OPEN),
            StatusItem::new("~F10~ Menu", KB_F10, 0),
            StatusItem::new("~Alt-X~ Exit", 0x2D00, CM_QUIT),
        ],
    )
}

fn show_about_dialog(app: &mut Application) {
    let mut dialog = Dialog::new(
        Rect::new(20, 7, 60, 16),
        "About"
    );

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 2, 36, 7),
        "Turbo Vision Demo\n\
         Version 1.0\n\
         \n\
         A demonstration of the\n\
         Turbo Vision framework",
    )));

    dialog.add(Box::new(Button::new(
        Rect::new(14, 5, 24, 7),
        "  OK  ",
        CM_OK,
        true
    )));

    dialog.set_initial_focus();
    dialog.execute(app);
}

// ASCII Table Window
struct AsciiTable {
    bounds: Rect,
    state: StateFlags,
}

impl AsciiTable {
    fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            state: 0,
        }
    }
}

impl View for AsciiTable {
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

        // Draw ASCII table (characters 32-255)
        // Format: 4 columns, showing Char Dec Hex
        for row in 0..height {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', colors::EDITOR_NORMAL, width);

            if row == 0 {
                // Header
                buf.move_str(2, "Char Dec  Hex", Attr::new(TvColor::Yellow, TvColor::Blue));
            } else if row > 0 && row <= 56 {
                // 4 columns of characters (32-255 = 224 chars / 4 = 56 rows)
                for col in 0..4 {
                    let ascii_val = 32 + ((row - 1) * 4) + col;
                    if ascii_val < 256 {
                        let x = col * 18 + 1;
                        let ch = if ascii_val < 127 {
                            char::from(ascii_val as u8)
                        } else {
                            '·' // Placeholder for extended ASCII
                        };
                        let text = format!(" {}   {:3}  {:02X}", ch, ascii_val, ascii_val);
                        buf.move_str(x, &text, colors::EDITOR_NORMAL);
                    }
                }
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + row as i16, &buf);
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {}
    fn update_cursor(&self, _terminal: &mut Terminal) {}
}

fn show_ascii_table(app: &mut Application) {
    let (width, height) = app.terminal.size();

    // Create a window in the center of the screen
    let win_width = 76i16;
    let win_height = 22i16;
    let win_x = (width as i16 - win_width) / 2;
    let win_y = (height as i16 - win_height - 2) / 2; // -2 for menu and status

    let mut window = Window::new(
        Rect::new(win_x, win_y, win_x + win_width, win_y + win_height),
        "ASCII Table"
    );

    // ASCII table fills the interior
    let ascii_table = AsciiTable::new(
        Rect::new(1, 1, win_width - 2, win_height - 2)
    );

    window.add(Box::new(ascii_table));
    app.desktop.add(Box::new(window));
}

fn show_calculator_placeholder(app: &mut Application) {
    let mut dialog = Dialog::new(Rect::new(25, 9, 55, 14), "Calculator");
    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 2, 26, 3),
        "Calculator not yet implemented"
    )));
    dialog.add(Box::new(Button::new(Rect::new(10, 3, 20, 5), "  OK  ", CM_OK, true)));
    dialog.set_initial_focus();
    dialog.execute(app);
}

fn show_calendar_placeholder(app: &mut Application) {
    let mut dialog = Dialog::new(Rect::new(25, 9, 55, 14), "Calendar");
    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 2, 26, 3),
        "Calendar not yet implemented"
    )));
    dialog.add(Box::new(Button::new(Rect::new(10, 3, 20, 5), "  OK  ", CM_OK, true)));
    dialog.set_initial_focus();
    dialog.execute(app);
}

fn show_puzzle_placeholder(app: &mut Application) {
    let mut dialog = Dialog::new(Rect::new(25, 9, 55, 14), "Puzzle");
    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 2, 26, 3),
        "Puzzle game not yet implemented"
    )));
    dialog.add(Box::new(Button::new(Rect::new(10, 3, 20, 5), "  OK  ", CM_OK, true)));
    dialog.set_initial_focus();
    dialog.execute(app);
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // Create status line
    let status_line = create_status_line(width, height);
    app.set_status_line(status_line);

    // Show about dialog on startup
    show_about_dialog(&mut app);

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
                    CM_ASCII_TABLE => show_ascii_table(&mut app),
                    CM_CALCULATOR => show_calculator_placeholder(&mut app),
                    CM_CALENDAR => show_calendar_placeholder(&mut app),
                    CM_PUZZLE => show_puzzle_placeholder(&mut app),
                    CM_NEXT => {
                        // Cycle to next window (bring next window to front)
                        app.desktop.select_next();
                    }
                    CM_PREV => {
                        // Cycle to previous window (bring previous window to front)
                        app.desktop.select_prev();
                    }
                    CM_TILE => {
                        // TODO: Implement tile windows
                    }
                    CM_CASCADE => {
                        // TODO: Implement cascade windows
                    }
                    _ => {}
                }
            }
        }

        app.idle();
        app.desktop.remove_closed_windows();
        app.desktop.handle_moved_windows(&mut app.terminal);
    }

    Ok(())
}
