// (C) 2025 - Enzo Lombardi
// Biorhythm Calculator - Working Demo
// Displays biorhythm charts with semi-graphical ASCII visualization

use chrono::{Datelike, Local, NaiveDate};
use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_CLOSE, CM_CONTINUE, CM_OK, CM_QUIT};
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{Event, EventType, KB_ALT_C, KB_ALT_X, KB_F1, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::palette::{Attr, TvColor, colors};
use turbo_vision::core::state::{OF_CENTERED, SF_MODAL, SF_VISIBLE, StateFlags};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::View;
use turbo_vision::views::dialog::DialogBuilder;
use turbo_vision::views::menu_bar::{MenuBar, SubMenu};
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::validator::Validator;
use turbo_vision::views::view::write_line_to_terminal;

// Custom commands
const CM_BIORHYTHM: u16 = 100;
const CM_ABOUT: u16 = 101;

/// DateFieldValidator - validates numeric date field input (day, month, year)
/// Checks values during typing, not just characters
struct DateFieldValidator {
    min: i64,
    max: i64,
}

impl DateFieldValidator {
    fn new(min: i64, max: i64) -> Self {
        Self { min, max }
    }
}

impl Validator for DateFieldValidator {
    fn is_valid(&self, input: &str) -> bool {
        // Empty is invalid for final validation
        if input.is_empty() {
            return false;
        }

        // Try to parse as number
        match input.parse::<i64>() {
            Ok(value) => value >= self.min && value <= self.max,
            Err(_) => false,
        }
    }

    fn is_valid_input(&self, input: &str, _append: bool) -> bool {
        // Empty string is valid during typing
        if input.is_empty() {
            return true;
        }

        // Must be all digits
        if !input.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Try to parse the value
        match input.parse::<i64>() {
            Ok(value) => {
                // Reject if already above max
                if value > self.max {
                    return false;
                }

                // If already within range, check if adding more digits could exceed max
                if value >= self.min && value <= self.max {
                    // Check if we could still add more digits
                    let min_str_len = self.min.to_string().len();
                    let max_str_len = self.max.to_string().len();
                    let max_allowed_len = min_str_len.max(max_str_len);
                    let input_len = input.len();

                    // If we've reached max length, the value is final - accept it
                    if input_len >= max_allowed_len {
                        return true;
                    }

                    // Single digit values: accept them as they could be final
                    // User can press Tab/Enter to move on, or continue typing
                    // Example: "9" is valid for day (could be final) or month (could be final)
                    if value < 10 {
                        return true;
                    }

                    // Multi-digit values: check if adding more would exceed max
                    // Build minimum possible by appending one 0
                    let min_possible_str = format!("{}0", input);
                    if let Ok(min_possible) = min_possible_str.parse::<i64>() {
                        // If even adding a 0 exceeds max, reject
                        // Example: "20" -> "200" > 31, but "10" -> "100" > 31 too
                        if min_possible > self.max {
                            return false;
                        }
                    }

                    return true;
                }

                // Value is below minimum - check if it could become valid by adding more digits
                // Example: "1" could become "1999" for range [1900-2100]

                // Use the max of min and max string lengths to allow full range
                let min_str_len = self.min.to_string().len();
                let max_str_len = self.max.to_string().len();
                let max_allowed_len = min_str_len.max(max_str_len);
                let input_len = input.len();

                // If we've already reached max allowed length and still below min, reject
                if input_len >= max_allowed_len {
                    return false;
                }

                // Build the maximum possible number by appending 9s
                // For "1" with 3 remaining digits: "1999"
                // For "19" with 2 remaining digits: "1999"
                let mut max_possible_str = input.to_string();
                let remaining_digits = max_allowed_len - input_len;
                for _ in 0..remaining_digits {
                    max_possible_str.push('9');
                }

                // Parse and check if this could reach the valid range
                if let Ok(max_possible) = max_possible_str.parse::<i64>() {
                    max_possible >= self.min
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    fn error(&self) {
        // Error handling via visual feedback in InputLine
    }
}

#[derive(Clone)]
struct Biorhythm {
    days_alive: u32,
}

impl Biorhythm {
    /// Create a new biorhythm instance from the number of days alive
    fn new(days_alive: u32) -> Self {
        Self { days_alive }
    }

    // Biorhythm cycles (in days)
    const PHYSICAL_CYCLE: f64 = 23.0;
    const EMOTIONAL_CYCLE: f64 = 28.0;
    const INTELLECTUAL_CYCLE: f64 = 33.0;

    /// Calculate sine wave value for a given cycle period and day offset
    fn cycle_value(&self, offset: i32, period: f64) -> f64 {
        let days = self.days_alive as i32 + offset;
        (2.0 * std::f64::consts::PI * days as f64 / period).sin()
    }

    /// Calculate physical cycle value for a given day offset from today
    fn physical(&self, day_offset: i32) -> f64 {
        self.cycle_value(day_offset, Self::PHYSICAL_CYCLE)
    }

    /// Calculate emotional cycle value for a given day offset from today
    fn emotional(&self, day_offset: i32) -> f64 {
        self.cycle_value(day_offset, Self::EMOTIONAL_CYCLE)
    }

    /// Calculate intellectual cycle value for a given day offset from today
    fn intellectual(&self, day_offset: i32) -> f64 {
        self.cycle_value(day_offset, Self::INTELLECTUAL_CYCLE)
    }
}

struct BiorhythmChart {
    bounds: Rect,
    biorhythm: Rc<RefCell<Option<Biorhythm>>>,
    state: StateFlags,
}

impl BiorhythmChart {
    /// Create a new biorhythm chart view with the given bounds and shared data
    fn new(bounds: Rect, biorhythm: Rc<RefCell<Option<Biorhythm>>>) -> Self {
        Self { bounds, biorhythm, state: SF_VISIBLE }
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

        let bio_opt = self.biorhythm.borrow();

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
                    for x in 9..9 + chart_width {
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
                                if x != today_x {
                                    // Don't overwrite today marker
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
                    let msg = "Press Alt+C or F10 -> Biorhythm -> Calculate";
                    let msg_x = (width.saturating_sub(msg.len())) / 2;
                    buf.move_str(msg_x, msg, colors::DIALOG_NORMAL);
                }

                write_line_to_terminal(terminal, self.bounds.a.x, self.bounds.a.y + y as i16, &buf);
            }
        }
    }

    fn handle_event(&mut self, _event: &mut Event) {}
    fn update_cursor(&self, _terminal: &mut Terminal) {}

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

/// Create birth date input dialog with validators and return dialog + shared field data
fn create_biorhythm_dialog(birth_date: Option<&NaiveDate>) -> (turbo_vision::views::dialog::Dialog, Rc<RefCell<String>>, Rc<RefCell<String>>, Rc<RefCell<String>>) {
    use turbo_vision::views::{button::ButtonBuilder, input_line::InputLineBuilder, static_text::StaticTextBuilder};

    let dialog_width = 50i16;
    let dialog_height = 12i16;

    // Create dialog with dummy position - OF_CENTERED will auto-center it
    let mut dialog = DialogBuilder::new().bounds(Rect::new(0, 0, dialog_width, dialog_height)).title("Enter Birth Date").build();
    dialog.set_options(OF_CENTERED);

    // Get today's date for the displayed message
    let (today_year, today_month, today_day) = {
        let today = Local::now().date_naive();
        (today.year(), today.month(), today.day())
    };

    dialog.add(Box::new(
        StaticTextBuilder::new()
            .bounds(Rect::new(2, 1, 46, 3))
            .text(&format!("Format: DD/MM/YYYY\nRange : 01/01/1900 - {}/{}/{}", today_day, today_month, today_year))
            .build(),
    ));

    dialog.add(Box::new(StaticTextBuilder::new().bounds(Rect::new(2, 4, 12, 5)).text("Day:").build()));
    dialog.add(Box::new(StaticTextBuilder::new().bounds(Rect::new(2, 5, 12, 6)).text("Month:").build()));
    dialog.add(Box::new(StaticTextBuilder::new().bounds(Rect::new(2, 6, 12, 7)).text("Year:").build()));

    // Convert NaiveDate to String components to fill the input lines
    let (prev_day, prev_month, prev_year) = if let Some(date) = birth_date {
        (date.day().to_string(), date.month().to_string(), date.year().to_string())
    } else {
        (String::new(), String::new(), String::new())
    };

    // Create shared data for input fields with initial values
    let day_data = Rc::new(RefCell::new(prev_day));
    let month_data = Rc::new(RefCell::new(prev_month));
    let year_data = Rc::new(RefCell::new(prev_year));

    // Input fields with validators - Day: [1-31]
    let day_validator = Rc::new(RefCell::new(DateFieldValidator::new(1, 31)));
    let mut day_input = InputLineBuilder::new().bounds(Rect::new(12, 4, 17, 5)).max_length(2).data(Rc::clone(&day_data)).build();
    day_input.set_validator(day_validator);
    dialog.add(Box::new(day_input));

    // Month: [1-12]
    let month_validator = Rc::new(RefCell::new(DateFieldValidator::new(1, 12)));
    let mut month_input = InputLineBuilder::new().bounds(Rect::new(12, 5, 17, 6)).max_length(2).data(Rc::clone(&month_data)).build();
    month_input.set_validator(month_validator);
    dialog.add(Box::new(month_input));

    // Year: [1900-2100]
    let year_validator = Rc::new(RefCell::new(DateFieldValidator::new(1900, 2100)));
    let mut year_input = InputLineBuilder::new().bounds(Rect::new(12, 6, 17, 7)).max_length(4).data(Rc::clone(&year_data)).build();
    year_input.set_validator(year_validator);
    dialog.add(Box::new(year_input));

    // Buttons
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(15, 8, 25, 10)).title("  OK  ").command(CM_OK).default(true).build()));
    dialog.add(Box::new(
        ButtonBuilder::new().bounds(Rect::new(27, 8, 37, 10)).title("Cancel").command(CM_CANCEL).default(false).build(),
    ));

    dialog.set_initial_focus();
    (dialog, day_data, month_data, year_data)
}

/// Validate that birth date is not in the future and year is >= 1900
fn validate_birth_date(birth_date: &NaiveDate) -> bool {
    let today = Local::now().date_naive();
    birth_date.year() >= 1900 && *birth_date <= today
}

/// Display application information dialog with biorhythm cycle details
fn show_about_dialog(app: &mut Application) {
    use turbo_vision::helpers::msgbox::{MF_ABOUT, MF_OK_BUTTON, message_box};

    let message = r"Biorhythm Calculator v1.0

Read docs/BIORHYTHM-CALCULATOR-TUTORIAL.md

Calculates three cycles:
  • Physical (23 days)
  • Emotional (28 days)
  • Intellectual (33 days)

Semi-graphical ASCII chart";

    message_box(app, message, MF_ABOUT | MF_OK_BUTTON);
}

/// Parse day, month, year strings into NaiveDate (returns None if invalid)
fn parse_birth_date(day_str: &str, month_str: &str, year_str: &str) -> Option<NaiveDate> {
    if let (Ok(day), Ok(month), Ok(year)) = (day_str.parse::<u32>(), month_str.parse::<u32>(), year_str.parse::<i32>()) {
        NaiveDate::from_ymd_opt(year, month, day)
    } else {
        None
    }
}

/// Run modal birth date dialog. Returns a validated date if any
fn run_modal_birth_date_dialog(app: &mut Application, birth_date: Option<&NaiveDate>) -> Option<NaiveDate> {
    use std::time::Duration;
    use turbo_vision::core::command::CM_COMMAND_SET_CHANGED;
    use turbo_vision::core::command_set;

    let (mut dialog, day_data, month_data, year_data) = create_biorhythm_dialog(birth_date);

    // Set modal flag
    let old_state = dialog.state();
    dialog.set_state(old_state | SF_MODAL);

    // Initially DD MM and YYYY fields are empty so CM_OK is grayed
    command_set::disable_command(CM_OK);

    // Broadcast the change to update button state globally
    let mut broadcast_event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    dialog.handle_event(&mut broadcast_event);
    command_set::clear_command_set_changed();

    // Add dialog to desktop - this will center it automatically via OF_CENTERED
    app.desktop.add(Box::new(dialog));
    let dialog_index = app.desktop.child_count() - 1;

    // Cache the last validated date to avoid re-parsing on OK click
    let mut last_valid_date = None;

    // Main dialog event loop
    let result = loop {
        // Draw desktop (which includes the dialog as a child)
        app.desktop.draw(&mut app.terminal);
        // Get cursor position from the dialog through desktop
        if let Some(dialog_view) = app.desktop.window_at_mut(dialog_index) {
            dialog_view.update_cursor(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        // Poll for event
        if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)).ok().flatten() {
            //handle_global_shortcuts(&mut event);

            // Handle modal-specific shortcuts
            if event.what == EventType::Keyboard {
                match event.key_code {
                    // KB_F1 => {
                    //     // Show help without converting to command (to avoid interference)
                    //     show_about_dialog(app);
                    //     continue; // Skip rest of event processing
                    // }
                    // KB_ALT_X | KB_CTRL_C | KB_ESC_ESC => {
                    KB_ALT_X => {
                        // Convert quit shortcuts to CM_CANCEL in modal context
                        event = Event::command(CM_CANCEL);
                    }
                    _ => {}
                }
            }

            // Handle the event through the desktop child
            if let Some(dialog_view) = app.desktop.window_at_mut(dialog_index) {
                dialog_view.handle_event(&mut event);

                // If event was converted to command, process it again
                if event.what == EventType::Command {
                    dialog_view.handle_event(&mut event);
                }
            }

            // After every event, revalidate and update command state
            // Parse and validate the current input, caching the result if valid
            last_valid_date = parse_birth_date(&day_data.borrow(), &month_data.borrow(), &year_data.borrow()).filter(|date| validate_birth_date(date));

            // Enable/disable CM_OK command and broadcast change
            if last_valid_date.is_some() {
                command_set::enable_command(CM_OK);
            } else {
                command_set::disable_command(CM_OK);
            }

            // Broadcast to update button state if command set changed
            if command_set::command_set_changed() {
                let mut broadcast_event = Event::broadcast(CM_COMMAND_SET_CHANGED);
                if let Some(dialog_view) = app.desktop.window_at_mut(dialog_index) {
                    dialog_view.handle_event(&mut broadcast_event);
                }
                command_set::clear_command_set_changed();
            }
        }

        // Check if dialog should close (returns CM_OK, CM_CANCEL, or CM_CONTINUE)
        let end_state = if let Some(dialog_view) = app.desktop.window_at_mut(dialog_index) {
            dialog_view.get_end_state()
        } else {
            break None; // Dialog disappeared = cancellation
        };

        match end_state {
            CM_CONTINUE => continue,        // Dialog still running, continue loop
            CM_OK => break last_valid_date, // Return validated date
            _ => break None,                // Any other command (CM_CANCEL, etc.) = cancellation
        }
    };

    // Remove dialog from desktop
    app.desktop.remove_child(dialog_index);

    // Re-enable CM_OK command for future dialogs
    command_set::enable_command(CM_OK);

    // Return result (already contains the correct Option<NaiveDate>)
    // No need to re-parse: last_valid_date was cached during validation
    result
}

/// Calculate days alive from birth date and update shared biorhythm data
fn process_birth_date_result(biorhythm_data: &Rc<RefCell<Option<Biorhythm>>>, birth_date: &NaiveDate) {
    let today = Local::now().date_naive();
    let days_alive = (today - *birth_date).num_days().try_into().unwrap();
    *biorhythm_data.borrow_mut() = Some(Biorhythm::new(days_alive));
}

/// Process command events and return whether the application should continue running
fn handle_command_event(command: u16, app: &mut Application, biorhythm_data: &Rc<RefCell<Option<Biorhythm>>>, birth_date: &mut Option<NaiveDate>) -> bool {
    match command {
        CM_BIORHYTHM => {
            // Show the birth date dialog and process the result if user confirmed
            if let Some(new_birth_date) = run_modal_birth_date_dialog(app, birth_date.as_ref()) {
                process_birth_date_result(biorhythm_data, &new_birth_date);
                *birth_date = Some(new_birth_date);
            }
            true
        }
        CM_ABOUT => {
            show_about_dialog(app);
            true
        }
        CM_CLOSE | CM_QUIT => false,
        _ => true,
    }
}

/// Convert global keyboard shortcuts (Alt+C, Alt+X, F1) to command events
fn handle_global_shortcuts(event: &mut Event) {
    if event.what != EventType::Keyboard {
        return;
    }

    let command = match event.key_code {
        KB_ALT_C => Some(CM_BIORHYTHM),
        KB_ALT_X => Some(CM_QUIT),
        KB_F1 => Some(CM_ABOUT),
        _ => None,
    };

    if let Some(cmd) = command {
        *event = Event::command(cmd);
    }
}

/// Create menu bar with Biorhythm and Help menus
fn add_menu_bar(app: &mut Application) {
    let (width, _) = app.terminal.size();
    let mut menu_bar = MenuBar::new(Rect::new(0, 0, width as i16, 1));
    let biorhythm_menu = Menu::from_items(vec![
        MenuItem::with_shortcut("~C~alculate", CM_BIORHYTHM, 0, "Alt+C", 0),
        MenuItem::separator(),
        MenuItem::with_shortcut("E~x~it", CM_QUIT, 0, "Alt+X", 0),
    ]);
    let help_menu = Menu::from_items(vec![MenuItem::with_shortcut("~A~bout", CM_ABOUT, 0, "F1", 0)]);
    menu_bar.add_submenu(SubMenu::new("~B~iorhythm", biorhythm_menu));
    menu_bar.add_submenu(SubMenu::new("~H~elp", help_menu));
    app.set_menu_bar(menu_bar);
}

/// Create status line with F1/F10/Alt-X shortcuts at bottom of screen
fn add_status_line(app: &mut Application) {
    let (width, height) = app.terminal.size();

    let status_line = StatusLine::new(
        Rect::new(0, height as i16 - 1, width as i16, height as i16),
        vec![
            StatusItem::new("~F1~ Help", KB_F1, CM_ABOUT),
            StatusItem::new("~F10~ Menu", KB_F10, 0),
            StatusItem::new("~Alt-X~ Exit", 0x2D00, CM_QUIT),
        ],
    );
    app.set_status_line(status_line);
}

/// Create and add centered biorhythm chart window to the desktop
fn add_chart(app: &mut Application, biorhythm_data: &Rc<RefCell<Option<Biorhythm>>>) {
    // Calculate window dimensions
    let (width, height) = app.terminal.size();
    let window_width = 76i16; // TODO should NOT be hard coded
    let available_width = width as i16;
    let available_height = height as i16 - 3; // Subtract menu bar, status line and shadow
    let margin_vertical: i16 = 1;
    let window_height = available_height - (margin_vertical * 2);
    let window_x = (available_width - (window_width + 2)) / 2;
    let window_y = margin_vertical;

    // Create and show the main dialog with chart
    let mut main_dialog = DialogBuilder::new()
        .bounds(Rect::new(window_x, window_y, window_x + window_width, window_y + window_height))
        .title("Biorhythm Calculator")
        .build();

    let chart_width = window_width - 2;
    let chart_height = window_height - 2;
    let chart = BiorhythmChart::new(Rect::new(1, 1, chart_width, chart_height), Rc::clone(&biorhythm_data));
    main_dialog.add(Box::new(chart));
    app.desktop.add(Box::new(main_dialog));
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    add_menu_bar(&mut app);
    add_status_line(&mut app);

    let biorhythm_data = Rc::new(RefCell::new(None));

    // Displays the dialog box for entering the date of birth
    let birth_date_result = run_modal_birth_date_dialog(&mut app, None);

    // If user cancelled, quit the app
    let Some(birth_date) = birth_date_result else {
        return Ok(());
    };

    // Process the birth date and update birth_state
    process_birth_date_result(&biorhythm_data, &birth_date);
    let mut current_birth_date = Some(birth_date);

    add_chart(&mut app, &biorhythm_data);

    // Main event loop
    app.running = true;

    while app.running {
        app.draw();
        app.terminal.flush()?;

        if let Ok(Some(mut event)) = app.terminal.poll_event(std::time::Duration::from_millis(50)) {
            // Order matters (very first)
            // Convert global keyboard shortcuts to commands so that F1, Ctrl+N etc. work even when menus are closed
            handle_global_shortcuts(&mut event);

            // Let menu bar handle events first
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

            // Let status line handle events
            if let Some(ref mut status_line) = app.status_line {
                status_line.handle_event(&mut event);
            }

            // Let desktop handle events
            app.desktop.handle_event(&mut event);

            // Handle custom commands
            if event.what == EventType::Command {
                if !handle_command_event(event.command, &mut app, &biorhythm_data, &mut current_birth_date) {
                    app.running = false;
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
