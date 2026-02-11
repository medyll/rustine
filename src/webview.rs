use anyhow::Result;
use once_cell::sync::OnceCell;
use crossbeam_channel::{unbounded, Sender};
use std::thread;
use url::Url;

type UserEvent = String;

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
        thread::spawn(move || {
            while let Ok(url) = rx.recv() {
                if let Err(e) = proxy.send_event(url.clone()) {
                    eprintln!("webview proxy send_event failed for {}: {}", url, e);
                }
            }
        });

        event_loop.run(move |event, _, control_flow| {
            *control_flow = tao::event_loop::ControlFlow::Wait;
            match event {
                tao::event::Event::UserEvent(url) => {
                    // Navigate the existing webview to the requested URL and log failures.
                    match webview.load_url(&url) {
                        Ok(()) => {
                            // success
                        }
                        Err(e) => {
                            eprintln!("webview failed to load URL {}: {}", url, e);
                        }
                    }
                }
                tao::event::Event::WindowEvent { event, .. } => match event {
                    tao::event::WindowEvent::CloseRequested => {
                        *control_flow = tao::event_loop::ControlFlow::Exit;
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
    tx.send(normalized.clone()).map_err(|e| anyhow::anyhow!("failed to send open_url: {}", e))?;
    Ok(())
}
