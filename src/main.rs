mod db;
mod tray;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    println!("Starting Rustine (prototype)");

    // Initialize DB (actor) and keep the handle
    let db_handle = db::init_db()?;

    // Start tray in background (skeleton)
        // Set global DB handle for UI access and start tray in background
        crate::db::set_global(db_handle.clone())?;
        tray::start_tray(db_handle.clone())?;

    // Launch Dioxus app (UI in src/ui.rs)
    // The UI can call `crate::tray::get_receiver()` to obtain the tray event receiver.
    dioxus_desktop::launch(ui::app);

    drop(db_handle); // keep db_handle alive for now if needed later

    Ok(())
}
