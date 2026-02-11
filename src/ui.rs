use dioxus::prelude::*;
use std::thread;

pub fn app() -> Element {
    // simple v0.7-compatible scaffold: a button that inserts a demo URL and prints history
    if let Some(rx) = crate::tray::get_receiver() {
        thread::spawn(move || {
            while let Ok(ev) = rx.recv() {
                println!("Received tray event in UI: {:?}", ev);
            }
        });
    }

    let on_demo = move |_| {
        if let Some(db) = crate::db::get_global() {
            thread::spawn(move || {
                let ts = chrono::Utc::now().timestamp();
                if let Err(e) = db.insert_url("Demo", "https://example.com", ts) {
                    eprintln!("Failed to insert demo url: {}", e);
                } else if let Ok(list) = db.list_recent(10) {
                    println!("Recent URLs:");
                    for r in list { println!("- {} {}", r.label, r.url); }
                }
            });
        }
    };

    let on_delete_latest = move |_| {
        if let Some(db) = crate::db::get_global() {
            thread::spawn(move || {
                match db.list_recent(1) {
                    Ok(list) if !list.is_empty() => {
                        let id = list[0].id;
                        if let Err(e) = db.delete(id) {
                            eprintln!("Failed to delete id {}: {}", id, e);
                        } else {
                            println!("Deleted id {}", id);
                        }
                    }
                    Ok(_) => println!("No entries to delete"),
                    Err(e) => eprintln!("Failed to fetch latest for delete: {}", e),
                }
            });
        }
    };

    rsx!(div { style: "padding:16px; font-family:Arial, sans-serif;",
        h1 { "Rustine — prototype (v0.7 UI)" }
        button { onclick:on_demo, "Insert demo URL and print history" }
        button { onclick:on_delete_latest, "Delete latest" }
        p { "More UI features (reactive lists, inputs) can be added next." }
    })
}
