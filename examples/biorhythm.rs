// (C) 2025 - Enzo Lombardi
// Biorhythm Calculator - Working Demo
// Displays biorhythm charts with semi-graphical ASCII visualization

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use turbo_vision::app::Application;
use turbo_vision::core::command::{CM_CANCEL, CM_CLOSE, CM_OK, CM_QUIT};
use turbo_vision::core::draw::DrawBuffer;
use turbo_vision::core::event::{Event, EventType, KB_ALT_C, KB_ALT_X, KB_F1, KB_F10};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::core::palette::{Attr, TvColor, colors};
use turbo_vision::core::state::StateFlags;
use turbo_vision::core::state::{OF_CENTERED, SF_MODAL, SF_VISIBLE};
use turbo_vision::terminal::Terminal;
use turbo_vision::views::view::write_line_to_terminal;
use turbo_vision::views::{
    View,
    button::ButtonBuilder,
    dialog::DialogBuilder,
    input_line::InputLineBuilder,
    menu_bar::{MenuBar, SubMenu},
    static_text::StaticTextBuilder,
    status_line::{StatusItem, StatusLine},
    validator::RangeValidator,
};

// Custom commands
const CM_BIORHYTHM: u16 = 100;
const CM_ABOUT: u16 = 101;

/// Stores the birth date values
#[derive(Clone, Default)]
struct BirthDate {
    day: u32,
    month: u32,
    year: u32,
}

impl BirthDate {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Clone)]
struct Biorhythm {
    // Number of days alive is inherently non-negative, keep as u32
    days_alive: u32,
}

impl Biorhythm {
    fn new(days_alive: u32) -> Self {
        Self { days_alive }
    }

    // Biorhythm cycles (in days)
    const PHYSICAL_CYCLE: f64 = 23.0;
    const EMOTIONAL_CYCLE: f64 = 28.0;
    const INTELLECTUAL_CYCLE: f64 = 33.0;

    fn cycle_value(&self, offset: i32, period: f64) -> f64 {
        // days may be negative for past offsets
        let days = self.days_alive as i32 + offset;
        (2.0 * std::f64::consts::PI * days as f64 / period).sin()
    }

    // Accept signed offsets for plotting (past/future relative to today)
    fn physical(&self, day_offset: i32) -> f64 {
        self.cycle_value(day_offset, Self::PHYSICAL_CYCLE)
    }

    fn emotional(&self, day_offset: i32) -> f64 {
        self.cycle_value(day_offset, Self::EMOTIONAL_CYCLE)
    }

    fn intellectual(&self, day_offset: i32) -> f64 {
        self.cycle_value(day_offset, Self::INTELLECTUAL_CYCLE)
    }
}

struct BiorhythmChart {
    bounds: Rect,
    biorhythm: Arc<Mutex<Option<Biorhythm>>>, // TODO: why not Rc<RefCell<Option<Biorhythm>>> instead?
    state: StateFlags,
}

impl BiorhythmChart {
    fn new(bounds: Rect, biorhythm: Arc<Mutex<Option<Biorhythm>>>) -> Self {
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

        let bio_opt = self.biorhythm.lock().expect("Biorhythm mutex poisoned");

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

    fn get_palette(&self) -> Option<turbo_vision::core::palette::Palette> {
        None
    }
}

/// Simple date calculation functions (no external dependencies)
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(month: u32, year: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Calculate days since Jan 1, 1970 (Unix epoch)
fn days_since_epoch(year: u32, month: u32, day: u32) -> u32 {
    let mut days: u32 = 0;

    // Add days for complete years
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Add days for complete months in the current year
    for m in 1..month {
        days += days_in_month(m, year);
    }

    // Add remaining days
    days += day;

    days
}

fn get_current_date() -> (u32, u32, u32) {
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

/// Calculate days alive since the birth date. No need to validate.
fn is_in_future(birth_date: &BirthDate) -> Option<u32> {
    let (today_year, today_month, today_day) = get_current_date();
    let birth_days = days_since_epoch(birth_date.year, birth_date.month, birth_date.day);
    let today_days = days_since_epoch(today_year, today_month, today_day);

    if today_days < birth_days {
        None // Birth date is in the future
    } else {
        // Some(days_alive)
        Some(today_days - birth_days)
    }
}

// The D, M and Y fields use validator
fn create_biorhythm_dialog(state: &BirthDate) -> (turbo_vision::views::dialog::Dialog, Rc<RefCell<String>>, Rc<RefCell<String>>, Rc<RefCell<String>>) {
    // Dialog dimensions: 50 wide, 12 tall
    let dialog_width = 50i16;
    let dialog_height = 12i16;

    // Create dialog with dummy position - OF_CENTERED will auto-center it
    let mut dialog = DialogBuilder::new().bounds(Rect::new(0, 0, dialog_width, dialog_height)).title("Enter Birth Date").build();

    // Enable automatic centering (matches Borland's ofCentered option)
    dialog.set_options(OF_CENTERED);

    // Get today's date for display
    let (today_year, today_month, today_day) = get_current_date();

    dialog.add(Box::new(
        StaticTextBuilder::new()
            .bounds(Rect::new(2, 1, 46, 3))
            .text(&format!("Format: DD/MM/YYYY\nRange : 01/01/1900 - {}/{}/{}", today_day, today_month, today_year))
            .build(),
    ));

    // Labels
    dialog.add(Box::new(StaticTextBuilder::new().bounds(Rect::new(2, 4, 12, 5)).text("Day:").build()));
    dialog.add(Box::new(StaticTextBuilder::new().bounds(Rect::new(2, 5, 12, 6)).text("Month:").build()));
    dialog.add(Box::new(StaticTextBuilder::new().bounds(Rect::new(2, 6, 12, 7)).text("Year:").build()));

    // Convert u32/i32 to String for display
    let prev_day = if state.day > 0 { state.day.to_string() } else { String::new() };
    let prev_month = if state.month > 0 { state.month.to_string() } else { String::new() };
    let prev_year = if state.year > 0 { state.year.to_string() } else { String::new() };

    // Create shared data for input fields with initial values
    let day_data = Rc::new(RefCell::new(prev_day));
    let month_data = Rc::new(RefCell::new(prev_month));
    let year_data = Rc::new(RefCell::new(prev_year));

    // Input fields with validators
    // Day: [1-31]
    let day_validator = Rc::new(RefCell::new(RangeValidator::new(1, 31)));
    let mut day_input = InputLineBuilder::new().bounds(Rect::new(12, 4, 17, 5)).max_length(2).data(Rc::clone(&day_data)).build();
    day_input.set_validator(day_validator);
    dialog.add(Box::new(day_input));

    // Month: [1-12]
    let month_validator = Rc::new(RefCell::new(RangeValidator::new(1, 12)));
    let mut month_input = InputLineBuilder::new().bounds(Rect::new(12, 5, 17, 6)).max_length(2).data(Rc::clone(&month_data)).build();
    month_input.set_validator(month_validator);
    dialog.add(Box::new(month_input));

    // Year: [1900-2100]
    let year_validator = Rc::new(RefCell::new(RangeValidator::new(1900, 2100)));
    let mut year_input = InputLineBuilder::new().bounds(Rect::new(12, 6, 17, 7)).max_length(4).data(Rc::clone(&year_data)).build();
    year_input.set_validator(year_validator);
    dialog.add(Box::new(year_input));

    // Buttons (child indices 8 and 9)
    dialog.add(Box::new(ButtonBuilder::new().bounds(Rect::new(15, 8, 25, 10)).title("  OK  ").command(CM_OK).default(true).build()));
    dialog.add(Box::new(
        ButtonBuilder::new().bounds(Rect::new(27, 8, 37, 10)).title("Cancel").command(CM_CANCEL).default(false).build(),
    ));

    dialog.set_initial_focus();
    (dialog, day_data, month_data, year_data)
}

/// Extended validation of the birth date (beyond what the validators can do)
fn validate_birth_date(birth_date: &BirthDate) -> bool {
    // Check if the date is valid (accounting for month lengths and leap years).
    if birth_date.day > days_in_month(birth_date.month, birth_date.year) {
        return false;
    }

    // Calculate days alive and confirms the birth date in not set in the future
    is_in_future(birth_date).is_some()
}

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

/// Helper function to parse three strings into a BirthDate
/// Returns None if parsing fails
fn parse_birth_state(day_str: &str, month_str: &str, year_str: &str) -> Option<BirthDate> {
    if let (Ok(day), Ok(month), Ok(year)) = (day_str.parse::<u32>(), month_str.parse::<u32>(), year_str.parse::<u32>()) {
        Some(BirthDate { day, month, year })
    } else {
        None
    }
}

/// Runs a modal birth date dialog with validation and user input handling.
///
/// This function manages the entire lifecycle of the birth date input dialog:
/// - Creates and displays the dialog with input fields
/// - Handles real-time validation of user input
/// - Enables/disables the OK button based on validation state
/// - Processes events until the user confirms or cancels
/// - Parses the validated input strings into numeric date components
///
/// # Returns
/// - `Some(BirthDateState)` if the user confirmed with valid date data
/// - `None` if the user cancelled or if parsing failed (though parsing failure
///   should not occur due to the validators)
///
/// Note
/// Initially, run_modal_birth_date_dialog() returned Rc<RefCell<String>>
/// The Rc<RefCell<String>> wrappers were needed for sharing mutable data between the dialog's
/// input widgets and our code while the dialog was active.
/// Once the dialog closes, we no longer need shared ownership.
/// We just need the final values.
/// By cloning the strings and parsing them into numbers before returning, we transfer ownership of
/// simple value types instead of shared references, eliminating the need for reference counting entirely.
/// In short: Rc<RefCell<T>> enables sharing during the dialog's lifetime, but after the dialog closes,
/// we only care about the final values, not the shared containers.
fn run_modal_birth_date_dialog(app: &mut Application, state: &BirthDate) -> Option<BirthDate> {
    use std::time::Duration;
    use turbo_vision::core::command::CM_COMMAND_SET_CHANGED;
    use turbo_vision::core::command_set;

    // day_data is of type Rc<RefCell<String>>. See the InputLineBuilder() signature
    let (mut dialog, day_data, month_data, year_data) = create_biorhythm_dialog(state);

    // Set modal flag
    let old_state = dialog.state();
    dialog.set_state(old_state | SF_MODAL);

    // Initial validation and command state
    let is_valid = if let Some(parsed_state) = parse_birth_state(&day_data.borrow(), &month_data.borrow(), &year_data.borrow()) {
        validate_birth_date(&parsed_state)
    } else {
        false
    };

    // Enable/disable CM_OK command based on the previous validation
    if is_valid {
        command_set::enable_command(CM_OK);
    } else {
        command_set::disable_command(CM_OK);
    }

    // Broadcast the change to update button state globally
    let mut broadcast_event = Event::broadcast(CM_COMMAND_SET_CHANGED);
    dialog.handle_event(&mut broadcast_event);
    command_set::clear_command_set_changed();

    // Add dialog to desktop - this will center it automatically via OF_CENTERED
    app.desktop.add(Box::new(dialog));
    let dialog_index = app.desktop.child_count() - 1;

    let result;

    // Main dialog event loop
    loop {
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
            let is_valid = if let Some(parsed_state) = parse_birth_state(&day_data.borrow(), &month_data.borrow(), &year_data.borrow()) {
                validate_birth_date(&parsed_state)
            } else {
                false
            };

            // Enable/disable CM_OK command and broadcast change
            if is_valid {
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

        // Check if dialog should close (returns CM_OK or CM_CANCEL)
        let end_state = if let Some(dialog_view) = app.desktop.window_at_mut(dialog_index) {
            dialog_view.get_end_state()
        } else {
            0
        };
        if end_state != 0 {
            result = end_state;
            break;
        }
    }

    // Remove dialog from desktop
    app.desktop.remove_child(dialog_index);

    // Re-enable CM_OK command for future dialogs
    command_set::enable_command(CM_OK);

    // Process the result: parse strings to numbers if user confirmed
    if result == CM_OK {
        let day_str = day_data.borrow();
        let month_str = month_data.borrow();
        let year_str = year_data.borrow();

        // Parse the validated input strings into numeric values
        // This should always succeed thanks to the validators, but we handle it defensively
        // First try a trimmed parse (accept leading/trailing spaces)
        if let (Ok(day), Ok(month), Ok(year)) = (day_str.parse::<u32>(), month_str.parse::<u32>(), year_str.parse::<u32>()) {
            Some(BirthDate { day, month, year })
        } else {
            // Fallback: user may have pasted "DD MM YYYY" into one field - try to split combined input
            let combined = format!("{day_str} {month_str} {year_str}");
            let parts: Vec<&str> = combined.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(day), Ok(month), Ok(year)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
                    Some(BirthDate { day, month, year })
                } else {
                    None
                }
            } else {
                // Parsing failed - should not happen with proper validators
                None
            }
        }
    } else {
        // User cancelled
        None
    }
}

/// Processes a validated birth date by calculating the biorhythm and updating application state.
///
/// This function receives pre-validated and pre-parsed date components from the dialog,
/// calculates the number of days since birth, creates a biorhythm instance, and updates
/// the state for future dialog invocations.
///
/// # Arguments
/// * `biorhythm_data` - Shared biorhythm data to update
/// * `state` - Birth date state containing the validated date and to persist for future dialog invocations
///
/// # Returns
/// `true` if the biorhythm was successfully calculated and stored, `false` if the date
/// is invalid (e.g., in the future or otherwise impossible)
fn process_birth_date_result(biorhythm_data: &Arc<Mutex<Option<Biorhythm>>>, state: &BirthDate) -> bool {
    // Calculate days alive and confirms the birth date in not in the future
    if let Some(days_alive) = is_in_future(&state) {
        // Create and store the biorhythm data
        *biorhythm_data.lock().expect("Biorhythm mutex poisoned") = Some(Biorhythm::new(days_alive));
        true
    } else {
        // Date is invalid (e.g., in the future)
        false
    }
}

/// Handle command events - returns true if app should continue running
fn handle_command_event(command: u16, app: &mut Application, biorhythm_data: &Arc<Mutex<Option<Biorhythm>>>, birth_state: &mut BirthDate) -> bool {
    match command {
        CM_BIORHYTHM => {
            // Show the birth date dialog and process the result if user confirmed
            if let Some(new_state) = run_modal_birth_date_dialog(app, birth_state) {
                process_birth_date_result(biorhythm_data, &new_state);
                *birth_state = new_state;
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

/// Convert global keyboard shortcuts to command events
/// These shortcuts work regardless of whether menus are open or not
/// Note: ESC is NOT converted here - ESC_ESC it's handled by MenuBar to close menus
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

/// Create and configure the menu bar
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

/// Create and configure the status line at the bottom of the screen
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

/// Design the chart dialog box
fn add_chart(app: &mut Application, biorhythm_data: &Arc<Mutex<Option<Biorhythm>>>) {
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
    let chart = BiorhythmChart::new(Rect::new(1, 1, chart_width, chart_height), Arc::clone(&biorhythm_data));
    main_dialog.add(Box::new(chart));
    app.desktop.add(Box::new(main_dialog));
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    add_menu_bar(&mut app);
    add_status_line(&mut app);

    let biorhythm_data = Arc::new(Mutex::new(None));
    let initial_birth_date = BirthDate::new();

    // Displays the dialog box for entering the date of birth
    let birth_date_result = run_modal_birth_date_dialog(&mut app, &initial_birth_date);

    // If user cancelled, quit the app
    let Some(mut current_birth_state) = birth_date_result else {
        return Ok(());
    };

    // Process the birth date and update birth_state
    process_birth_date_result(&biorhythm_data, &current_birth_state);

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
                if !handle_command_event(event.command, &mut app, &biorhythm_data, &mut current_birth_state) {
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
