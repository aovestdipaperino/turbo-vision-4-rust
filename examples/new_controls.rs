// (C) 2025 - Enzo Lombardi
// New Controls Demo - the widgets added beyond the Borland set.
//
// One dialog exercising all three, wired to each other:
//   ComboBox  picks the bar's glyph style, its mode, and whether the
//             percentage is shown. F4 or a click drops the list down.
//   Spinner   sets the target percentage. Type digits, use the arrows, or
//             click the steppers.
//   ProgressBar  redraws from those settings every frame, and animates
//             itself in marquee mode.
//
// Tab moves between controls. Alt-X quits.

use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{EventType, KB_ALT_X};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::button::Button;
use turbo_vision::views::combo_box::{ComboBox, ComboState};
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::progress_bar::{ProgressBar, ProgressMode, ProgressStyle};
use turbo_vision::views::spinner::Spinner;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::{IdleView, View, ViewId};

use std::cell::RefCell;
use std::rc::Rc;

// Combo registration ids. Each must be unique among the live combo boxes.
const COMBO_STYLE: u16 = 1;
const COMBO_MODE: u16 = 2;
const COMBO_PERCENT: u16 = 3;

/// Read a combo's current choice as an index, defaulting to the first item.
fn choice(state: &Rc<RefCell<ComboState>>) -> usize {
    state.borrow().selected.unwrap_or(0)
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut dialog = Dialog::new(Rect::new(8, 2, 72, 21), "New Controls");

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 1, 60, 2),
        "Controls added beyond the Borland Turbo Vision set.",
    )));

    // --- Combo boxes -----------------------------------------------------
    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 3, 20, 4),
        "Bar style:",
    )));
    let style_combo = ComboBox::with_items(
        Rect::new(20, 3, 40, 4),
        COMBO_STYLE,
        vec!["Smooth".into(), "Blocks".into(), "ASCII".into()],
    );
    let style_state = style_combo.state();
    dialog.add(Box::new(style_combo));

    dialog.add(Box::new(StaticText::new(Rect::new(2, 5, 20, 6), "Mode:")));
    let mode_combo = ComboBox::with_items(
        Rect::new(20, 5, 40, 6),
        COMBO_MODE,
        vec!["Determinate".into(), "Marquee".into()],
    );
    let mode_state = mode_combo.state();
    dialog.add(Box::new(mode_combo));

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 7, 20, 8),
        "Percentage:",
    )));
    let percent_combo = ComboBox::with_items(
        Rect::new(20, 7, 40, 8),
        COMBO_PERCENT,
        vec!["Shown".into(), "Hidden".into()],
    );
    let percent_state = percent_combo.state();
    dialog.add(Box::new(percent_combo));

    // --- Spinner ---------------------------------------------------------
    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 9, 20, 10),
        "Target:",
    )));
    let mut spinner = Spinner::new(Rect::new(20, 9, 32, 10), 0, 100);
    spinner.set_value(35);
    spinner.set_step(5);
    spinner.set_suffix("%");
    let spinner_id: ViewId = dialog.add(Box::new(spinner));

    // --- Progress bar ----------------------------------------------------
    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 11, 20, 12),
        "Progress:",
    )));
    let bar_id: ViewId = dialog.add(Box::new(ProgressBar::new(Rect::new(20, 11, 60, 12), 100)));

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 13, 60, 14),
        "F4 opens a list. Tab moves on. Digits drive the spinner.",
    )));

    dialog.add(Box::new(Button::new(
        Rect::new(24, 15, 38, 17),
        "Close",
        CM_QUIT,
        true,
    )));

    dialog.set_initial_focus();
    app.desktop.add(Box::new(dialog));

    let (w, h) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, h as i16 - 1, w as i16, h as i16),
        vec![StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT)],
    ));

    loop {
        // Push the current combo and spinner settings into the bar, then let
        // the bar animate itself. Both lookups go through the desktop's child
        // list, so the dialog stays the owner of every control.
        let target = read_spinner(&mut app, spinner_id).unwrap_or(0);
        let style = match choice(&style_state) {
            0 => ProgressStyle::Smooth,
            1 => ProgressStyle::Blocks,
            _ => ProgressStyle::Ascii,
        };
        let mode = if choice(&mode_state) == 1 {
            ProgressMode::Marquee
        } else {
            ProgressMode::Determinate
        };
        let show_percent = choice(&percent_state) == 0;

        with_bar(&mut app, bar_id, |bar| {
            bar.set_style(style);
            bar.set_mode(mode);
            bar.set_show_percent(show_percent);
            bar.set_value(target as u64);
            bar.idle();
        });

        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut status) = app.status_line {
            status.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        let Some(mut event) = app
            .terminal
            .poll_event(Duration::from_millis(30))
            .ok()
            .flatten()
        else {
            continue;
        };

        if event.what == EventType::Keyboard && event.key_code == KB_ALT_X {
            break;
        }

        app.desktop.handle_event(&mut event);

        // A combo box asks for its list with a command; the application opens
        // the popup, because only it can reach the terminal.
        if event.what == EventType::Command {
            if event.command == CM_QUIT {
                break;
            }
            app.handle_event(&mut event);
        }
    }

    Ok(())
}

/// Read the spinner's current value out of the dialog.
fn read_spinner(app: &mut Application, id: ViewId) -> Option<i64> {
    let mut value = None;
    with_child::<Spinner, _>(app, id, |s| value = Some(s.value()));
    value
}

/// Run `f` on the progress bar living in the dialog.
fn with_bar(app: &mut Application, id: ViewId, f: impl FnOnce(&mut ProgressBar)) {
    with_child::<ProgressBar, _>(app, id, f);
}

/// Find a child of the desktop's dialog by view id and downcast it.
///
/// Examples normally keep their own handles to controls, but these live inside
/// the dialog that owns them, so they are reached by id instead.
fn with_child<T: 'static, F: FnOnce(&mut T)>(app: &mut Application, id: ViewId, f: F) {
    for i in 0..app.desktop.child_count() {
        let child = app.desktop.child_at_mut(i);
        if let Some(dialog) = child.as_any_mut().downcast_mut::<Dialog>() {
            if let Some(view) = dialog.child_by_id_mut(id) {
                if let Some(target) = view.as_any_mut().downcast_mut::<T>() {
                    f(target);
                    return;
                }
            }
        }
    }
}
