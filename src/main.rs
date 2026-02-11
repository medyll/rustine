mod db;
mod tray;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    println!("Starting Rustine (prototype)");

    // Initialize DB (simple synchronous handle for scaffold)
    let _db = db::init_db()?;

    // Start tray in background (skeleton)
    tray::start_tray()?

    // Launch Dioxus app (UI in src/ui.rs)
    dioxus_desktop::launch(ui::app);

    Ok(())
}
