// (C) 2026 - Enzo Lombardi
// Extras Notebook Example
// Demonstrates the container controls from turbo-vision-extras:
// - Notebook: tabbed pages (click a tab or Ctrl+PgUp / Ctrl+PgDn)
// - ScrollPane: a form taller than its window (wheel / Ctrl+Up / Ctrl+Down)
// - popup_menu(): a context menu on F9 with a check-mark item
//
// The example uses a hand-rolled event loop (get_event/handle_event) so it
// can open the popup menu, which needs terminal access — the same pattern
// an application would use for right-click context menus.

use std::cell::RefCell;
use std::rc::Rc;
use turbo_vision::core::menu_data::MenuItemBuilder;
use turbo_vision::core::status_data::StatusItemBuilder;
use turbo_vision::views::GroupLike;

use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::{EventType, KB_ESC_ESC, KB_F9};
use turbo_vision::core::geometry::{Point, Rect};
use turbo_vision::core::menu_data::{Menu, MenuItem};
use turbo_vision::views::static_text::StaticText;
use turbo_vision::views::status_line::StatusLine;
use turbo_vision::views::window::WindowBuilder;
use turbo_vision_extras::{
    ComboBox, Notebook, ScrollPane, SpinControl, is_menu_item_checked, popup_menu,
    set_menu_item_checked,
};

const CM_POPUP: u16 = 1000;
const CM_TOGGLE_WRAP: u16 = 1010;
const CM_SAY_HELLO: u16 = 1011;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let (width, height) = app.terminal.size();
    app.set_status_line(StatusLine::new(
        Rect::new(0, height - 1, width, height),
        vec![
            StatusItemBuilder::new()
                .text("~F9~ Menu")
                .key("F9")
                .command(CM_POPUP)
                .build(),
            StatusItemBuilder::new()
                .text("~Alt-X~ Exit")
                .key("Alt+X")
                .command(CM_QUIT)
                .build(),
            StatusItemBuilder::new()
                .text("~Esc-Esc~ Exit")
                .key_code(KB_ESC_ESC)
                .command(CM_QUIT)
                .build(),
        ],
    ));

    // ---- Notebook window -------------------------------------------------
    let mut nb_window = WindowBuilder::new()
        .bounds(Rect::new(3, 2, 50, 16))
        .title("Notebook")
        .build();

    let mut notebook = Notebook::new(Rect::new(1, 1, 44, 12));

    let general = notebook.add_page("General");
    notebook.add_to_page(
        general,
        Box::new(StaticText::new(Rect::new(1, 1, 40, 2), "Language:")),
    );
    let language = Rc::new(RefCell::new("Rust".to_string()));
    notebook.add_to_page(
        general,
        Box::new(ComboBox::new(
            Rect::new(12, 1, 32, 2),
            vec!["Rust".into(), "Pascal".into(), "C++".into()],
            language,
        )),
    );

    let advanced = notebook.add_page("Advanced");
    notebook.add_to_page(
        advanced,
        Box::new(StaticText::new(Rect::new(1, 1, 40, 2), "Workers:")),
    );
    let workers = Rc::new(RefCell::new(8));
    notebook.add_to_page(
        advanced,
        Box::new(SpinControl::new(Rect::new(12, 1, 22, 2), 1, 64, workers)),
    );

    let about = notebook.add_page("About");
    notebook.add_to_page(
        about,
        Box::new(StaticText::new(
            Rect::new(1, 1, 42, 3),
            "Tabbed pages, one Group per page.\nCtrl+PgUp / Ctrl+PgDn switch tabs.",
        )),
    );

    nb_window.add(notebook);
    app.desktop.add(nb_window);

    // ---- ScrollPane window ------------------------------------------------
    let mut sp_window = WindowBuilder::new()
        .bounds(Rect::new(52, 3, 88, 17))
        .title("Tall Form")
        .build();

    // A 30-row virtual form inside an ~11-row viewport
    let mut pane = ScrollPane::new(Rect::new(1, 1, 33, 12), 30);
    for i in 0..10 {
        let y = i * 3;
        pane.add(
            Box::new(StaticText::new(
                Rect::new(1, y, 30, y + 1),
                &format!("Field {} — scroll with the wheel", i + 1),
            )),
            Rect::new(1, y, 30, y + 1),
        );
    }
    sp_window.add(pane);
    app.desktop.add(sp_window);

    // ---- Event loop with a context menu ------------------------------------
    // The menu keeps state across openings so the check mark persists
    let mut context_menu = Menu::from_items(vec![
        MenuItemBuilder::new()
            .text("Word wrap")
            .command(CM_TOGGLE_WRAP)
            .build(),
        MenuItem::separator(),
        MenuItemBuilder::new()
            .text("Say hello")
            .command(CM_SAY_HELLO)
            .build(),
        MenuItemBuilder::new()
            .text("E~x~it")
            .command(CM_QUIT)
            .build(),
    ]);
    set_menu_item_checked(&mut context_menu, 0, true);

    while app.running {
        let Some(mut event) = app.get_event() else {
            continue;
        };

        // Let the app dispatch first: the status line converts F9 into
        // CM_POPUP, and unhandled commands survive dispatch
        if !(event.what == EventType::Keyboard && event.key_code == KB_F9) {
            app.handle_event(&mut event);
        }

        // F9 (raw key or the status line's command) opens the popup
        let popup_requested = (event.what == EventType::Keyboard && event.key_code == KB_F9)
            || (event.what == EventType::Command && event.command == CM_POPUP);
        if popup_requested {
            match popup_menu(&mut app.terminal, Point::new(10, 5), context_menu.clone()) {
                Some(CM_TOGGLE_WRAP) => {
                    let now = !is_menu_item_checked(&context_menu, 0);
                    set_menu_item_checked(&mut context_menu, 0, now);
                }
                Some(CM_QUIT) => app.running = false,
                Some(CM_SAY_HELLO) | None => {}
                Some(_) => {}
            }
            continue;
        }
    }

    Ok(())
}
