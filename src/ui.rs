use futures::stream::StreamExt;
use dioxus::prelude::*;
use dioxus::prelude::use_coroutine;
use chrono::Utc;




fn root() -> Element {
    // background tray printer (non-blocking, just logs events)
    use_effect(|| {
        if let Some(rx) = crate::tray::get_receiver() {
            std::thread::spawn(move || {
                while let Ok(ev) = rx.recv() {
                    println!("Received tray event in UI: {:?}", ev);
                }
            });
        }
        // Return unit, not a closure
    });

    let mut urls = use_signal(|| Vec::<crate::db::UrlRecord>::new());

    // Coroutine for async DB actions
    let db_coroutine = use_coroutine(move |mut rx| async move {
        while let Some(action) = rx.next().await {
            match action {
                DbAction::Load => {
                    if let Some(db) = crate::db::get_global() {
                        if let Ok(list) = db.list_recent(100) {
                            urls.set(list);
                        }
                    }
                }
                DbAction::Delete(id) => {
                    if let Some(db) = crate::db::get_global() {
                        let _ = db.delete(id);
                        if let Ok(list) = db.list_recent(100) {
                            urls.set(list);
                        }
                    }
                }
                DbAction::Insert(lab, urlv, ts) => {
                    if let Some(db) = crate::db::get_global() {
                        let _ = db.insert_url(&lab, &urlv, ts);
                        if let Ok(list) = db.list_recent(100) {
                            urls.set(list);
                        }
                    }
                }
            }
        }
    });

    // Load initial list once
    {
        let db_coroutine = db_coroutine.clone();
        use_effect(move || {
            db_coroutine.send(DbAction::Load);
        });
    }

    // input state for new URL
    let mut label_input = use_signal(|| String::new());
    let mut url_input = use_signal(|| String::new());

    let on_delete = {
        let db_coroutine = db_coroutine.clone();
        move |id: i64| {
            db_coroutine.send(DbAction::Delete(id));
        }
    };

    let current_urls = urls.with(|v| v.clone());
    let current_label = label_input.with(|s| s.clone());
    let current_url = url_input.with(|s| s.clone());

    rsx!(div { style: "padding:16px; font-family:Arial, sans-serif;",
        h1 { "Rustine — reactive list" }
        form { onsubmit: move |e| {
                e.prevent_default();
                    let lab = label_input.with(|s| s.clone());
                    let urlv = url_input.with(|s| s.clone());
                let db_coroutine = db_coroutine.clone();
                // send insert action with timestamp
                db_coroutine.send(DbAction::Insert(lab, urlv, Utc::now().timestamp()));
                // clear inputs
                label_input.set(String::new());
                url_input.set(String::new());
            },
            input { placeholder: "Label", value: "{current_label}", oninput: move |e| label_input.set(e.value().clone()) }
            input { placeholder: "URL", value: "{current_url}", oninput: move |e| url_input.set(e.value().clone()) }
            button { "Ajouter" }
        }
        ul {
            for rec in current_urls.iter().cloned() {
                li { style: "display:flex; gap:8px; align-items:center;",
                    span { "{rec.label} — {rec.url}" }
                    button { onclick: move |_| on_delete(rec.id), "Supprimer" }
                }
            }
        }
    })
}

// Helper enum for DB actions
enum DbAction {
    Load,
    Delete(i64),
    Insert(String, String, i64),
}

pub fn app() -> Element {
    root()
}
