use futures::stream::StreamExt;
use dioxus::prelude::*;
use dioxus::prelude::use_coroutine;
use chrono::Utc;
use url::Url;
// use crate::webview; -- calling via `crate::webview::open_url(...)`




fn root() -> Element {
    // background tray printer (non-blocking, just logs events)
    use_effect(|| {
        if let Some(rx) = crate::tray::get_receiver() {
            std::thread::spawn(move || {
                while let Ok(ev) = rx.recv() {
                    match ev {
                        crate::tray::TrayEvent::OpenUrl(id) => {
                            if let Some(db) = crate::db::get_global() {
                                match db.get_by_id(id) {
                                    Ok(Some(rec)) => {
                                        let u = rec.url.clone();
                                        let _ = crate::webview::open_url(u);
                                    }
                                    Ok(None) => eprintln!("URL id not found: {}", id),
                                    Err(e) => eprintln!("DB error getting url {}: {}", id, e),
                                }
                            }
                        }
                        other => println!("Received tray event in UI: {:?}", other),
                    }
                }
            });
        }
        // Return unit, not a closure
    });

    let mut urls = use_signal(|| Vec::<crate::db::UrlRecord>::new());

    // input state for new URL and error message
    let mut label_input = use_signal(|| String::new());
    let mut url_input = use_signal(|| String::new());
    let mut error_msg = use_signal(|| String::new());

    // Coroutine for async DB actions (created after signals so it can capture them)
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
                        match db.insert_url(&lab, &urlv, ts) {
                            Ok(()) => {
                                // clear any previous error and refresh
                                error_msg.set(String::new());
                                if let Ok(list) = db.list_recent(100) {
                                    urls.set(list);
                                }
                            }
                            Err(e) => {
                                error_msg.set(format!("Erreur DB: {}", e));
                            }
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

    let on_delete = {
        let db_coroutine = db_coroutine.clone();
        move |id: i64| {
            db_coroutine.send(DbAction::Delete(id));
        }
    };

    let current_urls = urls.with(|v| v.clone());
    let current_label = label_input.with(|s| s.clone());
    let current_url = url_input.with(|s| s.clone());
    let current_error = error_msg.with(|s| s.clone());

    rsx!(div { style: "padding:16px; font-family:Arial, sans-serif;",
        h1 { "Rustine — reactive list" }
        form { onsubmit: move |e| {
                e.prevent_default();
                let lab = label_input.with(|s| s.clone()).trim().to_string();
                let urlv = url_input.with(|s| s.clone()).trim().to_string();
                // strict validation: non-empty, valid URL, http(s) scheme, has host
                error_msg.set(String::new());
                if lab.is_empty() {
                    error_msg.set("Le label ne peut pas être vide".to_string());
                    return;
                }
                if urlv.is_empty() {
                    error_msg.set("L'URL ne peut pas être vide".to_string());
                    return;
                }
                let parsed = match Url::parse(&urlv) {
                    Ok(u) => u,
                    Err(_) => {
                        error_msg.set("URL invalide".to_string());
                        return;
                    }
                };
                let scheme = parsed.scheme();
                if scheme != "http" && scheme != "https" {
                    error_msg.set("Seules les URLs http(s) sont autorisées".to_string());
                    return;
                }
                if parsed.host().is_none() {
                    error_msg.set("L'URL doit contenir un hôte".to_string());
                    return;
                }
                // normalize URL (e.g. add trailing slash if parsed)
                let normalized = Into::<String>::into(parsed);
                let db_coroutine = db_coroutine.clone();
                // send insert action with timestamp
                db_coroutine.send(DbAction::Insert(lab, normalized, Utc::now().timestamp()));
                // clear inputs (error cleared by coroutine on success)
                label_input.set(String::new());
                url_input.set(String::new());
            },
            input { placeholder: "Label", value: "{current_label}", oninput: move |e| label_input.set(e.value().clone()) }
            input { placeholder: "URL", value: "{current_url}", oninput: move |e| url_input.set(e.value().clone()) }
            button { "Ajouter" }
        }
        if !current_error.is_empty() {
            p { style: "color: #c00; margin-top:8px;", "{current_error}" }
        }
        ul {
            for rec in current_urls.iter().cloned() {
                li { style: "display:flex; gap:8px; align-items:center;",
                    a { href: "#", onclick: move |e| {
                            e.prevent_default();
                            let u = rec.url.clone();
                            let _ = crate::webview::open_url(u);
                        }, "{rec.label} — {rec.url}" }
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
