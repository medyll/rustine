# Architecture – rustine

## Chosen Approach

**Single-Process Desktop App with Channel-Based Actor Pattern**

A native desktop application where all components run in a single process. The database runs in its own thread with a channel-based actor pattern for thread-safe communication. The UI uses Dioxus for cross-platform rendering, and system tray provides background access.

**Justification:** For a personal URL manager with no scaling requirements, this approach minimizes complexity while providing good isolation between UI and data layers. The actor pattern ensures thread safety without requiring async runtime.

---

## System Context Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                        User                                   │
└──────────────────────────┬───────────────────────────────────┘
                           │ interacts
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                      rustine App                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────┐   │
│  │   UI Layer │◄──│  Tray Menu │   │  WebView/URL   │   │
│  │  (Dioxus)  │   │ (tray-icon)│   │   Launcher     │   │
│  └──────┬──────┘   └─────────────┘   └────────┬────────┘   │
│         │                                     │              │
│         │ channels                           │              │
│         ▼                                    ▼              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              DB Actor (db.rs)                       │   │
│  │         SQLite + rusqlite (bundled)                 │   │
│  └─────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                   External Systems                           │
│  ┌─────────────────┐    ┌────────────────────────────┐      │
│  │  System Browser │    │  Target Websites          │      │
│  │  (default)      │    │  (favicon fetching)      │      │
│  └─────────────────┘    └────────────────────────────┘      │
└──────────────────────────────────────────────────────────────┘
```

---

## Components

### main.rs
- **Responsibility:** App entry point, initialization orchestration
- **Technology:** Rust standard library
- **Interfaces:** None (entry point)
- **Scaling strategy:** N/A (single instance)

### ui.rs (Dioxus UI)
- **Responsibility:** Render URL list, handle user input, manage application state
- **Technology:** Dioxus 0.7, Tailwind CSS
- **Interfaces:** 
  - Exposes: URL list, forms for add/edit
  - Consumes: DB commands via channels
- **Scaling strategy:** N/A (single-user desktop app)

### db.rs (Database Actor)
- **Responsibility:** Persist URLs, site metadata, icons; handle CRUD operations
- **Technology:** rusqlite (bundled SQLite), crossbeam channels
- **Interfaces:**
  - Exposes: `insert_url`, `list_recent`, `delete_url`, `upsert_site_meta`, `insert_icon`
  - Consumes: `DbRequest` enum via channel
- **Scaling strategy:** Local file-based SQLite; adequate for <10,000 URLs

### tray.rs (System Tray)
- **Responsibility:** Provide background access, quick menu, show/hide window
- **Technology:** tray-icon crate, crossbeam channels
- **Interfaces:**
  - Exposes: `TrayEvent` enum (Show, Add, Quit, OpenUrl)
  - Consumes: Events from system tray
- **Scaling strategy:** N/A

### webview.rs (URL Launcher)
- **Responsibility:** Open URLs in system browser, fetch favicons and site metadata
- **Technology:** wry, tao, reqwest (blocking), scraper
- **Interfaces:**
  - Exposes: `open_url(url: String)`
- **Scaling strategy:** Thread-per-fetch for metadata; blocks on HTTP

---

## Architecture Decisions (ADR)

### ADR-01 – Channel-Based Database Actor
- **Status:** Accepted
- **Context:** Need thread-safe DB access from UI thread without async complexity
- **Decision:** Use crossbeam channels to communicate with a dedicated DB thread. All DB operations happen in one thread, eliminating concurrency issues.
- **Consequences:** 
  - Pro: Simple, no async runtime needed
  - Pro: Thread-safe by design
  - Con: DB operations are serialized (acceptable for single-user app)

### ADR-02 – System Tray as Primary Access Point
- **Status:** Accepted
- **Context:** User wants quick access to URLs without opening main window each time
- **Decision:** App minimizes to system tray; tray menu shows recent URLs for quick launch
- **Consequences:**
  - Pro: Fast access to URLs
  - Pro: App runs in background
  - Con: Requires platform-specific tray icon handling

### ADR-03 – Embedded WebView for Metadata Fetching
- **Status:** Accepted (with fallback)
- **Context:** Need to fetch favicons and site names from target websites
- **Decision:** Use reqwest blocking HTTP to fetch favicon.ico first, then parse HTML for link tags if needed. Fall back to system browser for URL opening.
- **Consequences:**
  - Pro: No embedded webview complexity
  - Pro: Works offline once icons are cached
  - Con: Some sites may block automated fetching

### ADR-04 – Feature-Gated Tray Implementation
- **Status:** Accepted
- **Context:** Tray functionality may not work on all platforms/configurations
- **Decision:** Gate tray behind `real_tray` feature flag; default enabled but can be disabled
- **Consequences:**
  - Pro: Allows testing without tray
  - Pro: Easier CI testing
  - Con: Additional complexity in build

---

## Data Flow

### Flow: Add New URL
1. User enters URL in UI form
2. UI sends `DbRequest::Insert` to DB channel
3. DB thread validates URL, inserts into SQLite
4. DB thread responds with success
5. UI refreshes URL list
6. (Background) UI triggers metadata fetch via webview module
7. webview spawns thread to fetch favicon
8. Favicon fetched and stored in DB
9. UI receives event to refresh icon display

### Flow: Launch URL
1. User clicks URL in list (or selects from tray menu)
2. UI calls `webview::open_url(url)`
3. webview creates/runs hidden tao event loop with wry webview
4. WebView navigates to URL
5. (Alternative) Uses `webbrowser` crate to open system default

### Flow: System Tray Quick Launch
1. User clicks tray icon
2. Tray menu opens with recent URLs
3. User selects URL
4. Tray module sends `TrayEvent::OpenUrl(id)` to UI
5. UI retrieves URL from DB
6. UI calls `webview::open_url()`

---

## Deployment Architecture

| Environment | Infrastructure | Notes |
|---|---|---|
| Dev | Local machine | `cargo run` or IDE |
| Release | Local machine | `cargo build --release` |
| Distribution | Manual | Platform-specific installers (MSI, DMG, AppImage) |

**Note:** This is a locally-installed desktop app, not a server application. No cloud deployment.

---

## Cross-Cutting Concerns

### Security
- **Data at rest:** SQLite file stored in user's app data directory
- **No secrets:** No API keys, credentials, or sensitive data stored
- **Input validation:** URL format validated before storage
- **No remote code:** App doesn't execute user-provided content

### Observability
- **Logging:** Currently using `eprintln!` for errors; no structured logging
- **Metrics:** None (not needed for local app)
- **Tracing:** None (not needed for local app)
- **Alerting:** N/A

### Resilience
- **Retry strategy:** Metadata fetch failures are silent; URL still usable
- **Circuit breaker:** N/A
- **Backup/DR:** User can back up `rustine.db` manually

### Error Handling
- **Database errors:** Logged, UI shows error toast
- **Network errors:** Silently ignored for metadata; core URL launch works offline
- **Invalid URLs:** Validation error shown to user before save

---

## Open Architectural Questions

- [ ] Should we add structured logging (e.g., `log` crate + file output)?
- [ ] Should we add crash reporting / error telemetry?
- [ ] Should the app check for updates?
- [ ] Should we add an optional sync mechanism (local network or file-based)?
