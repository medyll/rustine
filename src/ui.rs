use dioxus::prelude::*;
use std::thread;



#[component]
fn Root(cx: Scope) -> Element {
    // background tray printer
    if let Some(rx) = crate::tray::get_receiver() {
        thread::spawn(move || {
            while let Ok(ev) = rx.recv() {
                println!("Received tray event in UI: {:?}", ev);
            }
        });
    }

    let urls = use_signal(|| Vec::<crate::db::UrlRecord>::new());

    // load initial list once
    {
        let urls = urls.clone();
        use_effect(move || {
            if let Some(db) = crate::db::get_global() {
                thread::spawn(move || {
                    if let Ok(list) = db.list_recent(100) {
                        urls.set(list);
                    }
                });
            }
        });
    }

    // snapshot for iteration
    let current_urls = urls.with(|v| v.clone());

    cx.render(rsx!(div { style: "padding:16px; font-family:Arial, sans-serif;",
        h1 { "Rustine — reactive list" }
        ul {
            for rec in current_urls.iter().cloned() {
                li { style: "display:flex; gap:8px; align-items:center;",
                    span { "{rec.label} — {rec.url}" }
                    button { onclick: move |_| {
                        if let Some(db) = crate::db::get_global() {
                            let id = rec.id;
                            let urls = urls.clone();
                            thread::spawn(move || {
                                if let Err(e) = db.delete(id) {
                                    eprintln!("Failed to delete {}: {}", id, e);
                                }
                                if let Ok(list) = db.list_recent(100) {
                                    urls.set(list);
                                }
                            });
                        }
                    }, "Supprimer" }
                }
            }
        }
    }))
}


pub fn app(cx: Scope) -> Element {
    rsx!(Root { cx })
}
