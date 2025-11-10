// (C) 2025 - Enzo Lombardi
// Dynamic Title Demo - demonstrates changing window title at runtime

use std::time::Duration;
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use turbo_vision::views::View;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::window::WindowBuilder;

// Custom command IDs for this example
const CM_UPDATE_TITLE: u16 = 100;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    let mut window = WindowBuilder::new().bounds(Rect::new(10, 3, 70, 18)).title("Click button to change title").build();

    window.add(Box::new(
        StaticTextBuilder::new()
            .bounds(Rect::new(2, 2, 56, 6))
            .text("This demo shows dynamic window title updates.\n\nClick the button below to cycle through\ndifferent window titles.")
            .build(),
    ));

    window.add(Box::new(
        ButtonBuilder::new()
            .bounds(Rect::new(15, 8, 40, 10))
            .title("Change Title")
            .command(CM_UPDATE_TITLE)
            .default(true)
            .build(),
    ));

    window.add(Box::new(ButtonBuilder::new().bounds(Rect::new(15, 11, 40, 13)).title("Quit").command(CM_QUIT).default(false).build()));

    app.desktop.add(Box::new(window));
    let window_index = app.desktop.child_count() - 1;

    let titles = [
        "Title 1: Hello World!",
        "Title 2: Dynamic Updates",
        "Title 3: Real-time Changes",
        "Title 4: Window Title Demo",
        "Title 5: Borland TV Style",
    ];
    let mut title_index = 0;

    loop {
        app.desktop.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        if let Some(mut event) = app.terminal.poll_event(Duration::from_millis(50)).ok().flatten() {
            app.desktop.handle_event(&mut event);

            if event.what == EventType::Command {
                match event.command {
                    CM_UPDATE_TITLE => {
                        // Update the window title
                        if let Some(view) = app.desktop.window_at_mut(window_index) {
                            // Downcast from &mut dyn View to &mut Window using as_any_mut()
                            if let Some(win) = view.as_any_mut().downcast_mut::<turbo_vision::views::window::Window>() {
                                win.set_title(titles[title_index]);
                            }
                        }
                        title_index = (title_index + 1) % titles.len();
                        // app.beep();
                    }
                    CM_QUIT => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
