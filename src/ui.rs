use futures::stream::StreamExt;
use dioxus::prelude::*;
use std::fs;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use dioxus::prelude::use_coroutine;
use chrono::Utc;
use url::Url;
// use crate::webview; -- calling via `crate::webview::open_url(...)`

fn root() -> Element {
    let mut urls = use_signal(|| Vec::<crate::db::UrlRecord>::new());

    // input state for new URL and error message
    let mut label_input = use_signal(|| String::new());
    let mut url_input = use_signal(|| String::new());
    let mut error_msg = use_signal(|| String::new());

    // selected screen for simple in-app navigation (None => list view)
    let mut selected_screen = use_signal(|| Option::<i64>::None);

    // Load compiled Tailwind CSS from assets at runtime and inject into the page.
    let style_css = use_signal(|| String::new());
    {
        let mut style_css = style_css.clone();
        use_effect(move || {
            let css = fs::read_to_string("assets/dist/styles.css").unwrap_or_default();
            style_css.set(css);
        });
    }

    // tray -> UI error channel: use a futures unbounded sender so the UI can await messages
    let (err_tx, err_rx) = futures::channel::mpsc::unbounded::<String>();

    if let Some(rx) = crate::tray::get_receiver() {
        let tx = err_tx.clone();
        std::thread::spawn(move || {
            while let Ok(ev) = rx.recv() {
                match ev {
                    crate::tray::TrayEvent::OpenUrl(id) => {
                        if let Some(db) = crate::db::get_global() {
                            match db.get_by_id(id) {
                                Ok(Some(rec)) => {
                                    let u = rec.url.clone();
                                    if let Err(e) = crate::webview::open_url(u) {
                                        let _ = tx.unbounded_send(format!("Erreur ouverture URL (tray): {}", e));
                                    }
                                }
                                Ok(None) => {
                                    let _ = tx.unbounded_send(format!("URL introuvable (id={})", id));
                                }
                                Err(e) => {
                                    let _ = tx.unbounded_send(format!("Erreur DB (tray): {}", e));
                                }
                            }
                        } else {
                            let _ = tx.unbounded_send("Base de données non disponible".to_string());
                        }
                    }
                    crate::tray::TrayEvent::Show => {
                        let _ = tx.unbounded_send("Tray: show clicked".to_string());
                    }
                    crate::tray::TrayEvent::Add => {
                        let _ = tx.unbounded_send("Tray: add clicked".to_string());
                    }
                    crate::tray::TrayEvent::Quit => {
                        let _ = tx.unbounded_send("Tray: quit requested".to_string());
                    }
                    // all TrayEvent variants handled above; no-op for others
                }
            }
        });
    }

    // Consume error messages on the UI async context and set the signal
    let mut err_rx_opt = Some(err_rx);

    use_future(move || {
        let err_rx_opt = err_rx_opt.take();
        async move {
            if let Some(mut rx) = err_rx_opt {
                while let Some(msg) = rx.next().await {
                    error_msg.set(msg);
                }
            }
        }
    });

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

    let mut on_details = {
        let mut selected = selected_screen.clone();
        move |id: i64| {
            selected.set(Some(id));
        }
    };

    let current_urls = urls.with(|v| v.clone());
    let current_label = label_input.with(|s| s.clone());
    let current_url = url_input.with(|s| s.clone());
    let current_error = error_msg.with(|s| s.clone());

    let style_content = style_css.with(|s| s.clone());

    rsx!(div { style: "padding:16px; font-family:Arial, sans-serif;",
        style { "{style_content}" }
        if let Some(screen_id) = selected_screen.with(|s| *s) {
            h1 { "Details" }
            p { "Screen id: {screen_id}" }
            button { onclick: move |_| selected_screen.set(None), "Back" }
        } else {
            h1 { "Rustine — reactive list" }
            form { onsubmit: move |e| {
                    e.prevent_default();
                    let lab = label_input.with(|s| s.clone()).trim().to_string();
                    let urlv = url_input.with(|s| s.clone()).trim().to_string();
                    // strict validation: non-empty, valid URL, http(s) scheme, has host
                    error_msg.set(String::new());
                    if lab.is_empty() {
                        error_msg.set("The label cannot be empty".to_string());
                        return;
                    }
                    if urlv.is_empty() {
                        error_msg.set("The URL cannot be empty".to_string());
                        return;
                    }
                    let parsed = match Url::parse(&urlv) {
                        Ok(u) => u,
                        Err(_) => {
                            error_msg.set("Invalid URL".to_string());
                            return;
                        }
                    };
                    let scheme = parsed.scheme();
                    if scheme != "http" && scheme != "https" {
                        error_msg.set("Only http(s) URLs are supported".to_string());
                        return;
                    }
                    if parsed.host().is_none() {
                        error_msg.set("The URL must contain a host".to_string());
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
                button { "Add" }
            }
            if !current_error.is_empty() {
                p { style: "color: #c00; margin-top:8px;", "{current_error}" }
            }
            ul {
                for rec in current_urls.iter().cloned() {
                    li { style: "display:flex; gap:8px; align-items:center;",
                        { if let Some(data) = rec.icon_data.clone() {
                            let mime = rec.icon_mime.clone().unwrap_or_else(|| "image/png".to_string());
                            let b64 = STANDARD.encode(&data);
                            let src = format!("data:{};base64,{}", mime, b64);
                            rsx!(img { src: "{src}", width: "16", height: "16", style: "border-radius:2px;" })
                        } else { rsx!() } }
                        a { href: "#", onclick: move |e| {
                                e.prevent_default();
                                let u = rec.url.clone();
                                if let Err(err) = crate::webview::open_url(u) {
                                    error_msg.set(format!("Error opening URL: {}", err));
                                }
                            }, "{rec.label} — {rec.url}" }
                        button { onclick: move |_| on_delete(rec.id), "Delete" }
                        button { onclick: move |_| on_details(rec.id), "Details" }
                    }
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
