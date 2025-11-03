// History System Demo
//
// Demonstrates:
// - HistoryManager for storing history
// - HistoryWindow for displaying history popup
// - History button (conceptual - full InputLine integration would be more complex)

use turbo_vision::core::history::HistoryManager;
use turbo_vision::core::geometry::Point;
use turbo_vision::views::history_window::HistoryWindow;
use turbo_vision::app::Application;

const HISTORY_ID_SEARCH: u16 = 1;
const HISTORY_ID_FILENAME: u16 = 2;

fn main() -> std::io::Result<()> {
    // Populate some test history
    HistoryManager::add(HISTORY_ID_SEARCH, "rustlang".to_string());
    HistoryManager::add(HISTORY_ID_SEARCH, "turbo vision".to_string());
    HistoryManager::add(HISTORY_ID_SEARCH, "terminal ui".to_string());

    HistoryManager::add(HISTORY_ID_FILENAME, "config.toml".to_string());
    HistoryManager::add(HISTORY_ID_FILENAME, "main.rs".to_string());

    let mut app = Application::new()?;

    println!("History System Demo");
    println!("===================");
    println!();
    println!("Search history ({} items):", HistoryManager::count(HISTORY_ID_SEARCH));
    for (i, item) in HistoryManager::get_list(HISTORY_ID_SEARCH).iter().enumerate() {
        println!("  {}: {}", i + 1, item);
    }
    println!();
    println!("Filename history ({} items):", HistoryManager::count(HISTORY_ID_FILENAME));
    for (i, item) in HistoryManager::get_list(HISTORY_ID_FILENAME).iter().enumerate() {
        println!("  {}: {}", i + 1, item);
    }
    println!();
    println!("Opening history window...");

    // Show history window
    let mut window = HistoryWindow::new(Point::new(10, 5), HISTORY_ID_SEARCH, 30);

    if let Some(selected) = window.execute(&mut app.terminal) {
        println!("Selected: {}", selected);
    } else {
        println!("Cancelled");
    }

    Ok(())
}
