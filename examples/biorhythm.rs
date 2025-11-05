// Biorhythm Calculator - Working Demo
// Displays biorhythm charts with semi-graphical ASCII visualization

use turbo_vision::app::Application;
use turbo_vision::views::{
    dialog::Dialog,
    button::Button,
    static_text::StaticText,
    menu_bar::{MenuBar, SubMenu},
    status_line::{StatusLine, StatusItem},
    window::Window,
    View,
};
use turbo_vision::core::command::{CM_QUIT, CM_OK, CM_CANCEL, CM_CLOSE};
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

// Custom commands
const CM_BIORHYTHM: u16 = 100;
const CM_ABOUT: u16 = 101;

// Biorhythm cycles (in days)
const PHYSICAL_CYCLE: f64 = 23.0;
const EMOTIONAL_CYCLE: f64 = 28.0;
const INTELLECTUAL_CYCLE: f64 = 33.0;

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

fn create_biorhythm_dialog() -> Dialog {
    let mut dialog = Dialog::new(Rect::new(15, 7, 65, 17), "Calculate Biorhythm");

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 2, 46, 6),
        "Choose age (simplified demo):\n\
         \n\
         • Born 5,000 days ago (~14 years)\n\
         • Born 10,000 days ago (~27 years)\n\
         • Born 15,000 days ago (~41 years)",
    )));

    dialog.add(Box::new(Button::new(Rect::new(5, 7, 20, 9), " 5,000 days", 100, true)));
    dialog.add(Box::new(Button::new(Rect::new(22, 7, 37, 9), "10,000 days", 101, false)));
    dialog.add(Box::new(Button::new(Rect::new(5, 10, 20, 12), "15,000 days", 102, false)));
    dialog.add(Box::new(Button::new(Rect::new(22, 10, 37, 12), "  Cancel  ", CM_CANCEL, false)));

    dialog.set_initial_focus();
    dialog
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

fn main() -> std::io::Result<()> {
    let mut app = Application::new()?;
    let (width, height) = app.terminal.size();

    let biorhythm_data = Arc::new(Mutex::new(Some(Biorhythm::new(10000))));

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

    // Main window with chart
    let mut main_window = Window::new(Rect::new(2, 1, 78, 22), "Biorhythm Calculator");
    let chart = BiorhythmChart::new(Rect::new(1, 1, 74, 19), Arc::clone(&biorhythm_data));
    main_window.add(Box::new(chart));
    app.desktop.add(Box::new(main_window));

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
                        let mut dialog = create_biorhythm_dialog();
                        let result = dialog.execute(&mut app);
                        let days = match result {
                            100 => Some(5000),
                            101 => Some(10000),
                            102 => Some(15000),
                            _ => None,
                        };
                        if let Some(days_alive) = days {
                            *biorhythm_data.lock().unwrap() = Some(Biorhythm::new(days_alive));
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
