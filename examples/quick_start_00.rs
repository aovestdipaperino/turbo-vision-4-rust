// (C) 2025 - Enzo Lombardi
//
// CTRL+C or ALt+X or ESC-ESC to exit.

use turbo_vision::prelude::*;
fn main() -> turbo_vision::core::error::Result<()> {
    let mut app = Application::new()?;
    app.run();
    Ok(())
}
