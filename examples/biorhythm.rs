// (C) 2025 - Enzo Lombardi
// Biorhythm Calculator - Working Demo
// Displays biorhythm charts with semi-graphical ASCII visualization

use turbo_vision::app::Application;
use turbo_vision::views::{
    dialog::Dialog,
    button::Button,
    static_text::StaticText,
    input_line::InputLine,
    menu_bar::{MenuBar, SubMenu},
    status_line::{StatusLine, StatusItem},
    View,
    validator::RangeValidator,
};
use turbo_vision::core::command::{CM_QUIT, CM_OK, CM_CANCEL, CM_CLOSE, CommandId};
use turbo_vision::core::event::{Event, EventType, KB_F1, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::state::*;
use turbo_vision::core::palette::{colors, Attr, TvColor};
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

// Custom commands
const CM_BIORHYTHM: u16 = 100;
const CM_ABOUT: u16 = 101;

// Biorhythm cycles (in days)
const PHYSICAL_CYCLE: f64 = 23.0;
const EMOTIONAL_CYCLE: f64 = 28.0;
const INTELLECTUAL_CYCLE: f64 = 33.0;

// Simple date calculation functions (no external dependencies)
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(month: u32, year: i32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 0,
    }
}

fn days_since_epoch(year: i32, month: u32, day: u32) -> i32 {
    // Calculate days since Jan 1, 1970 (Unix epoch)
    let mut days = 0;

    // Add days for complete years
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Add days for complete months in the current year
    for m in 1..month {
        days += days_in_month(m, year) as i32;
    }

    // Add remaining days
    days += day as i32;

    days
}

fn get_current_date() -> (i32, u32, u32) {
    // Get current date from system time
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let total_days = duration.as_secs() / 86400;

    // Simple algorithm to convert days since epoch to Y/M/D
    let mut days_left = total_days as i32;
    let mut year = 1970;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days_left >= days_in_year {
            days_left -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let mut month = 1;
    while month <= 12 {
        let days_in_current_month = days_in_month(month, year) as i32;
        if days_left >= days_in_current_month {
            days_left -= days_in_current_month;
            month += 1;
        } else {
            break;
        }
    }

    let day = days_left + 1;
    (year, month, day as u32)
}

fn calculate_days_alive(birth_year: i32, birth_month: u32, birth_day: u32) -> Option<i32> {
    // Validate date
    if birth_month < 1 || birth_month > 12 || birth_day < 1 {
        return None;
    }
    if birth_day > days_in_month(birth_month, birth_year) {
        return None;
    }

    let (today_year, today_month, today_day) = get_current_date();
    let birth_days = days_since_epoch(birth_year, birth_month, birth_day);
    let today_days = days_since_epoch(today_year, today_month, today_day);

    let days_alive = today_days - birth_days;
    if days_alive < 0 {
        None  // Birth date is in the future
    } else {
        Some(days_alive)
    }
}

#[derive(Clone)]
struct Biorhythm {
    days_alive: i32,
}

impl Biorhythm {
    fn new(days_alive: i32) -> Self {
        Self { days_alive }
    }

    fn physical(&self, day_offset: i32) -> f64 {
        let days = self.days_alive + day_offset;
        (2.0 * PI * days as f64 / PHYSICAL_CYCLE).sin()
    }

    fn emotional(&self, day_offset: i32) -> f64 {
        let days = self.days_alive + day_offset;
        (2.0 * PI * days as f64 / EMOTIONAL_CYCLE).sin()
    }

    fn intellectual(&self, day_offset: i32) -> f64 {
        let days = self.days_alive + day_offset;
        (2.0 * PI * days as f64 / INTELLECTUAL_CYCLE).sin()
    }
}

struct BiorhythmChart {
    bounds: Rect,
    biorhythm: Arc<Mutex<Option<Biorhythm>>>,
    state: StateFlags,
}

impl BiorhythmChart {
    fn new(bounds: Rect, biorhythm: Arc<Mutex<Option<Biorhythm>>>) -> Self {
        Self {
            bounds,
            biorhythm,
            state: SF_VISIBLE,
        }
    }
}

impl View for BiorhythmChart {
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
        if (self.state & SF_VISIBLE) == 0 {
            return;
        }

        let width = self.bounds.width() as usize;
        let height = self.bounds.height() as usize;

        if width < 10 || height < 10 {
            return;
        }

        let bio_opt = self.biorhythm.lock().unwrap();

        if let Some(ref bio) = *bio_opt {
            // Draw title
            let mut buf = DrawBuffer::new(width);
            buf.move_char(0, ' ', colors::DIALOG_NORMAL, width);
            let title = format!("Biorhythm Chart - {} days since birth", bio.days_alive);
            let title_start = (width.saturating_sub(title.len())) / 2;
            buf.move_str(title_start, &title, colors::DIALOG_FRAME);
            write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y, &buf);

            // Chart dimensions
            let chart_top = 2;
            let chart_height = height.saturating_sub(4);
            let chart_width = width.saturating_sub(10);
            let center_y = chart_top + chart_height / 2;

            // Draw each line of the chart
            for y in 1..height {
                let mut line = DrawBuffer::new(width);
                line.move_char(0, ' ', colors::DIALOG_NORMAL, width);

                if y == chart_top {
                    // Top Y-axis label
                    line.move_str(4, "+1.0", colors::DIALOG_NORMAL);
                } else if y == center_y {
                    // Center Y-axis label and horizontal line
                    line.move_str(4, " 0.0", colors::DIALOG_NORMAL);
                    for x in 9..9+chart_width {
                        line.move_char(x, '-', colors::DIALOG_NORMAL, 1);
                    }
                } else if y == chart_top + chart_height {
                    // Bottom Y-axis label
                    line.move_str(4, "-1.0", colors::DIALOG_NORMAL);
                } else if y >= chart_top && y < chart_top + chart_height {
                    // Chart area - draw biorhythm curves
                    let days_range = chart_width.min(60);
                    let start_offset = -(days_range as i32 / 2);
                    let today_x = 9 + days_range / 2;

                    // Initialize with today marker
                    line.move_char(today_x, '|', colors::MENU_SHORTCUT, 1);

                    // Draw cycles with colored blocks and different scaling factors
                    let cycles: [(char, Attr, f64, fn(&Biorhythm, i32) -> f64); 3] = [
                        // Physical: scale by 1.1 to extend range (taller peaks/troughs)
                        ('■', Attr::new(TvColor::Red, TvColor::LightGray), 0.9, |b, d| b.physical(d)),
                        // Emotional: scale by 0.9 to compress range (shorter peaks/troughs)
                        ('■', Attr::new(TvColor::Green, TvColor::LightGray), 1.0, |b, d| b.emotional(d)),
                        // Intellectual: normal scaling
                        ('■', Attr::new(TvColor::Blue, TvColor::LightGray), 0.8, |b, d| b.intellectual(d)),
                    ];

                    for (symbol, color, scale_factor, calc_fn) in &cycles {
                        for i in 0..days_range {
                            let day_offset = start_offset + i as i32;
                            let value = calc_fn(bio, day_offset);
                            let y_offset = (-value * (chart_height as f64 / 2.0) * scale_factor) as i32;
                            let target_y = (center_y as i32 + y_offset) as usize;

                            if target_y == y {
                                let x = 9 + i;
                                if x != today_x {  // Don't overwrite today marker
                                    line.move_char(x, *symbol, *color, 1);
                                }
                            }
                        }
                    }
                }

                // Legend on last line
                if y == height - 1 {
                    line.move_char(2, '■', Attr::new(TvColor::Red, TvColor::LightGray), 1);
                    line.move_str(3, ":Physical(23d) ", colors::DIALOG_NORMAL);
                    line.move_char(19, '■', Attr::new(TvColor::Green, TvColor::LightGray), 1);
                    line.move_str(20, ":Emotional(28d) ", colors::DIALOG_NORMAL);
                    line.move_char(37, '■', Attr::new(TvColor::Blue, TvColor::LightGray), 1);
                    line.move_str(38, ":Intellectual(33d) ", colors::DIALOG_NORMAL);
                    line.move_char(58, '|', colors::MENU_SHORTCUT, 1);
                    line.move_str(59, ":Today", colors::DIALOG_NORMAL);
                }

                write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &line);
            }
        } else {
            // No data - show prompt
            for y in 0..height {
                let mut buf = DrawBuffer::new(width);
                buf.move_char(0, ' ', colors::DIALOG_NORMAL, width);

                if y == height / 2 {
                    let msg = "Press F10 -> Biorhythm -> Calculate";
                    let msg_x = (width.saturating_sub(msg.len())) / 2;
                    buf.move_str(msg_x, msg, colors::DIALOG_NORMAL);
                }

                write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &buf);
            }
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {}
    fn update_cursor(&self, _terminal: &mut Terminal) {}
}

/// Validate the complete birth date
fn validate_birth_date(day_str: &str, month_str: &str, year_str: &str) -> bool {
    // Check for empty fields
    if day_str.trim().is_empty() || month_str.trim().is_empty() || year_str.trim().is_empty() {
        return false;
    }

    // Parse the values
    let day_result = day_str.parse::<u32>();
    let month_result = month_str.parse::<u32>();
    let year_result = year_str.parse::<i32>();

    // Check if all fields parse successfully
    let (Ok(day), Ok(month), Ok(year)) = (day_result, month_result, year_result) else {
        return false;
    };

    // Check basic ranges
    if day < 1 || day > 31 || month < 1 || month > 12 || year < 1900 || year > 2100 {
        return false;
    }

    // Check if the date is valid (accounting for month lengths and leap years)
    if day > days_in_month(month, year) {
        return false;
    }

    // Check if date is not in the future
    calculate_days_alive(year, month, day).is_some()
}

/// Custom dialog that validates birthdate fields using the View::valid() method
/// Demonstrates Borland TV's validation pattern where Dialog::valid() checks
/// all input fields before allowing the dialog to close
struct BirthdateDialog {
    dialog: Dialog,
    day_data: Rc<RefCell<String>>,
    month_data: Rc<RefCell<String>>,
    year_data: Rc<RefCell<String>>,
}

impl BirthdateDialog {
    fn new(prev_day: &str, prev_month: &str, prev_year: &str) -> Self {
        // Dialog dimensions: 50 wide, 12 tall
        let dialog_width = 50i16;
        let dialog_height = 12i16;

        // Create dialog with dummy position - OF_CENTERED will auto-center it
        let mut dialog = Dialog::new(
            Rect::new(0, 0, dialog_width, dialog_height),
            "Enter Birth Date"
        );

        // Enable automatic centering (matches Borland's ofCentered option)
        dialog.set_options(OF_CENTERED);

        // Get today's date for display
        let (today_year, today_month, today_day) = get_current_date();

        dialog.add(Box::new(StaticText::new(
            Rect::new(2, 2, 46, 4),
            &format!("Enter your birth date (Today: {}/{}/{})", today_day, today_month, today_year),
        )));

        // Labels
        dialog.add(Box::new(StaticText::new(Rect::new(2, 4, 12, 5), "Day:")));
        dialog.add(Box::new(StaticText::new(Rect::new(2, 5, 12, 6), "Month:")));
        dialog.add(Box::new(StaticText::new(Rect::new(2, 6, 12, 7), "Year:")));

        // Create shared data for input fields with initial values
        let day_data = Rc::new(RefCell::new(prev_day.to_string()));
        let month_data = Rc::new(RefCell::new(prev_month.to_string()));
        let year_data = Rc::new(RefCell::new(prev_year.to_string()));

        // Input fields with validators and OF_VALIDATE flag
        // Day: 1-31
        let day_validator = Rc::new(RefCell::new(RangeValidator::new(1, 31)));
        let mut day_input = InputLine::new(Rect::new(12, 4, 18, 5), 2, Rc::clone(&day_data));
        day_input.set_validator(day_validator);
        day_input.set_options(OF_VALIDATE);  // Mark for validation on focus release
        dialog.add(Box::new(day_input));

        // Month: 1-12
        let month_validator = Rc::new(RefCell::new(RangeValidator::new(1, 12)));
        let mut month_input = InputLine::new(Rect::new(12, 5, 18, 6), 2, Rc::clone(&month_data));
        month_input.set_validator(month_validator);
        month_input.set_options(OF_VALIDATE);  // Mark for validation on focus release
        dialog.add(Box::new(month_input));

        // Year: 1900-2100
        let year_validator = Rc::new(RefCell::new(RangeValidator::new(1900, 2100)));
        let mut year_input = InputLine::new(Rect::new(12, 6, 20, 7), 4, Rc::clone(&year_data));
        year_input.set_validator(year_validator);
        year_input.set_options(OF_VALIDATE);  // Mark for validation on focus release
        dialog.add(Box::new(year_input));

        // Buttons
        dialog.add(Box::new(Button::new(Rect::new(15, 8, 25, 10), "  OK  ", CM_OK, true)));
        dialog.add(Box::new(Button::new(Rect::new(27, 8, 37, 10), "Cancel", CM_CANCEL, false)));

        dialog.set_initial_focus();

        Self {
            dialog,
            day_data,
            month_data,
            year_data,
        }
    }

    fn get_values(&self) -> (String, String, String) {
        (
            self.day_data.borrow().clone(),
            self.month_data.borrow().clone(),
            self.year_data.borrow().clone(),
        )
    }

    fn execute(&mut self, app: &mut Application) -> CommandId {
        self.dialog.execute(app)
    }
}

impl View for BirthdateDialog {
    fn bounds(&self) -> Rect {
        self.dialog.bounds()
    }

    fn set_bounds(&mut self, bounds: Rect) {
        self.dialog.set_bounds(bounds);
    }

    fn draw(&mut self, terminal: &mut Terminal) {
        self.dialog.draw(terminal);
    }

    fn update_cursor(&self, terminal: &mut Terminal) {
        self.dialog.update_cursor(terminal);
    }

    fn handle_event(&mut self, event: &mut Event) {
        self.dialog.handle_event(event);
    }

    fn state(&self) -> StateFlags {
        self.dialog.state()
    }

    fn set_state(&mut self, state: StateFlags) {
        self.dialog.set_state(state);
    }

    fn options(&self) -> u16 {
        self.dialog.options()
    }

    fn set_options(&mut self, options: u16) {
        self.dialog.set_options(options);
    }

    fn get_end_state(&self) -> CommandId {
        self.dialog.get_end_state()
    }

    fn set_end_state(&mut self, command: CommandId) {
        self.dialog.set_end_state(command);
    }

    /// Validate the complete birthdate before allowing dialog to close
    /// Matches Borland: TDialog::valid() - validates all input fields
    /// This is called automatically by Dialog::handle_event() when closing
    fn valid(&mut self, command: CommandId) -> bool {
        // Always allow cancel
        if command == CM_CANCEL {
            return true;
        }

        // For other commands (like CM_OK), validate the complete birthdate
        validate_birth_date(
            &self.day_data.borrow(),
            &self.month_data.borrow(),
            &self.year_data.borrow(),
        )
    }
}

fn create_about_dialog() -> Dialog {
    let mut dialog = Dialog::new(Rect::new(18, 6, 62, 17), "About Biorhythm");

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 2, 40, 8),
        "Biorhythm Calculator v1.0\n\
         \n\
         Calculates three cycles:\n\
         • Physical (23 days)\n\
         • Emotional (28 days)\n\
         • Intellectual (33 days)\n\
         \n\
         Semi-graphical ASCII chart",
    )));

    dialog.add(Box::new(Button::new(Rect::new(15, 8, 25, 10), "  OK  ", CM_OK, true)));
    dialog.set_initial_focus();
    dialog
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    let biorhythm_data = Arc::new(Mutex::new(None));  // Start with no data

    // Store previous birth date values (default: empty - will prompt on startup)
    let mut prev_day = String::from("");
    let mut prev_month = String::from("");
    let mut prev_year = String::from("");

    // Menu bar
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));
    let biorhythm_menu = Menu::from_items(vec![
        MenuItem::with_shortcut("~C~alculate", CM_BIORHYTHM, 0, "Alt+C", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ]);
    let help_menu = Menu::from_items(vec![
        MenuItem::with_shortcut("~A~bout", CM_ABOUT, 0, "F1", 0),
    ]);
    menu_bar.add_submenu(SubMenu::new("~B~iorhythm", biorhythm_menu));
    menu_bar.add_submenu(SubMenu::new("~H~elp", help_menu));
    app.set_menu_bar(menu_bar);

    // Status line
    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F1~ Help", KB_F1, CM_ABOUT),
            StatusItem::new("~F10~ Menu", KB_F10, 0),
            StatusItem::new("~Alt-X~ Exit", 0x2D00, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);

    // Calculate window size: fixed width, maximum height
    // Account for menu bar (1 row), status line (1 row), and shadow (2 cols, 1 row)
    let window_width = 76i16;  // Fixed width for optimal chart readability
    let available_width = width as i16;
    let available_height = height as i16 - 2;  // Subtract menu bar and status line

    // Use maximum available height with small top/bottom margins
    let margin_vertical = 1i16;    // Leave 1 row top and bottom
    let window_height = available_height - (margin_vertical * 2) - 1;  // -1 for shadow

    // Center the window horizontally, position vertically with margin
    let window_x = (available_width - (window_width + 2)) / 2;
    let window_y = 1 + margin_vertical;  // 1 for menu bar + vertical margin

    // Show birthdate dialog at startup using the new validation pattern
    let mut dialog = BirthdateDialog::new(&prev_day, &prev_month, &prev_year);
    let result = dialog.execute(&mut app);

    // If user canceled, quit the app
    if result == CM_CANCEL {
        return Ok(());
    }

    // User clicked OK - parse and set the birthdate
    if result == CM_OK {
        let (day_str, month_str, year_str) = dialog.get_values();

        if let (Ok(day), Ok(month), Ok(year)) = (
            day_str.parse::<u32>(),
            month_str.parse::<u32>(),
            year_str.parse::<i32>(),
        ) {
            if let Some(days_alive) = calculate_days_alive(year, month, day) {
                *biorhythm_data.lock().unwrap() = Some(Biorhythm::new(days_alive));
                // Update previous values for next time
                prev_day = day_str;
                prev_month = month_str;
                prev_year = year_str;
            }
        }
    }

    // Now create and show the main dialog with chart - sized to available space
    let mut main_dialog = Dialog::new(
        Rect::new(window_x, window_y, window_x + window_width, window_y + window_height),
        "Biorhythm Calculator"
    );

    // Chart uses interior space (dialog minus frame), with 1-column margins
    let chart_width = window_width - 2;   // Subtract frame (2 chars total)
    let chart_height = window_height - 2; // Subtract frame (2 chars total)
    let chart = BiorhythmChart::new(
        Rect::new(1, 1, chart_width, chart_height),
        Arc::clone(&biorhythm_data)
    );
    main_dialog.add(Box::new(chart));
    app.desktop.add(Box::new(main_dialog));

    // Custom event handler loop
    app.running = true;

    while app.running {
        // Draw every frame since chart data can change
        app.draw();
        app.terminal.flush()?;

        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            // Let menu bar handle events first (including F10)
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

            // Let status line handle events (keyboard shortcuts)
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Let desktop handle events (window close, etc.)
            app.desktop.handle_event(&mut event);

            // Handle custom commands after UI components have processed events
            if event.what == EventType::Command {
                match event.command {
                    CM_BIORHYTHM => {
                        // Show birthdate dialog using the new validation pattern
                        let mut dialog = BirthdateDialog::new(&prev_day, &prev_month, &prev_year);
                        let result = dialog.execute(&mut app);

                        if result == CM_OK {
                            let (day_str, month_str, year_str) = dialog.get_values();

                            if let (Ok(day), Ok(month), Ok(year)) = (
                                day_str.parse::<u32>(),
                                month_str.parse::<u32>(),
                                year_str.parse::<i32>(),
                            ) {
                                if let Some(days_alive) = calculate_days_alive(year, month, day) {
                                    *biorhythm_data.lock().unwrap() = Some(Biorhythm::new(days_alive));
                                    // Update previous values for next time
                                    prev_day = day_str;
                                    prev_month = month_str;
                                    prev_year = year_str;
                                }
                            }
                        }
                        continue;
                    }
                    CM_ABOUT => {
                        let mut dialog = create_about_dialog();
                        dialog.execute(&mut app);
                        continue;
                    }
                    CM_CLOSE => {
                        // Main window close button clicked - exit application
                        app.running = false;
                    }
                    CM_QUIT => {
                        app.running = false;
                    }
                    _ => {}
                }
            }
        }

        app.idle();
        app.desktop.remove_closed_windows();
        app.desktop.handle_moved_windows(&mut app.terminal);

        // Exit if all windows are closed
        if app.desktop.child_count() == 0 {
            app.running = false;
        }
    }

    Ok(())
}
