mod db;
mod tray;
mod ui;

use dioxus_desktop::launch::launch;

fn main() {
    println!("Starting Rustine (prototype)");

    // Initialize DB (actor) and keep the handle
    let db_handle = db::init_db().expect("failed to init db");

    // Set global DB handle for UI/tray access and start tray
    crate::db::set_global(db_handle.clone()).expect("failed to set global db");
    tray::start_tray(db_handle.clone()).expect("failed to start tray");

    // Launch Dioxus app (UI in src/ui.rs)
    // pass empty plugin and state vectors
    launch(ui::app, Vec::new(), Vec::new());
}
