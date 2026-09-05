// (C) 2025 - Enzo Lombardi
// New Controls Demo - the widgets added beyond the Borland set.
//
// One dialog, two tabbed pages, five new controls:
//
//   TabbedPane   holds both pages. F6 and Shift+F6 switch, so do Alt-P and
//                Alt-F, and so does clicking a tab.
//   ComboBox     picks the bar's glyph style, its mode, and whether the
//                percentage shows. F4 or a click drops the list down.
//   Spinner      sets the target percentage. Type digits, use the arrows,
//                or click the steppers.
//   ProgressBar  redraws from those settings every frame, and animates
//                itself in marquee mode.
//   Table        a scrollable grid on the second page. Arrows move the
//                focused cell, Ctrl+Left and Ctrl+Right jump to the end
//                columns.
//
// Tab moves between controls on a page. Alt-X quits.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{EventType, KB_ALT_X};
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::GroupLike;
use turbo_vision::views::button::Button;
use turbo_vision::views::combo_box::{ComboBox, ComboState};
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::group::Group;
use turbo_vision::views::progress_bar::{ProgressBar, ProgressMode, ProgressStyle};
use turbo_vision::views::spinner::Spinner;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::StatusLine;
use turbo_vision::views::tabbed_pane::TabbedPane;
use turbo_vision::views::table::{Column, Table};
use turbo_vision::views::{View, ViewId};

// Combo registration ids. Each must be unique among the live combo boxes.
const COMBO_STYLE: u16 = 1;
const COMBO_MODE: u16 = 2;
const COMBO_PERCENT: u16 = 3;

/// Read a combo's current choice as an index, defaulting to the first item.
fn choice(state: &Rc<RefCell<ComboState>>) -> usize {
    state.borrow().selected.unwrap_or(0)
}

/// Handles onto the controls the loop has to read or write each frame.
struct Handles {
    pane: ViewId,
    spinner: ViewId,
    bar: ViewId,
    style: Rc<RefCell<ComboState>>,
    mode: Rc<RefCell<ComboState>>,
    percent: Rc<RefCell<ComboState>>,
}

/// The first page's controls, handed back so the loop can reach them.
struct ProgressPage {
    page: Group,
    style: Rc<RefCell<ComboState>>,
    mode: Rc<RefCell<ComboState>>,
    percent: Rc<RefCell<ComboState>>,
    spinner: ViewId,
    bar: ViewId,
}

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut dialog = Dialog::new(Rect::new(6, 2, 74, 23), "New Controls");
    let mut pane = TabbedPane::new(Rect::new(2, 1, 66, 16));

    let progress = build_progress_page(&pane);
    pane.add_page("~P~rogress", progress.page);
    pane.add_page("~F~iles", build_table_page(&pane));

    let pane_id = dialog.add(pane);

    dialog.add(StaticText::new(
        Rect::new(2, 17, 64, 18),
        "F6 switches pages. F4 opens a list. Tab moves on.",
    ));
    dialog.add(Button::new(
        Rect::new(26, 18, 40, 20),
        "Close",
        CM_QUIT,
        true,
    ));

    dialog.set_initial_focus();
    app.desktop.add(dialog);

    let handles = Handles {
        pane: pane_id,
        spinner: progress.spinner,
        bar: progress.bar,
        style: progress.style,
        mode: progress.mode,
        percent: progress.percent,
    };

    let (w, h) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, h as i16 - 1, w as i16, h as i16),
        vec![
            StatusItemBuilder::new()
                .text("~Alt-X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
        ],
    ));

    run_loop(&mut app, &handles);
    Ok(())
}

/// Build the first page: three combo boxes, a spinner and the bar they drive.
fn build_progress_page(pane: &TabbedPane) -> ProgressPage {
    let mut page = Group::new(pane.page_area());

    page.add(StaticText::new(Rect::new(1, 0, 20, 1), "Bar style:"));
    let style_combo = ComboBox::with_items(
        Rect::new(18, 0, 38, 1),
        COMBO_STYLE,
        vec!["Smooth".into(), "Blocks".into(), "ASCII".into()],
    );
    let style = style_combo.state();
    page.add(style_combo);

    page.add(StaticText::new(Rect::new(1, 2, 20, 3), "Mode:"));
    let mode_combo = ComboBox::with_items(
        Rect::new(18, 2, 38, 3),
        COMBO_MODE,
        vec!["Determinate".into(), "Marquee".into()],
    );
    let mode = mode_combo.state();
    page.add(mode_combo);

    page.add(StaticText::new(Rect::new(1, 4, 20, 5), "Percentage:"));
    let percent_combo = ComboBox::with_items(
        Rect::new(18, 4, 38, 5),
        COMBO_PERCENT,
        vec!["Shown".into(), "Hidden".into()],
    );
    let percent = percent_combo.state();
    page.add(percent_combo);

    page.add(StaticText::new(Rect::new(1, 6, 20, 7), "Target:"));
    let mut spin = Spinner::new(Rect::new(18, 6, 30, 7), 0, 100);
    spin.set_value(35);
    spin.set_step(5);
    spin.set_suffix("%");
    let spinner = page.add(spin);

    page.add(StaticText::new(Rect::new(1, 8, 20, 9), "Progress:"));
    let bar = page.add(ProgressBar::new(Rect::new(18, 8, 61, 9), 100));

    page.set_initial_focus();
    ProgressPage {
        page,
        style,
        mode,
        percent,
        spinner,
        bar,
    }
}

/// Build the second page: a grid describing this crate's own source files.
fn build_table_page(pane: &TabbedPane) -> Group {
    let mut page = Group::new(pane.page_area());

    page.add(StaticText::new(
        Rect::new(1, 0, 61, 1),
        "Arrows move the focused cell. Ctrl+Left and Ctrl+Right jump.",
    ));

    let mut table = Table::new(Rect::new(1, 2, 61, 11), 0);
    table.set_columns(vec![
        Column::new("Name", 20),
        Column::right("Lines", 7),
        Column::new("Kind", 10),
        Column::new("Module", 12),
    ]);
    table.set_rows(
        [
            ("progress_bar.rs", 520, "view", "views"),
            ("combo_box.rs", 690, "view", "views"),
            ("spinner.rs", 610, "view", "views"),
            ("table.rs", 700, "view", "views"),
            ("tabbed_pane.rs", 560, "view", "views"),
            ("dialog.rs", 900, "container", "views"),
            ("group.rs", 840, "container", "views"),
            ("window.rs", 780, "container", "views"),
            ("application.rs", 960, "runtime", "app"),
            ("palette.rs", 1010, "core", "core"),
            ("event.rs", 430, "core", "core"),
            ("geometry.rs", 210, "core", "core"),
            ("draw.rs", 180, "core", "core"),
            ("crossterm_backend.rs", 640, "backend", "terminal"),
        ]
        .into_iter()
        .map(|(name, lines, kind, module)| {
            vec![
                name.to_string(),
                lines.to_string(),
                kind.to_string(),
                module.to_string(),
            ]
        })
        .collect(),
    );
    page.add(table);

    page.set_initial_focus();
    page
}

/// Drive the dialog: push the settings into the bar, draw, dispatch.
fn run_loop(app: &mut Application, handles: &Handles) {
    loop {
        apply_settings(app, handles);

        app.terminal.draw_view(&mut app.desktop);
        if let Some(ref mut status) = app.status_line {
            app.terminal.draw_view(status);
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

        turbo_vision::views::view::dispatch_to_child(&mut app.desktop, &mut event);

        if event.what == EventType::Command {
            if event.command == CM_QUIT {
                break;
            }
            // A combo box asks for its list with a command; the application
            // opens the popup, because only it can reach the terminal.
            app.handle_event(&mut event);
        }
    }
}

/// Copy the combo and spinner settings into the progress bar, then tick it.
fn apply_settings(app: &mut Application, handles: &Handles) {
    let target = read_spinner(app, handles).unwrap_or(0);
    let style = match choice(&handles.style) {
        0 => ProgressStyle::Smooth,
        1 => ProgressStyle::Blocks,
        _ => ProgressStyle::Ascii,
    };
    let mode = if choice(&handles.mode) == 1 {
        ProgressMode::Marquee
    } else {
        ProgressMode::Determinate
    };
    let show_percent = choice(&handles.percent) == 0;

    with_page_child::<ProgressBar, _>(app, handles.pane, handles.bar, |bar| {
        bar.set_style(style);
        bar.set_mode(mode);
        bar.set_show_percent(show_percent);
        bar.set_value(target as u64);
        bar.idle();
    });
}

/// Read the spinner's current value out of the first page.
fn read_spinner(app: &mut Application, handles: &Handles) -> Option<i64> {
    let mut value = None;
    with_page_child::<Spinner, _>(app, handles.pane, handles.spinner, |s| {
        value = Some(s.value());
    });
    value
}

/// Run `f` on a control living on the tabbed pane's first page.
///
/// Examples normally keep their own handles to controls, but these are owned by
/// the page group inside the pane inside the dialog, so they are reached by id:
/// desktop, then dialog, then pane, then page.
fn with_page_child<T: 'static, F: FnOnce(&mut T)>(
    app: &mut Application,
    pane_id: ViewId,
    child_id: ViewId,
    f: F,
) {
    for i in 0..app.desktop.child_count() {
        let child = app.desktop.child_at_mut(i);
        let Some(dialog) = child.as_any_mut().downcast_mut::<Dialog>() else {
            continue;
        };
        let Some(pane_view) = dialog.child_by_id_mut(pane_id) else {
            continue;
        };
        let Some(pane) = pane_view.as_any_mut().downcast_mut::<TabbedPane>() else {
            continue;
        };
        let Some(page) = pane.page_mut(0) else {
            continue;
        };
        if let Some(view) = page.child_by_id_mut(child_id) {
            if let Some(target) = view.as_any_mut().downcast_mut::<T>() {
                f(target);
                return;
            }
        }
    }
}
