// (C) 2025 - Enzo Lombardi
// Clusters, tooltips and the frame zoom icon.
//
//   CheckBoxes / RadioButtons  each hold several items in one focusable
//                              control with a single bitmask value. Arrows
//                              move within a cluster, Space toggles or
//                              selects, Tab leaves it, Alt+letter jumps.
//   Tooltip                    one per dialog, holding a hint per control.
//                              Rest the pointer on a control to raise it.
//   Zoom icon                  the window behind the dialog carries [^] on
//                              its title bar; click it to zoom and restore.
//
// Alt-X quits.

use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{EventType, KB_ALT_X};
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;
use turbo_vision::views::button::Button;
use turbo_vision::views::cluster_group::{CheckBoxes, RadioButtons};
use turbo_vision::views::dialog::Dialog;
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::{StatusItem, StatusLine};
use turbo_vision::views::tooltip::Tooltip;
use turbo_vision::views::window::Window;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // A plain window behind the dialog, to show the frame icons.
    let window = Window::new(Rect::new(2, 2, 40, 10), "Zoom me");
    app.desktop.add(Box::new(window));

    let mut dialog = Dialog::new(Rect::new(18, 4, 80, 20), "Clusters and hints");

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 1, 30, 2),
        "Style (check boxes):",
    )));
    let styles = CheckBoxes::new(
        Rect::new(2, 2, 24, 5),
        vec!["~B~old".into(), "~I~talic".into(), "~U~nderline".into()],
    );
    let styles_rect = Rect::new(2, 2, 24, 5);
    dialog.add(Box::new(styles));

    dialog.add(Box::new(StaticText::new(
        Rect::new(26, 1, 50, 2),
        "Align (radio buttons):",
    )));
    let align = RadioButtons::new(
        Rect::new(26, 2, 48, 5),
        vec!["~L~eft".into(), "~C~entre".into(), "~R~ight".into()],
    );
    let align_rect = Rect::new(26, 2, 48, 5);
    dialog.add(Box::new(align));

    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 7, 58, 8),
        "Rest the pointer on a cluster to raise its hint.",
    )));
    dialog.add(Box::new(StaticText::new(
        Rect::new(2, 8, 58, 9),
        "The window behind has a zoom icon on its title bar.",
    )));

    let ok_rect = Rect::new(23, 11, 37, 13);
    dialog.add(Box::new(Button::new(ok_rect, "Close", CM_QUIT, true)));

    // The tooltip is added last so it draws over everything else. Hint rects
    // are dialog-relative, the same coordinates the controls were given.
    let mut tips = Tooltip::new(Rect::new(0, 0, 62, 16));
    tips.set_delay(Duration::from_millis(400));
    tips.add_hint(styles_rect, "Any combination of these");
    tips.add_hint(align_rect, "Exactly one of these");
    tips.add_hint(ok_rect, "Leave the demo");
    dialog.add(Box::new(tips));

    dialog.set_initial_focus();
    app.desktop.add(Box::new(dialog));

    let (w, h) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, h as i16 - 1, w as i16, h as i16),
        vec![StatusItem::new("~Alt-X~ Exit", KB_ALT_X, CM_QUIT)],
    ));

    loop {
        app.desktop.draw(&mut app.terminal);
        if let Some(ref mut status) = app.status_line {
            status.draw(&mut app.terminal);
        }
        let _ = app.terminal.flush();

        match app.terminal.poll_event(Duration::from_millis(30)) {
            Ok(Some(mut event)) => {
                if event.what == EventType::Keyboard && event.key_code == KB_ALT_X {
                    break;
                }
                app.desktop.handle_event(&mut event);
                if event.what == EventType::Command && event.command == CM_QUIT {
                    break;
                }
                app.handle_event(&mut event);
            }
            // Nothing happening: drive the timers, which is what raises hints.
            _ => app.idle(),
        }
    }

    Ok(())
}
