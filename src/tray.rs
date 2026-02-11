use crossbeam_channel::{unbounded, Receiver, Sender};
use once_cell::sync::OnceCell;
use anyhow::Result;
use std::thread;
use std::time::Duration;

use crate::db::UrlRecord;

#[derive(Debug, Clone)]
pub enum TrayEvent {
    Show,
    Add,
    Quit,
    OpenUrl(i64), // open URL by id from history
}

static TRAY_RX: OnceCell<Receiver<TrayEvent>> = OnceCell::new();
static TRAY_TX: OnceCell<Sender<TrayEvent>> = OnceCell::new();

#[cfg(feature = "real_tray")]
mod real_tray {
    use super::*;
    use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
    use tray_icon::TrayIconBuilder;
    use std::sync::Arc;

    pub fn start_real_tray(db: crate::db::DbHandle) -> Result<()> {
        // Build initial menu and create tray icon. Callbacks send `TrayEvent` via global sender.
        let tx = TRAY_TX
            .get()
            .expect("TRAY_TX must be set before starting real tray")
            .clone();

        // Build menu
        let mut tray_menu = Menu::new();

        let show_item = MenuItem::new("Voir les URLs", true, None);
        let add_item = MenuItem::new("Ajouter URL", true, None);

        let history_submenu = Submenu::new("Historique", true);

        let quit_item = MenuItem::new("Quitter Rustine", true, None);

        tray_menu.append_items(&[
            &show_item,
            &add_item,
            &PredefinedMenuItem::separator(),
            &history_submenu,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ]);

        // NOTE: tray_icon API may require an actual icon - for simplicity rely on builder defaults
        let _tray = TrayIconBuilder::new()
            .with_menu(tray_menu)
            .with_tooltip("Rustine")
            .build()?;

        // Spawn a thread to refresh history periodically (and wire callbacks when supported)
        let db = db.clone();
        thread::spawn(move || loop {
            match db.list_recent(5) {
                Ok(list) => {
                    // In a full implementation we would rebuild the submenu items with callbacks
                    for rec in &list {
                        println!("tray history: {} -> {}", rec.label, rec.url);
                    }
                }
                Err(e) => eprintln!("Failed to fetch recent URLs for real tray: {}", e),
            }
            thread::sleep(Duration::from_secs(30));
        });

        Ok(())
    }
}

/// Start a tray icon/menu. By default uses a simulator; enable the `real_tray` feature to use `tray-icon` integration.
pub fn start_tray(db: crate::db::DbHandle) -> Result<()> {
    // create global tx/rx for tray events
    let (tx, rx): (Sender<TrayEvent>, Receiver<TrayEvent>) = unbounded();
    let _ = TRAY_RX.set(rx.clone());
    let _ = TRAY_TX.set(tx.clone());

    #[cfg(feature = "real_tray")]
    {
        // attempt to start real tray integration
        if let Err(e) = real_tray::start_real_tray(db) {
            eprintln!("Failed to start real tray: {}. Falling back to simulator.", e);
        } else {
            return Ok(());
        }
    }

    // Fallback simulator: spawn thread that fetches recent entries and logs menu structure
    thread::spawn(move || {
        println!("Tray simulator started");
        loop {
            match db.list_recent(5) {
                Ok(list) => {
                    println!("Building tray submenu — recent URLs:");
                    for rec in &list {
                        println!(" - {} ({}) id={}", rec.label, rec.url, rec.id);
                    }
                    println!("Tray menu: [Voir les URLs] [Ajouter URL] [Historique..] [Quitter]");
                }
                Err(e) => eprintln!("Failed to fetch recent URLs for tray: {}", e),
            }
            thread::sleep(Duration::from_secs(30));
        }
    });

    Ok(())
}

/// Returns a clone of the `Receiver<TrayEvent>` if the tray has been started.
pub fn get_receiver() -> Option<Receiver<TrayEvent>> {
    TRAY_RX.get().cloned()
}

/// Send a tray event programmatically (useful for testing or hooking real menu callbacks).
pub fn send_event(ev: TrayEvent) -> Result<()> {
    if let Some(tx) = TRAY_TX.get() {
        tx.send(ev).map_err(|e| anyhow::anyhow!("failed to send tray event: {}", e))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("tray not started"))
    }
}
