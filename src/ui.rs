use dioxus::prelude::*;
use std::thread;
use url::Url;

fn url_valid(s: &str) -> bool {
    Url::parse(s).is_ok()
}

#[derive(Props, PartialEq)]
struct UrlItemProps {
    id: i64,
    label: String,
    url: String,
}

fn UrlItem(cx: Scope<UrlItemProps>) -> Element {
    let id = cx.props.id;
    let label = cx.props.label.clone();
    let url = cx.props.url.clone();

    let on_delete = move |_| {
        if let Some(db) = crate::db::get_global() {
            let id = id;
            thread::spawn(move || {
                if let Err(e) = db.delete(id) {
                    eprintln!("Failed to delete url {}: {}", id, e);
                }
            });
        }
    };

    cx.render(rsx! {
        div { style: "display:flex; align-items:center; gap:8px; margin:4px 0;", 
            a { href: "#", onclick: move |_| { println!("Open URL: {}", url); }, "{label}" }
            span { " — " }
            span { "{url}" }
            button { onclick:on_delete, "Supprimer" }
        }
    })
}

fn UrlList(cx: Scope) -> Element {
    let urls = use_state(cx, || Vec::<crate::db::UrlRecord>::new());

    // load recent items on mount (synchronous prototype)
    use_effect(cx, (), move |_| {
        if let Some(db) = crate::db::get_global() {
            match db.list_recent(100) {
                Ok(list) => urls.set(list),
                Err(e) => eprintln!("Failed to load urls: {}", e),
            }
        }
        async move {}
    });

    cx.render(rsx! {
        div {
            h2 { "Historique" }
            ul {
                for record in urls.get().iter() {
                    rsx!(li { UrlItem { id: record.id, label: record.label.clone(), url: record.url.clone() } })
                }
            }
        }
    })
}

fn UrlInput(cx: Scope) -> Element {
    let input = use_state(cx, String::new);
    let label = use_state(cx, String::new);

    let on_add = {
        let input = input.clone();
        let label = label.clone();
        move |_| {
            let url = input.get().to_string();
            let lab = label.get().to_string();
            if !url_valid(&url) {
                eprintln!("URL invalid: {}", url);
                return;
            }
            if let Some(db) = crate::db::get_global() {
                thread::spawn(move || {
                    let ts = chrono::Utc::now().timestamp();
                    if let Err(e) = db.insert_url(&lab, &url, ts) {
                        eprintln!("Failed to insert url: {}", e);
                    } else {
                        println!("Inserted {}", url);
                    }
                });
            }
        }
    };

    cx.render(rsx! {
        div { style: "display:flex; flex-direction:column; gap:8px;", 
            input { r#type: "text", placeholder: "Label", oninput: move |e| label.set(e.value.clone()) }
            input { r#type: "text", placeholder: "https://...", oninput: move |e| input.set(e.value.clone()) }
            button { onclick:on_add, "Ajouter URL" }
        }
    })
}

pub fn app(cx: Scope) -> Element {
    // listen for tray events (prints only) -- kept for scaffold
    let tray_events = crate::tray::get_receiver();
    use_effect(cx, (), move |_| {
        if let Some(rx) = tray_events {
            std::thread::spawn(move || {
                while let Ok(ev) = rx.recv() {
                    println!("Received tray event in UI scaffold: {:?}", ev);
                }
            });
        }
        async move {}
    });

    cx.render(rsx! {
        div { style: "padding:16px; font-family:Arial, sans-serif;", 
            h1 { "Rustine — prototype" }
            UrlInput {}
            UrlList {}
        }
    })
}
