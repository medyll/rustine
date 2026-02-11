use anyhow::Result;
use once_cell::sync::OnceCell;
use crossbeam_channel::{unbounded, Sender};
use std::thread;
use url::Url;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::db;

#[derive(Clone)]
enum UserEvent {
    Navigate(String),
    Favicon(Vec<u8>),
}

static WEBVIEW_TX: OnceCell<Sender<UserEvent>> = OnceCell::new();

fn start_manager() -> Sender<UserEvent> {
    if let Some(tx) = WEBVIEW_TX.get() {
        return tx.clone();
    }

    let (tx, rx) = unbounded::<UserEvent>();

    // Spawn the thread that runs the tao event loop and owns the webview.
    thread::spawn(move || {
        // Build event loop (allow any_thread on Windows).
        #[cfg(target_os = "windows")]
        let event_loop: tao::event_loop::EventLoop<UserEvent> = {
            use tao::event_loop::EventLoopBuilder;
            use tao::platform::windows::EventLoopBuilderExtWindows;
            EventLoopBuilder::with_user_event()
                .with_any_thread(true)
                .build()
        };

        #[cfg(not(target_os = "windows"))]
        let event_loop: tao::event_loop::EventLoop<UserEvent> = {
            use tao::event_loop::EventLoopBuilder;
            EventLoopBuilder::with_user_event().build()
        };

        let window = tao::window::WindowBuilder::new()
            .with_title("Rustine — webview")
            .build(&event_loop)
            .expect("failed to build window");

        let webview = wry::WebViewBuilder::new()
            .with_url("about:blank")
            .build(&window)
            .expect("failed to build webview");

        let proxy = event_loop.create_proxy();

        // Forward channel messages into the event loop as user events.
        let proxy_for_channel = proxy.clone();
        thread::spawn(move || {
            while let Ok(ev) = rx.recv() {
                if let Err(e) = proxy_for_channel.send_event(ev.clone()) {
                    match ev {
                        UserEvent::Navigate(ref u) => eprintln!("webview proxy send_event failed for {}: {}", u, e),
                        UserEvent::Favicon(_) => eprintln!("webview proxy send_event failed for favicon: {}", e),
                    }
                }
            }
        });

    // Keep an Arc to the window so we can set the icon later when favicon arrives
    let window = Arc::new(window);

        // Move the proxy into the run closure so we can clone it for fetch threads
        let proxy_for_run = proxy.clone();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = tao::event_loop::ControlFlow::Wait;
            match event {
                tao::event::Event::UserEvent(user_ev) => match user_ev {
                    UserEvent::Navigate(url) => {
                        // Navigate the existing webview to the requested URL and log failures.
                        if let Err(e) = webview.load_url(&url) {
                            eprintln!("webview failed to load URL {}: {}", url, e);
                        } else {
                            // Ensure the webview window is visible and focused when navigating.
                            let _ = window.set_visible(true);
                            let _ = window.set_focus();
                            // spawn a background fetch for favicon for this url
                            let proxy_clone = proxy_for_run.clone();
                            let _window_clone = window.clone();
                            thread::spawn(move || {
                                // Try /favicon.ico first
                                if let Ok(parsed) = Url::parse(&url) {
                                    if let Some(host) = parsed.host_str() {
                                        let scheme = parsed.scheme();
                                        let port = parsed.port_or_known_default();
                                        let origin = if let Some(p) = port { format!("{}://{}:{}", scheme, host, p) } else { format!("{}://{}", scheme, host) };
                                        let fav_url = format!("{}/favicon.ico", origin.trim_end_matches('/'));
                                        if let Ok(resp) = reqwest::blocking::get(&fav_url) {
                                            if resp.status().is_success() {
                                                if let Ok(bytes) = resp.bytes() {
                                                                let vec = bytes.to_vec();
                                                                // Persist site metadata and icon (best-effort)
                                                                if let Some(dbh) = db::get_global() {
                                                                    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).ok();
                                                                    let _ = dbh.upsert_site_meta(&origin, None, None, None, now);
                                                                    if let Ok(Some(site)) = dbh.get_site_meta_by_origin(&origin) {
                                                                        let _ = dbh.insert_icon(site.id, &fav_url, None, None, None, vec.clone(), now);
                                                                    }
                                                                }
                                                                let _ = proxy_clone.send_event(UserEvent::Favicon(vec));
                                                    return;
                                                }
                                            }
                                        }

                                        // If /favicon.ico failed, fetch page HTML and try <link rel> icons
                                        if let Ok(page_resp) = reqwest::blocking::get(&url) {
                                            if page_resp.status().is_success() {
                                                if let Ok(text) = page_resp.text() {
                                                            use scraper::{Html, Selector};
                                                            let doc = Html::parse_document(&text);

                                                            // Try to find a web manifest first: <link rel="manifest" href="...">
                                                            let manifest_sel = Selector::parse("link[rel]").unwrap();
                                                            let mut manifest_found: Option<String> = None;
                                                            for element in doc.select(&manifest_sel) {
                                                                if let Some(rel) = element.value().attr("rel") {
                                                                    if rel.to_lowercase().contains("manifest") {
                                                                        if let Some(href) = element.value().attr("href") {
                                                                            if let Ok(manifest_url) = parsed.join(href) {
                                                                                manifest_found = Some(manifest_url.to_string());
                                                                                break;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }

                                                            if let Some(manifest_url_str) = manifest_found {
                                                                if let Ok(manifest_resp) = reqwest::blocking::get(&manifest_url_str) {
                                                                    if manifest_resp.status().is_success() {
                                                                        if let Ok(manifest_text) = manifest_resp.text() {
                                                                            // parse manifest json
                                                                            if let Ok(man) = serde_json::from_str::<serde_json::Value>(&manifest_text) {
                                                                                let name = man.get("name").and_then(|v| v.as_str()).map(|s| s.to_string())
                                                                                    .or_else(|| man.get("short_name").and_then(|v| v.as_str()).map(|s| s.to_string()));
                                                                                if let Some(dbh) = db::get_global() {
                                                                                    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).ok();
                                                                                    let _ = dbh.upsert_site_meta(&origin, name.as_deref(), None, Some(&manifest_url_str), now);
                                                                                }

                                                                                if let Some(icons) = man.get("icons").and_then(|v| v.as_array()) {
                                                                                    for icon_entry in icons.iter() {
                                                                                        if let Some(src) = icon_entry.get("src").and_then(|v| v.as_str()) {
                                                                                            if let Ok(icon_url) = parsed.join(src) {
                                                                                                if let Ok(icon_resp) = reqwest::blocking::get(icon_url.as_str()) {
                                                                                                    if icon_resp.status().is_success() {
                                                                                                        let mime = icon_resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).map(|s| s.to_string());
                                                                                                        if let Ok(bytes) = icon_resp.bytes() {
                                                                                                            let vec = bytes.to_vec();
                                                                                                            if let Some(dbh) = db::get_global() {
                                                                                                                let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).ok();
                                                                                                                let _ = dbh.upsert_site_meta(&origin, name.as_deref(), None, Some(&manifest_url_str), now);
                                                                                                                if let Ok(Some(site)) = dbh.get_site_meta_by_origin(&origin) {
                                                                                                                    let _ = dbh.insert_icon(site.id, icon_url.as_str(), None, None, mime.as_deref(), vec.clone(), now);
                                                                                                                }
                                                                                                            }
                                                                                                            let _ = proxy_clone.send_event(UserEvent::Favicon(vec));
                                                                                                            return;
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }

                                                            // Fallback: look for <link rel="icon"> or similar
                                                            let sel = Selector::parse("link[rel]").unwrap();
                                                            for element in doc.select(&sel) {
                                                                if let Some(rel) = element.value().attr("rel") {
                                                                    let rel_l = rel.to_lowercase();
                                                                    if rel_l.contains("icon") {
                                                                        if let Some(href) = element.value().attr("href") {
                                                                            if let Ok(icon_url) = parsed.join(href) {
                                                                                if let Ok(icon_resp) = reqwest::blocking::get(icon_url.as_str()) {
                                                                                    if icon_resp.status().is_success() {
                                                                                        if let Ok(bytes) = icon_resp.bytes() {
                                                                                            let vec = bytes.to_vec();
                                                                                            if let Some(dbh) = db::get_global() {
                                                                                                let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).ok();
                                                                                                let _ = dbh.upsert_site_meta(&origin, None, None, None, now);
                                                                                                if let Ok(Some(site)) = dbh.get_site_meta_by_origin(&origin) {
                                                                                                    let _ = dbh.insert_icon(site.id, icon_url.as_str(), None, None, None, vec.clone(), now);
                                                                                                }
                                                                                            }
                                                                                            let _ = proxy_clone.send_event(UserEvent::Favicon(vec));
                                                                                            return;
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                    UserEvent::Favicon(bytes) => {
                        // Try to decode image and set as window icon
                        match image::load_from_memory(&bytes) {
                            Ok(img) => {
                                let rgba = img.to_rgba8();
                                let (w, h) = (rgba.width(), rgba.height());
                                let raw = rgba.into_raw();
                                if let Ok(icon) = tao::window::Icon::from_rgba(raw, w, h) {
                                    // set icon on window
                                    window.set_window_icon(Some(icon));
                                }
                            }
                            Err(e) => eprintln!("failed to decode favicon: {}", e),
                        }
                    }
                },
                tao::event::Event::WindowEvent { event, .. } => match event {
                    tao::event::WindowEvent::CloseRequested => {
                        // Prevent closing the main application when the webview window is closed.
                        // Instead of exiting the event loop (which can terminate the app on
                        // some platforms/configurations), just hide the webview window.
                        let _ = window.set_visible(false);
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    });

    let _ = WEBVIEW_TX.set(tx.clone());
    tx
}

pub fn open_url(url: String) -> Result<()> {
    // Normalize the URL: trim, and if no scheme present, default to http://
    let s = url.trim();
    if s.is_empty() {
        return Err(anyhow::anyhow!("empty url"));
    }
    let normalized = if s.starts_with("http://") || s.starts_with("https://") || s.starts_with("about:") || s.starts_with("data:") || s.starts_with("file:") {
        s.to_string()
    } else {
        // Try with http:// prefix
        let candidate = format!("http://{}", s);
        match Url::parse(&candidate) {
            Ok(_) => candidate,
            Err(_) => return Err(anyhow::anyhow!("invalid url after normalization: {}", s)),
        }
    };

    let tx = start_manager();
    tx.send(UserEvent::Navigate(normalized.clone())).map_err(|e| anyhow::anyhow!("failed to send open_url: {}", e))?;
    Ok(())
}
