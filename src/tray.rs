use anyhow::Result;
use std::thread;

/// Start a tray icon and menu in a background thread (skeleton)
pub fn start_tray() -> Result<()> {
    // For scaffold, spawn a thread that would own the tray icon and listen for events.
    thread::spawn(|| {
        // TODO: instantiate tray-icon, build menu and forward events to UI via channel
        println!("Tray thread started (skeleton)");
    });

    Ok(())
}
