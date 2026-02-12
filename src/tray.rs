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

        // Build the tray on the current (main) thread. This avoids creating
        // a platform event loop off the main thread (Windows panics otherwise).
        // We keep the TrayIcon alive for the process lifetime by leaking it.
        let menu = Menu::new();
        let show_item = MenuItem::new("Voir les URLs", true, None);
        let add_item = MenuItem::new("Ajouter URL", true, None);
        let history_submenu = Submenu::new("Historique", true);

        let mut id_map: std::collections::HashMap<MenuId, TrayEvent> = std::collections::HashMap::new();

        if let Ok(list) = db.list_recent(5) {
            for rec in list {
                let display = rec.site_name.clone().unwrap_or_else(|| rec.label.clone());
                let label = format!("{} ({})", display, rec.url);
                let item = MenuItem::new(&label, true, None);
                let item_id = item.id();
                id_map.insert(item_id.clone(), TrayEvent::OpenUrl(rec.id));
                let _ = history_submenu.append_items(&[&item]);
                println!("[tray] added history menu item id={:?} -> url id={}", item_id, rec.id);
            }
        }

        let quit_item = MenuItem::new("Quitter Rustine", true, None);
        let _ = menu.append_items(&[
            &show_item,
            &add_item,
            &PredefinedMenuItem::separator(),
            &history_submenu,
            &PredefinedMenuItem::separator(),
            &quit_item,
        ]);

        println!("[tray] menu set: show_id={:?}, add_id={:?}, quit_id={:?}", show_item.id(), add_item.id(), quit_item.id());

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Rustine")
            .build()?;

        // Keep the tray alive for the process lifetime.
        let _static_tray_ref: &'static tray_icon::TrayIcon = Box::leak(Box::new(tray));

        println!("[tray] tray icon created on main thread and leaked to static lifetime");

        // Receiver for menu events (the crate will forward events when the
        // platform/main event loop runs, e.g., after the GUI is launched).
        let menu_rx = tray_icon::menu::MenuEvent::receiver();

        // Move id_map and the item ids into a background thread that will
        // forward menu events into our application channel. The actual menu
        // dispatch is performed by the global event loop, so reading from
        // `menu_rx` here is safe.
        let show_id = show_item.id().clone();
        let add_id = add_item.id().clone();
        let quit_id = quit_item.id().clone();
        let id_map = std::sync::Arc::new(id_map);

        thread::spawn(move || {
            while let Ok(ev) = menu_rx.recv() {
                println!("[tray] menu event id={:?}", ev.id);
                if let Some(action) = id_map.get(&ev.id) {
                    println!("[tray] mapped history item -> {:?}", action);
                    let _ = tx.send(action.clone());
                } else if ev.id == show_id {
                    println!("[tray] Show clicked");
                    let _ = tx.send(TrayEvent::Show);
                } else if ev.id == add_id {
                    println!("[tray] Add clicked");
                    let _ = tx.send(TrayEvent::Add);
                } else if ev.id == quit_id {
                    println!("[tray] Quit clicked");
                    let _ = tx.send(TrayEvent::Quit);
                } else {
                    println!("[tray] Unhandled menu event: {:?}", ev.id);
                }
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
