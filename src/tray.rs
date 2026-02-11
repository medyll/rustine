use crossbeam_channel::{unbounded, Receiver, Sender};
use once_cell::sync::OnceCell;
use anyhow::Result;
use std::thread;
use std::time::Duration;

// URL record type not referenced in this module

#[allow(dead_code)]
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
    use tray_icon::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu, MenuId};
    use tray_icon::TrayIconBuilder;
    // Arc not needed here

    pub fn start_real_tray(db: crate::db::DbHandle) -> Result<()> {
        let tx = TRAY_TX
            .get()
            .expect("TRAY_TX must be set before starting real tray")
            .clone();

        // spawn a dedicated tray thread which will own the TrayIcon and perform menu updates
        thread::spawn(move || -> Result<()> {
            // create an initial empty menu and tray icon
            let initial_menu = Menu::new();
            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(initial_menu))
                .with_tooltip("Rustine")
                .build()?;

            // receiver for menu events
            let menu_rx = tray_icon::menu::MenuEvent::receiver();

            loop {
                // Build new menu from DB
                let menu = Menu::new();
                let show_item = MenuItem::new("Voir les URLs", true, None);
                let add_item = MenuItem::new("Ajouter URL", true, None);
                let history_submenu = Submenu::new("Historique", true);

                let mut id_map: std::collections::HashMap<MenuId, TrayEvent> = std::collections::HashMap::new();

                // populate history
                if let Ok(list) = db.list_recent(5) {
                    for rec in list {
                        let display = rec.site_name.clone().unwrap_or_else(|| rec.label.clone());
                        let label = format!("{} ({})", display, rec.url);
                        let item = MenuItem::new(&label, true, None);
                        let item_id = item.id();
                        id_map.insert(item_id.clone(), TrayEvent::OpenUrl(rec.id));
                        let _ = history_submenu.append_items(&[&item]);
                    }
                }

                let _ = menu.append_items(&[
                    &show_item,
                    &add_item,
                    &PredefinedMenuItem::separator(),
                    &history_submenu,
                    &PredefinedMenuItem::separator(),
                    &MenuItem::new("Quitter Rustine", true, None),
                ]);

                // set the new menu on the tray icon
                tray.set_menu(Some(Box::new(menu)));

                // Drain menu events and forward as TrayEvent
                while let Ok(ev) = menu_rx.try_recv() {
                    if let Some(action) = id_map.get(&ev.id) {
                        let _ = tx.send(action.clone());
                    } else if ev.id == show_item.id() {
                        let _ = tx.send(TrayEvent::Show);
                    } else if ev.id == add_item.id() {
                        let _ = tx.send(TrayEvent::Add);
                    } else {
                        // other events (e.g., Quit)
                        // match by text fallback is not available; rely on static Quit item id
                    }
                }

                // Sleep until next refresh
                thread::sleep(Duration::from_secs(30));
            }
        });

        Ok(())
    }

}

/// Start a tray icon/menu. Requires the `real_tray` feature to be enabled.
pub fn start_tray(db: crate::db::DbHandle) -> Result<()> {
    // create global tx/rx for tray events
    let (tx, rx): (Sender<TrayEvent>, Receiver<TrayEvent>) = unbounded();
    let _ = TRAY_RX.set(rx.clone());
    let _ = TRAY_TX.set(tx.clone());

    #[cfg(feature = "real_tray")]
    {
        return real_tray::start_real_tray(db.clone());
    }

    // If `real_tray` feature is not enabled, return an error to make this explicit.
    #[cfg(not(feature = "real_tray"))]
    {
        return Err(anyhow::anyhow!(
            "real_tray feature not enabled — enable with `--features real_tray`"
        ));
    }
}

/// Returns a clone of the `Receiver<TrayEvent>` if the tray has been started.
pub fn get_receiver() -> Option<Receiver<TrayEvent>> {
    TRAY_RX.get().cloned()
}

/// Send a tray event programmatically (useful for testing or hooking real menu callbacks).
#[allow(dead_code)]
pub fn send_event(ev: TrayEvent) -> Result<()> {
    if let Some(tx) = TRAY_TX.get() {
        tx.send(ev).map_err(|e| anyhow::anyhow!("failed to send tray event: {}", e))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("tray not started"))
    }
}
