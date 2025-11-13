// (C) 2025 - Enzo Lombardi
// Terminal Widget Demo - demonstrates scrolling output viewer

use std::time::{Duration, Instant};
use turbo_vision::app::Application;
use turbo_vision::core::command::CM_QUIT;
use turbo_vision::core::event::EventType;
use turbo_vision::core::geometry::Rect;
use turbo_vision::core::palette::Attr;
use turbo_vision::views::button::ButtonBuilder;
use turbo_vision::views::static_text::StaticTextBuilder;
use turbo_vision::views::terminal_widget::TerminalWidget;
use turbo_vision::views::window::WindowBuilder;
use turbo_vision::views::View;

const CM_ADD_LINE: u16 = 100;
const CM_ADD_BATCH: u16 = 101;
const CM_CLEAR: u16 = 102;

fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;

    // Create terminal widget directly on desktop (not inside window)
    let mut terminal = TerminalWidget::new(Rect::new(7, 6, 73, 18)).with_scrollbar();

    // Add some initial content
    terminal.append_text("Build Output Viewer");
    terminal.append_text("==================");
    terminal.append_text("");
    terminal.append_text("Compiling my_project v0.1.0");
    terminal.append_line_colored(
        "   Compiling dep1 v1.0.0".to_string(),
        Attr::from_u8(0x0E), // Yellow
    );
    terminal.append_line_colored(
        "   Compiling dep2 v2.0.0".to_string(),
        Attr::from_u8(0x0E), // Yellow
    );
    terminal.append_line_colored(
        "   Compiling dep3 v1.5.0".to_string(),
        Attr::from_u8(0x0E), // Yellow
    );
    terminal.append_text("   Compiling main");
    terminal.append_line_colored(
        "    Finished dev [unoptimized + debuginfo] target(s) in 2.34s".to_string(),
        Attr::from_u8(0x0A), // Green
    );
    terminal.append_text("");
    terminal.append_text("Press buttons to interact...");
    terminal.append_text("");

    app.desktop.add(Box::new(terminal));

    // Create window with buttons
    let mut window = WindowBuilder::new()
        .bounds(Rect::new(5, 2, 75, 22))
        .title("Terminal Widget Demo")
        .build();

    window.add(Box::new(StaticTextBuilder::new()
        .bounds(Rect::new(2, 2, 66, 3))
        .text("Simulates build output. Use arrows/PgUp/PgDn to scroll. Auto-scrolls when at bottom.")
        .build()));

    // Buttons (positioned below the terminal widget area)
    window.add(Box::new(ButtonBuilder::new()
        .bounds(Rect::new(2, 15, 17, 17))
        .title("Add Line")
        .command(CM_ADD_LINE)
        .default(false)
        .build()));

    window.add(Box::new(ButtonBuilder::new()
        .bounds(Rect::new(18, 15, 36, 17))
        .title("Add 100 Lines")
        .command(CM_ADD_BATCH)
        .default(false)
        .build()));

    window.add(Box::new(ButtonBuilder::new()
        .bounds(Rect::new(37, 15, 48, 17))
        .title("Clear")
        .command(CM_CLEAR)
        .default(false)
        .build()));

    window.add(Box::new(ButtonBuilder::new()
        .bounds(Rect::new(49, 15, 60, 17))
        .title("Quit")
        .command(CM_QUIT)
        .default(true)
        .build()));

    app.desktop.add(Box::new(window));

    let mut line_counter = 1;
    let start_time = Instant::now();

    // Event loop
    loop {
        app.desktop.draw(&mut app.terminal);
        let _ = app.terminal.flush();

        if let Some(mut event) = app
            .terminal
            .poll_event(Duration::from_millis(50))
            .ok()
            .flatten()
        {
            app.desktop.handle_event(&mut event);

            if event.what == EventType::Command {
                match event.command {
                    CM_ADD_LINE => {
                        // Access terminal widget (first child of desktop)
                        if let Some(terminal) = app
                            .desktop
                            .child_at_mut(0)
                            .as_any_mut()
                            .downcast_mut::<TerminalWidget>()
                        {
                            let elapsed = start_time.elapsed();
                            let line = format!(
                                "[{:>6.2}s] Processing item {}...",
                                elapsed.as_secs_f32(),
                                line_counter
                            );
                            terminal.append_line(line);
                            line_counter += 1;
                        }
                    }
                    CM_ADD_BATCH => {
                        // Access terminal widget
                        if let Some(terminal) = app
                            .desktop
                            .child_at_mut(0)
                            .as_any_mut()
                            .downcast_mut::<TerminalWidget>()
                        {
                            for i in 0..100 {
                                let elapsed = start_time.elapsed();
                                let line = format!(
                                    "[{:>6.2}s] Batch processing item {}/100",
                                    elapsed.as_secs_f32(),
                                    i + 1
                                );
                                terminal.append_line(line);
                            }

                            terminal.append_line_colored(
                                "Batch processing complete!".to_string(),
                                Attr::from_u8(0x0A), // Green
                            );
                            line_counter += 100;
                        }
                    }
                    CM_CLEAR => {
                        // Access terminal widget
                        if let Some(terminal) = app
                            .desktop
                            .child_at_mut(0)
                            .as_any_mut()
                            .downcast_mut::<TerminalWidget>()
                        {
                            terminal.clear();
                            terminal.append_text("Output cleared.");
                            terminal.append_text("");
                            line_counter = 1;
                        }
                    }
                    CM_QUIT => break,
                    _ => {}
                }
            }

            // Handle Ctrl+C or F10
            if event.what == EventType::Keyboard {
                let key = event.key_code;
                if key == 0x0003 || key == turbo_vision::core::event::KB_F10 {
                    break;
                }
            }
        }
    }

    Ok(())
}
