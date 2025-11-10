// (C) 2025 - Enzo Lombardi
// Full Demo - Turbo Vision Feature Demonstration
// Port of the classic Borland TV demo application

use std::time::SystemTime;
use turbo_vision::app::Application;
use turbo_vision::core::command::{
    CM_CASCADE, CM_CLOSE, CM_NEXT, CM_OK, CM_PREV, CM_QUIT, CM_TILE, CM_ZOOM,
};
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{Event, EventType, KB_F1, KB_F10, KB_F3};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::palette::{colors, Attr, TvColor};
use turbo_vision::core::state::StateFlags;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{
    button::ButtonBuilder,
    dialog::DialogBuilder,
    file_dialog::FileDialogBuilder,
    input_line::InputLineBuilder,
    label::LabelBuilder,
    listbox::ListBoxBuilder,
    menu_bar::{MenuBar, SubMenu},
    static_text::StaticTextBuilder,
    status_line::{StatusItem, StatusLine},
    window::WindowBuilder,
    View,
};

// Custom commands
const CM_ABOUT: u16 = 100;
const CM_ASCII_TABLE: u16 = 101;
const CM_CALCULATOR: u16 = 102;
const CM_CALENDAR: u16 = 103;
const CM_PUZZLE: u16 = 104;
const CM_OPEN: u16 = 105;
const CM_CHDIR: u16 = 106;

// Calculator button commands
#[allow(dead_code)]
const CM_CALC_BUTTON: u16 = 200;
#[allow(dead_code)]
const CM_CALC_CLEAR: u16 = 200;
#[allow(dead_code)]
const CM_CALC_DELETE: u16 = 201;
#[allow(dead_code)]
const CM_CALC_PERCENT: u16 = 202;
#[allow(dead_code)]
const CM_CALC_PLUSMIN: u16 = 203;
#[allow(dead_code)]
const CM_CALC_7: u16 = 204;
#[allow(dead_code)]
const CM_CALC_8: u16 = 205;
#[allow(dead_code)]
const CM_CALC_9: u16 = 206;
#[allow(dead_code)]
const CM_CALC_DIV: u16 = 207;
#[allow(dead_code)]
const CM_CALC_4: u16 = 208;
#[allow(dead_code)]
const CM_CALC_5: u16 = 209;
#[allow(dead_code)]
const CM_CALC_6: u16 = 210;
#[allow(dead_code)]
const CM_CALC_MUL: u16 = 211;
#[allow(dead_code)]
const CM_CALC_1: u16 = 212;
#[allow(dead_code)]
const CM_CALC_2: u16 = 213;
#[allow(dead_code)]
const CM_CALC_3: u16 = 214;
#[allow(dead_code)]
const CM_CALC_MINUS: u16 = 215;
#[allow(dead_code)]
const CM_CALC_0: u16 = 216;
#[allow(dead_code)]
const CM_CALC_DECIMAL: u16 = 217;
#[allow(dead_code)]
const CM_CALC_EQUAL: u16 = 218;
#[allow(dead_code)]
const CM_CALC_PLUS: u16 = 219;

// ClockView - displays live time on menu bar
struct ClockView {
    bounds: Rect,
    state: StateFlags,
    owner: Option<*const dyn View>,
}

impl ClockView {
    fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            state: 0,
            owner: None,
        }
    }

    fn get_time_string() -> String {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let hours = ((now / 3600) % 24) as u8;
        let minutes = ((now / 60) % 60) as u8;
        let seconds = (now % 60) as u8;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

impl View for ClockView {
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
        let color = colors::MENU_NORMAL;

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', color, width);

        let time_str = Self::get_time_string();
        if time_str.len() <= width {
            buf.move_str(0, &time_str, color);
        }

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {}
    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

// MessageView - displays static message on status bar
struct MessageView {
    bounds: Rect,
    state: StateFlags,
    message: String,
    owner: Option<*const dyn View>,
}

impl MessageView {
    fn new(bounds: Rect, message: &str) -> Self {
        Self {
            bounds,
            state: 0,
            message: message.to_string(),
            owner: None,
        }
    }
}

impl View for MessageView {
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
        let color = colors::STATUS_NORMAL;

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', color, width);

        if self.message.len() <= width {
            // Right-align the message
            let x_pos = width.saturating_sub(self.message.len());
            buf.move_str(x_pos, &self.message, color);
        }

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, _event: &mut Event) {}
    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

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
        MenuItem::with_shortcut("~Z~oom", CM_ZOOM, 0, "F5", 0),
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
    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(20, 7, 60, 16))
        .title("About")
        .build();

    dialog.add(Box::new(StaticTextBuilder::new()
        .bounds(Rect::new(2, 2, 36, 7))
        .text("Turbo Vision Demo\n\
         Version 1.0\n\
         \n\
         A demonstration of the\n\
         Turbo Vision framework")
        .build()));

    dialog.add(Box::new(ButtonBuilder::new()
        .bounds(Rect::new(14, 5, 24, 7))
        .title("  OK  ")
        .command(CM_OK)
        .default(true)
        .build()));

    dialog.set_initial_focus();
    dialog.execute(app);
}

// ASCII Table Window
struct AsciiTable {
    bounds: Rect,
    state: StateFlags,
    owner: Option<*const dyn View>,
}

impl AsciiTable {
    fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            state: 0,
            owner: None,
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
                buf.move_str(
                    2,
                    "Char Dec  Hex",
                    Attr::new(TvColor::Yellow, TvColor::Blue),
                );
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

            write_line_to_terminal(
                terminal,
                self.bounds.a.x,
                self.bounds.a.y + row as i16,
                &buf,
            );
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {}
    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

fn show_ascii_table(app: &mut Application) {
    let (width, height) = app.terminal.size();

    // Create a window in the center of the screen
    let win_width = 76i16;
    let win_height = 22i16;
    let win_x = (width as i16 - win_width) / 2;
    let win_y = (height as i16 - win_height - 2) / 2; // -2 for menu and status

    let mut window = WindowBuilder::new()
        .bounds(Rect::new(win_x, win_y, win_x + win_width, win_y + win_height))
        .title("ASCII Table")
        .build();

    // ASCII table fills the interior
    let ascii_table = AsciiTable::new(Rect::new(1, 1, win_width - 2, win_height - 2));

    window.add(Box::new(ascii_table));
    app.desktop.add(Box::new(window));
}

// Calculator Implementation
#[derive(Debug, Clone, Copy, PartialEq)]
enum CalcState {
    First,
    Valid,
    Error,
}

struct CalcDisplay {
    bounds: Rect,
    state: StateFlags,
    options: u16,
    calc_state: CalcState,
    number: String,
    sign: char,
    operator: char,
    operand: f64,
    owner: Option<*const dyn View>,
}

impl CalcDisplay {
    fn new(bounds: Rect) -> Self {
        use turbo_vision::core::state::{OF_SELECTABLE, SF_VISIBLE};

        Self {
            bounds,
            state: SF_VISIBLE,
            options: OF_SELECTABLE, // Must be selectable to receive keyboard events
            calc_state: CalcState::First,
            number: "0".to_string(),
            sign: ' ',
            operator: '=',
            operand: 0.0,
            owner: None,
        }
    }

    fn clear(&mut self) {
        self.calc_state = CalcState::First;
        self.number = "0".to_string();
        self.sign = ' ';
        self.operator = '=';
    }

    fn error(&mut self) {
        self.calc_state = CalcState::Error;
        self.number = "Error".to_string();
        self.sign = ' ';
    }

    fn get_display(&self) -> f64 {
        self.number.parse::<f64>().unwrap_or(0.0)
    }

    fn set_display(&mut self, r: f64) {
        if r < 0.0 {
            self.sign = '-';
            self.number = format!("{:.10}", -r);
        } else {
            self.sign = ' ';
            self.number = format!("{:.10}", r);
        }

        // Remove trailing zeros after decimal point
        if self.number.contains('.') {
            self.number = self
                .number
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
        }

        if self.number.len() > 25 {
            self.error();
        }
    }

    fn check_first(&mut self) {
        if self.calc_state == CalcState::First {
            self.calc_state = CalcState::Valid;
            self.number = "0".to_string();
            self.sign = ' ';
        }
    }

    fn calc_key(&mut self, key: char) {
        if self.calc_state == CalcState::Error && key != 'C' {
            return;
        }

        match key {
            '0'..='9' => {
                self.check_first();
                if self.number.len() < 15 {
                    if self.number == "0" {
                        self.number.clear();
                    }
                    self.number.push(key);
                }
            }
            '\x08' | '\x1b' => {
                // Backspace or Escape
                self.check_first();
                if self.number.len() == 1 {
                    self.number = "0".to_string();
                } else {
                    self.number.pop();
                }
            }
            '_' => {
                // +/- (toggle sign)
                self.sign = if self.sign == ' ' { '-' } else { ' ' };
            }
            '.' => {
                self.check_first();
                if !self.number.contains('.') {
                    self.number.push('.');
                }
            }
            '+' | '-' | '*' | '/' | '=' | '%' | '\r' => {
                if self.calc_state == CalcState::Valid {
                    self.calc_state = CalcState::First;
                    let mut r = self.get_display() * if self.sign == '-' { -1.0 } else { 1.0 };

                    if key == '%' {
                        if self.operator == '+' || self.operator == '-' {
                            r = (self.operand * r) / 100.0;
                        } else {
                            r /= 100.0;
                        }
                    }

                    match self.operator {
                        '+' => self.set_display(self.operand + r),
                        '-' => self.set_display(self.operand - r),
                        '*' => self.set_display(self.operand * r),
                        '/' => {
                            if r == 0.0 {
                                self.error();
                            } else {
                                self.set_display(self.operand / r);
                            }
                        }
                        _ => {}
                    }
                }
                self.operator = key;
                self.operand = self.get_display() * if self.sign == '-' { -1.0 } else { 1.0 };
            }
            'C' => self.clear(),
            _ => {}
        }
    }

    fn handle_command(&mut self, command: u16) {
        let keys = [
            'C', '\x08', '%', '_', '7', '8', '9', '/', '4', '5', '6', '*', '1', '2', '3', '-', '0',
            '.', '=', '+',
        ];
        if command >= CM_CALC_BUTTON && command < CM_CALC_BUTTON + 20 {
            let idx = (command - CM_CALC_BUTTON) as usize;
            if idx < keys.len() {
                self.calc_key(keys[idx]);
            }
        }
    }
}

impl View for CalcDisplay {
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

    fn options(&self) -> u16 {
        self.options
    }

    fn set_options(&mut self, options: u16) {
        self.options = options;
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        let width = self.bounds.width() as usize;
        // Use LightCyan background (closest to RGB(175, 212, 250))
        let color = Attr::new(TvColor::Black, TvColor::LightCyan);

        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', color, width);

        // Right-align the display
        let display_text = format!("{}{}", self.sign, self.number);
        let x_pos = width.saturating_sub(display_text.len() + 1);
        buf.move_str(x_pos, &display_text, color);

        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);
    }

    fn handle_event(&mut self, event: &mut Event) {
        // Matches Borland: calcdisp.cc:61-83
        // Handle both keyboard events (when display has focus) and broadcasts from buttons
        match event.what {
            EventType::Keyboard => {
                // Handle keyboard input directly when display has focus
                let key_char = if event.key_code < 256 {
                    (event.key_code as u8) as char
                } else {
                    match event.key_code {
                        turbo_vision::core::event::KB_BACKSPACE => '\x08',
                        turbo_vision::core::event::KB_ESC => '\x1b',
                        turbo_vision::core::event::KB_ENTER => '\r',
                        _ => return,
                    }
                };
                self.calc_key(key_char.to_ascii_uppercase());
                event.what = EventType::Nothing;
            }
            EventType::Broadcast => {
                // Handle broadcasts from calculator buttons
                // Matches Borland: calcdisp.cc:74-81
                if event.command >= CM_CALC_BUTTON && event.command < CM_CALC_BUTTON + 20 {
                    self.handle_command(event.command);
                    event.what = EventType::Nothing;
                }
            }
            _ => {}
        }
    }

    fn can_focus(&self) -> bool {
        // CalcDisplay can receive focus to handle keyboard input
        // Matches Borland: ofSelectable is set in constructor (calcdisp.cc:42)
        true
    }

    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None  // CalcDisplay uses hardcoded colors
    }
}

fn show_calculator_placeholder(app: &mut Application) {
    use std::io::Write;
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("calc.log")
        .unwrap();

    writeln!(log, "\n=== show_calculator_placeholder START ===").unwrap();

    let display_len = 25;
    let dialog_width = 6 + display_len;
    let dialog_height = 15; // Back to original height

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(5, 3, 5 + dialog_width as i16, 3 + dialog_height as i16))
        .title("Calculator")
        .build();

    // Add display at top - moved 1 row up and 1 to the left
    let display = CalcDisplay::new(Rect::new(2, 1, 2 + display_len as i16, 2));
    dialog.add(Box::new(display));

    // Add buttons in 4x5 grid
    let button_labels = [
        "C", "<-", "%", "+-", "7", "8", "9", "/", "4", "5", "6", "*", "1", "2", "3", "-", "0", ".",
        "=", "+",
    ];

    for i in 0..20 {
        // Moved 1 row up and 1 to the left
        let x = (i % 4) * 6 + 2;
        let y = (i / 4) * 2 + 3;
        let mut button = ButtonBuilder::new()
            .bounds(Rect::new(x, y, x + 6, y + 2))
            .title(button_labels[i as usize])
            .command(CM_CALC_BUTTON + i as u16)
            .default(false)
            .build();
        // Make button broadcast and non-selectable
        // Matches Borland: bfBroadcast flag and ofSelectable cleared (calculat.cc:54-55)
        button.set_broadcast(true);
        button.set_selectable(false);
        writeln!(log, "Adding button {}: '{}'", i, button_labels[i as usize]).unwrap();
        dialog.add(Box::new(button));
    }

    dialog.set_initial_focus();

    // Add to desktop as non-modal window (like Borland does)
    writeln!(log, "Adding dialog to desktop...").unwrap();
    app.desktop.add(Box::new(dialog));
    writeln!(log, "=== show_calculator_placeholder DONE ===\n").unwrap();
}

// Calendar Implementation
struct CalendarView {
    bounds: Rect,
    state: StateFlags,
    month: u32,
    year: u32,
    cur_day: u32,
    cur_month: u32,
    cur_year: u32,
    owner: Option<*const dyn View>,
}

impl CalendarView {
    fn new(bounds: Rect) -> Self {
        use std::time::UNIX_EPOCH;

        // Get current date
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simple date calculation (days since epoch)
        let days_since_epoch = now / 86400;
        let (year, month, day) = Self::epoch_to_date(days_since_epoch as i32);

        Self {
            bounds,
            state: 0,
            month,
            year,
            cur_day: day,
            cur_month: month,
            cur_year: year,
            owner: None,
        }
    }

    fn epoch_to_date(days: i32) -> (u32, u32, u32) {
        // January 1, 1970 was a Thursday
        let mut year = 1970;
        let mut remaining_days = days;

        loop {
            let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            year += 1;
        }

        let days_in_month = [
            31,
            if Self::is_leap_year(year) { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];

        let mut month = 1;
        for &days in &days_in_month {
            if remaining_days < days {
                break;
            }
            remaining_days -= days;
            month += 1;
        }

        let day = remaining_days + 1;
        (year as u32, month, day as u32)
    }

    fn is_leap_year(year: u32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn days_in_month(month: u32, year: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }

    fn day_of_week(day: u32, month: u32, year: u32) -> u32 {
        // Zeller's congruence algorithm
        let mut m = month as i32;
        let mut y = year as i32;

        if m < 3 {
            m += 10;
            y -= 1;
        } else {
            m -= 2;
        }

        let century = y / 100;
        let yr = y % 100;
        let mut dw =
            (((26 * m - 2) / 10) + day as i32 + yr + (yr / 4) + (century / 4) - (2 * century)) % 7;

        if dw < 0 {
            dw += 7;
        }

        dw as u32
    }

    fn month_name(month: u32) -> &'static str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "",
        }
    }

    fn next_month(&mut self) {
        self.month += 1;
        if self.month > 12 {
            self.month = 1;
            self.year += 1;
        }
    }

    fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
    }
}

impl View for CalendarView {
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

        let color = Attr::new(TvColor::Black, TvColor::Cyan);
        let bold_color = Attr::new(TvColor::Yellow, TvColor::Cyan);

        // Line 0: Month and year with up/down arrows
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', color, width);
        let header = format!("↑{:>12} {:4} ↓", Self::month_name(self.month), self.year);
        buf.move_str(0, &header, color);
        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);

        // Line 1: Day headers
        let mut buf = DrawBuffer::new(width);
        buf.move_char(0, ' ', color, width);
        buf.move_str(0, "Su Mo Tu We Th Fr Sa", color);
        write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + 1, &buf);

        // Calculate starting day
        let first_day_of_week = Self::day_of_week(1, self.month, self.year);
        let days_in_month = Self::days_in_month(self.month, self.year);

        let mut current = if first_day_of_week == 0 {
            1
        } else {
            1 - first_day_of_week as i32
        };

        // Lines 2-7: Calendar grid (6 weeks)
        for week in 0..6 {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', color, width);

            for day_of_week in 0..7 {
                if current < 1 || current > days_in_month as i32 {
                    buf.move_str(day_of_week * 3, "   ", color);
                } else {
                    let day_str = format!("{:2}", current);
                    let day_color = if self.year == self.cur_year
                        && self.month == self.cur_month
                        && current == self.cur_day as i32
                    {
                        bold_color
                    } else {
                        color
                    };
                    buf.move_str(day_of_week * 3, &day_str, day_color);
                }
                current += 1;
            }

            write_line_to_terminal(
                terminal,
                self.bounds.a.x,
                self.bounds.a.y + 2 + week as i16,
                &buf,
            );
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        use turbo_vision::core::event::{KB_DOWN, KB_UP};

        match event.what {
            EventType::MouseDown => {
                let local_x = event.mouse.pos.x - self.bounds.a.x;
                let local_y = event.mouse.pos.y - self.bounds.a.y;

                // Check if clicked on up arrow (position 0, character at x=0)
                if local_y == 0 && local_x == 0 {
                    self.prev_month();
                    event.what = EventType::Nothing;
                }
                // Check if clicked on down arrow (position 18)
                else if local_y == 0 && local_x >= 18 {
                    self.next_month();
                    event.what = EventType::Nothing;
                }
            }
            EventType::Keyboard => {
                match event.key_code {
                    KB_DOWN | 0x2B => {
                        // Down arrow or '+'
                        self.next_month();
                        event.what = EventType::Nothing;
                    }
                    KB_UP | 0x2D => {
                        // Up arrow or '-'
                        self.prev_month();
                        event.what = EventType::Nothing;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

fn show_calendar_placeholder(app: &mut Application) {
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(1, 1, 24, 11))
        .title("Calendar")
        .build();

    let calendar_view = CalendarView::new(Rect::new(0, 0, 22, 10));
    window.add(Box::new(calendar_view));

    app.desktop.add(Box::new(window));
}

// Puzzle Game Implementation
struct PuzzleView {
    bounds: Rect,
    state: StateFlags,
    board: [[char; 6]; 6],
    moves: u16,
    solved: bool,
    owner: Option<*const dyn View>,
}

impl PuzzleView {
    fn new(bounds: Rect) -> Self {
        let mut puzzle = Self {
            bounds,
            state: 0,
            board: [[' '; 6]; 6],
            moves: 0,
            solved: false,
            owner: None,
        };

        // Initialize board with starting position
        let board_start = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', ' ',
        ];

        for i in 0..4 {
            for j in 0..4 {
                puzzle.board[i][j] = board_start[i * 4 + j];
            }
        }

        puzzle.scramble();
        puzzle
    }

    fn scramble(&mut self) {
        //use std::time::{SystemTime, UNIX_EPOCH};
        use std::time::UNIX_EPOCH;

        self.moves = 0;
        self.solved = false;

        // Use system time as seed
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let mut rng = seed;

        // Make 500 random moves
        for _ in 0..500 {
            // Simple LCG random number generator
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let direction = ((rng >> 16) % 4) as usize;

            match direction {
                0 => self.move_key(turbo_vision::core::event::KB_UP),
                1 => self.move_key(turbo_vision::core::event::KB_DOWN),
                2 => self.move_key(turbo_vision::core::event::KB_RIGHT),
                3 => self.move_key(turbo_vision::core::event::KB_LEFT),
                _ => {}
            }
            self.moves += 1;
        }

        self.moves = 0;
    }

    fn move_key(&mut self, key: u16) {
        // Find the empty space
        let mut empty_x = 0;
        let mut empty_y = 0;
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] == ' ' {
                    empty_x = j;
                    empty_y = i;
                    break;
                }
            }
        }

        use turbo_vision::core::event::{KB_DOWN, KB_LEFT, KB_RIGHT, KB_UP};

        match key {
            KB_DOWN => {
                if empty_y > 0 {
                    self.board[empty_y][empty_x] = self.board[empty_y - 1][empty_x];
                    self.board[empty_y - 1][empty_x] = ' ';
                    if self.moves < 1000 {
                        self.moves += 1;
                    }
                }
            }
            KB_UP => {
                if empty_y < 3 {
                    self.board[empty_y][empty_x] = self.board[empty_y + 1][empty_x];
                    self.board[empty_y + 1][empty_x] = ' ';
                    if self.moves < 1000 {
                        self.moves += 1;
                    }
                }
            }
            KB_RIGHT => {
                if empty_x > 0 {
                    self.board[empty_y][empty_x] = self.board[empty_y][empty_x - 1];
                    self.board[empty_y][empty_x - 1] = ' ';
                    if self.moves < 1000 {
                        self.moves += 1;
                    }
                }
            }
            KB_LEFT => {
                if empty_x < 3 {
                    self.board[empty_y][empty_x] = self.board[empty_y][empty_x + 1];
                    self.board[empty_y][empty_x + 1] = ' ';
                    if self.moves < 1000 {
                        self.moves += 1;
                    }
                }
            }
            _ => {}
        }
    }

    fn move_tile(&mut self, p: turbo_vision::core::geometry::Point) {
        // Convert screen coordinates to local coordinates
        let local_x = p.x - self.bounds.a.x;
        let local_y = p.y - self.bounds.a.y;

        // Find the empty space
        let mut empty_idx = 0;
        for i in 0..16 {
            if self.board[i / 4][i % 4] == ' ' {
                empty_idx = i;
                break;
            }
        }

        let x = local_x / 3;
        let y = local_y;

        let clicked_idx = (y * 4 + x) as usize;
        let diff = clicked_idx as i32 - empty_idx as i32;

        match diff {
            -4 => self.move_key(turbo_vision::core::event::KB_DOWN),
            -1 => self.move_key(turbo_vision::core::event::KB_RIGHT),
            1 => self.move_key(turbo_vision::core::event::KB_LEFT),
            4 => self.move_key(turbo_vision::core::event::KB_UP),
            _ => {}
        }
    }

    fn win_check(&mut self) {
        let solution = "ABCDEFGHIJKLMNO ";
        let mut idx = 0;
        for i in 0..4 {
            for j in 0..4 {
                if self.board[i][j] != solution.chars().nth(idx).unwrap() {
                    return;
                }
                idx += 1;
            }
        }
        self.solved = true;
    }
}

impl View for PuzzleView {
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

        // Color map for alternating tile colors
        let map = [0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1];

        let color_normal = Attr::new(TvColor::LightGray, TvColor::Blue);
        let color_alt = Attr::new(TvColor::White, TvColor::Cyan);
        let color_back = Attr::new(TvColor::LightGray, TvColor::Blue);

        for i in 0..4 {
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', color_back, width);

            if i == 1 {
                buf.move_str(13, "Moves", color_back);
            }
            if i == 2 {
                let moves_str = format!("{}", self.moves);
                buf.move_str(14, &moves_str, color_back);
            }

            for j in 0..4 {
                let tile = self.board[i][j];
                let tile_str = format!(" {} ", tile);

                let color = if tile == ' ' {
                    color_normal
                } else if self.solved {
                    color_normal
                } else {
                    let tile_idx = (tile as usize).saturating_sub('A' as usize);
                    if tile_idx < 15 && map[tile_idx] == 1 {
                        color_alt
                    } else {
                        color_normal
                    }
                };

                buf.move_str(j * 3, &tile_str, color);
            }

            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + i as i16, &buf);
        }
    }

    fn handle_event(&mut self, event: &mut Event) {
        if self.solved && (event.what == EventType::Keyboard || event.what == EventType::MouseDown)
        {
            self.scramble();
            event.what = EventType::Nothing;
            return;
        }

        match event.what {
            EventType::MouseDown => {
                self.move_tile(event.mouse.pos);
                self.win_check();
                event.what = EventType::Nothing;
            }
            EventType::Keyboard => {
                self.move_key(event.key_code);
                self.win_check();
                event.what = EventType::Nothing;
            }
            _ => {}
        }
    }

    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn set_owner(&mut self, owner: *const dyn View) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> Option<*const dyn View> {
        self.owner
    }

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

fn show_puzzle_placeholder(app: &mut Application) {
    // Create non-resizable window using builder pattern
    // Size increased by 1 row and 1 column: was 20x6, now 21x7
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(1, 1, 22, 8))
        .title("Puzzle")
        .resizable(false)  // Non-resizable (like TDialog)
        .build();

    // Puzzle view size also increased: was 17x4, now 18x5
    let puzzle_view = PuzzleView::new(Rect::new(1, 1, 19, 6));
    window.add(Box::new(puzzle_view));

    app.desktop.add(Box::new(window));
}

fn show_open_file_dialog(app: &mut Application) {
    let (width, height) = app.terminal.size();

    // Create centered file dialog
    let dialog_width = 60i16;
    let dialog_height = 18i16;
    let dialog_x = (width as i16 - dialog_width) / 2;
    let dialog_y = (height as i16 - dialog_height - 2) / 2;

    let mut file_dialog = FileDialogBuilder::new()
        .bounds(Rect::new(
            dialog_x,
            dialog_y,
            dialog_x + dialog_width,
            dialog_y + dialog_height,
        ))
        .title("Open File")
        .wildcard("*.*")
        .build();

    if let Some(path) = file_dialog.execute(app) {
        // Show selected file in a message (in a real app, would open the file)
        let msg = format!("Selected: {}", path.display());
        use turbo_vision::helpers::msgbox::{message_box, MF_INFORMATION, MF_OK_BUTTON};
        message_box(app, &msg, MF_INFORMATION | MF_OK_BUTTON);
    }
}

fn show_chdir_dialog(app: &mut Application) {
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;
    use std::rc::Rc;
    //use turbo_vision::core::command::CM_CANCEL;

    let (width, height) = app.terminal.size();

    // Create dialog - Matches Borland: TRect( 16, 2, 64, 21 )
    let dialog_width = 48i16;
    let dialog_height = 19i16;
    let dialog_x = (width as i16 - dialog_width) / 2;
    let dialog_y = (height as i16 - dialog_height - 2) / 2;

    let mut dialog = DialogBuilder::new()
        .bounds(Rect::new(
            dialog_x,
            dialog_y,
            dialog_x + dialog_width,
            dialog_y + dialog_height,
        ))
        .title("Change Directory")
        .build();

    // Directory name input - Matches Borland: TRect( 3, 3, 30, 4 )
    let dir_data = Rc::new(RefCell::new(String::new()));
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    *dir_data.borrow_mut() = current_dir.display().to_string();

    let dir_label = LabelBuilder::new()
        .bounds(Rect::new(2, 2, 18, 3))
        .text("Directory ~n~ame")
        .build();
    dialog.add(Box::new(dir_label));

    let dir_input = InputLineBuilder::new()
        .bounds(Rect::new(3, 3, dialog_width - 16, 4))
        .data(dir_data.clone())
        .max_length(255)
        .build();
    dialog.add(Box::new(dir_input));

    // Directory tree label - Matches Borland: TRect( 2, 5, ... )
    let tree_label = LabelBuilder::new()
        .bounds(Rect::new(2, 5, 18, 6))
        .text("Directory ~t~ree")
        .build();
    dialog.add(Box::new(tree_label));

    // Directory tree listbox - Matches Borland: TRect( 3, 6, 32, 16 )
    const CMD_DIR_SELECTED: u16 = 300;
    let mut dir_list = ListBoxBuilder::new()
        .bounds(Rect::new(3, 6, dialog_width - 16, dialog_height - 3))
        .on_select_command(CMD_DIR_SELECTED)
        .build();

    // Build directory tree
    let mut dir_items = Vec::new();
    if let Ok(current) = std::env::current_dir() {
        // Add parent directory (..)
        if current.parent().is_some() {
            dir_items.push("..".to_string());
        }

        // Add subdirectories
        if let Ok(entries) = fs::read_dir(&current) {
            let mut dirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            dirs.sort();

            // Format with tree characters (simplified)
            for (i, dir_name) in dirs.iter().enumerate() {
                let is_last = i == dirs.len() - 1;
                let prefix = if is_last { "└─ " } else { "├─ " };
                dir_items.push(format!("{}{}", prefix, dir_name));
            }
        }
    }

    dir_list.set_items(dir_items);
    dialog.add(Box::new(dir_list));

    // Buttons - Matches Borland: positioned on the right side
    let button_x = dialog_width - 13;

    // OK button - Matches Borland: TRect( 35, 6, 45, 8 )
    let ok_button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 6, button_x + 10, 8))
        .title("  ~O~K  ")
        .command(CM_OK)
        .default(true)
        .build();
    dialog.add(Box::new(ok_button));

    // ChDir button - Matches Borland: TRect( 35, 9, 45, 11 )
    const CM_CHDIR: u16 = 301;
    let chdir_button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 9, button_x + 10, 11))
        .title(" ~C~hdir ")
        .command(CM_CHDIR)
        .default(false)
        .build();
    dialog.add(Box::new(chdir_button));

    // Revert button - Matches Borland: TRect( 35, 12, 45, 14 )
    const CM_REVERT: u16 = 302;
    let revert_button = ButtonBuilder::new()
        .bounds(Rect::new(button_x, 12, button_x + 10, 14))
        .title(" ~R~evert")
        .command(CM_REVERT)
        .default(false)
        .build();
    dialog.add(Box::new(revert_button));

    dialog.set_initial_focus();

    // Execute dialog
    let command = dialog.execute(app);
    if command == CM_OK {
        let new_dir = dir_data.borrow().clone();
        if std::env::set_current_dir(&new_dir).is_ok() {
            let msg = format!("Changed to: {}", new_dir);
            use turbo_vision::helpers::msgbox::{message_box, MF_INFORMATION, MF_OK_BUTTON};
            message_box(app, &msg, MF_INFORMATION | MF_OK_BUTTON);
        } else {
            use turbo_vision::helpers::msgbox::{message_box, MF_ERROR, MF_OK_BUTTON};
            message_box(app, "Invalid directory", MF_ERROR | MF_OK_BUTTON);
        }
    }
}

fn main() -> turbo_vision::core::error::Result<()> {
    // Setup panic hook to log crashes
    std::panic::set_hook(Box::new(|panic_info| {
        use std::io::Write;
        let mut log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("crash.log")
            .unwrap();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        writeln!(log_file, "\n=== PANIC at timestamp {} ===", timestamp).unwrap();
        writeln!(log_file, "{}", panic_info).unwrap();

        if let Some(location) = panic_info.location() {
            writeln!(log_file, "Location: {}:{}:{}",
                location.file(), location.line(), location.column()).unwrap();
        }

        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            writeln!(log_file, "Message: {}", message).unwrap();
        } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            writeln!(log_file, "Message: {}", message).unwrap();
        }

        writeln!(log_file, "Backtrace:").unwrap();
        writeln!(log_file, "{:?}", std::backtrace::Backtrace::capture()).unwrap();
        writeln!(log_file, "=== END PANIC ===\n").unwrap();

        eprintln!("PANIC! Details written to crash.log");
    }));

    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    // Create menu bar
    let menu_bar = create_menu_bar(width);
    app.set_menu_bar(menu_bar);

    // Create status line
    let status_line = create_status_line(width, height);
    app.set_status_line(status_line);

    // Create clock view (right side of menu bar) - Matches Borland: tvdemo1.cc:128
    let clock_width = 9; // "HH:MM:SS" format + space
    let mut clock = ClockView::new(Rect::new(width as i16 - clock_width, 0, width as i16, 1));

    // Create heap/message view (right side of status bar) - Matches Borland: tvdemo1.cc:133
    let message = "Hello, World!";
    let msg_width = 13; // Fixed width like Borland's heap view
    let mut message_view = MessageView::new(
        Rect::new(
            width as i16 - msg_width,
            height as i16 - 1,
            width as i16,
            height as i16,
        ),
        message,
    );

    // Show about dialog on startup
    show_about_dialog(&mut app);

    // Main event loop
    app.running = true;
    while app.running {
        app.draw();

        // Draw clock and message views on top (like Borland's idle() update)
        // Matches Borland: tvdemo3.cc:173-174
        clock.draw(&mut app.terminal);
        message_view.draw(&mut app.terminal);

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
                    CM_ASCII_TABLE => show_ascii_table(&mut app),
                    CM_CALCULATOR => show_calculator_placeholder(&mut app),
                    CM_CALENDAR => show_calendar_placeholder(&mut app),
                    CM_PUZZLE => show_puzzle_placeholder(&mut app),
                    CM_OPEN => show_open_file_dialog(&mut app),
                    CM_CHDIR => show_chdir_dialog(&mut app),
                    CM_NEXT => {
                        // Cycle to next window (bring next window to front)
                        app.desktop.select_next();
                    }
                    CM_PREV => {
                        // Cycle to previous window (bring previous window to front)
                        app.desktop.select_prev();
                    }
                    CM_TILE => {
                        app.desktop.tile();
                    }
                    CM_CASCADE => {
                        app.desktop.cascade();
                    }
                    CM_ZOOM => {
                        // Zoom/restore the topmost window
                        // Matches Borland: Desktop handles cmZoom command
                        app.desktop.zoom_top_window();
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
