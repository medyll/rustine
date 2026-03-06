# Tech Spec – rustine

## Stack

| Layer | Technology | Justification |
|---|---|---|
| UI Framework | Dioxus 0.7 | Cross-platform desktop, Rust-native |
| Backend | Rust (2021) | Performance, safety |
| Database | SQLite (rusqlite bundled) | Zero-config, local storage |
| HTTP Client | reqwest (blocking) | Fetch favicons/metadata |
| HTML Parsing | scraper | Parse HTML for metadata |
| System Tray | tray-icon | Native system tray |
| WebView | wry + tao | Window management |
| Styling | Tailwind CSS (PostCSS) | Rapid UI development |

## System Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                    main.rs                           │
│  Entry point: init DB → set global → start tray    │
└─────────────────────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │  db.rs  │    │ tray.rs │    │  ui.rs  │
    │ SQLite  │    │  Tray   │    │ Dioxus  │
    │ Actor   │    │ Menu    │    │   UI    │
    └─────────┘    └─────────┘    └─────────┘
                         │               │
                         ▼               │
                  ┌─────────────┐         │
                  │ webview.rs │◄────────┘
                  │ URL Launcher│
                  │ Metadata    │
                  │ Fetching    │
                  └─────────────┘
```

### Module Responsibilities

| Module | Responsibility | Public API |
|---|---|---|
| `main.rs` | App entry, initialization | None (entry point) |
| `db.rs` | SQLite persistence, CRUD | `init_db()`, `get_global()`, URL insert/list/delete |
| `tray.rs` | System tray icon + menu | `start_tray()`, `get_receiver()` |
| `ui.rs` | Dioxus UI components | `app` (root component) |
| `webview.rs` | URL opening, metadata fetch | `open_url()` |

## Data Model

### Entity: UrlRecord

| Field | Type | Required | Description |
|---|---|---|---|
| id | i64 | Yes | Primary key (auto-increment) |
| label | String | Yes | User-defined label |
| url | String | Yes | Target URL |
| _timestamp | i64 | Yes | Unix timestamp |
| site_name | Option<String> | No | Auto-fetched site name |
| icon_mime | Option<String> | No | Favicon MIME type |
| icon_data | Option<Vec<u8>> | No | Favicon binary data |

### Entity: SiteMeta

| Field | Type | Required | Description |
|---|---|---|---|
| id | i64 | Yes | Primary key |
| origin | String | Yes | Site origin (e.g., "https://github.com") |
| site_name | Option<String> | No | Site display name |
| description | Option<String> | No | Site description |
| manifest_url | Option<String> | No | PWA manifest URL |
| metadata_fetched_at | Option<i64> | No | Fetch timestamp |

### Entity: Icon

| Field | Type | Required | Description |
|---|---|---|---|
| id | i64 | Yes | Primary key |
| site_id | i64 | Yes | Foreign key to SiteMeta |
| src_url | String | Yes | Icon source URL |
| width | Option<i64> | No | Icon width |
| height | Option<i64> | No | Icon height |
| mime | Option<String> | No | MIME type |
| data | Vec<u8> | Yes | Icon binary data |
| fetched_at | Option<i64> | No | Fetch timestamp |

## API Design

### Database Actor (db.rs)

Using channel-based actor pattern:

```rust
// Commands sent to DB thread
enum DbRequest {
    Insert { label, url, timestamp, resp: Sender<Result<()>> },
    ListRecent { limit, resp: Sender<Vec<UrlRecord>> },
    Delete { id, resp: Sender<Result<()>> },
    // ... site metadata ops
}
```

### Tray Events (tray.rs)

```rust
enum TrayEvent {
    Show,           // Show main window
    Add,            // Open add URL dialog
    Quit,           // Quit application
    OpenUrl(i64),   // Open URL by ID
}
```

### UI Signals (ui.rs)

- `use_signal` for reactive state (URL list, input fields, selected screen)
- `use_effect` for loading CSS and initializing data
- `use_coroutine` for async operations

## Integration Points

| System | Type | Notes |
|---|---|---|
| System Browser | External | Uses `webbrowser` crate to open default browser |
| Favicon Server | HTTP | Fetches from target site /favicon.ico or parses HTML |
| System Tray | OS-native | Uses `tray-icon` crate |

## Security Considerations

- **No remote code execution**: All URLs must be valid format
- **No secrets stored**: No API keys, credentials, or sensitive data
- **Local data only**: SQLite file stays on local disk
- **Input validation**: URL format validated before storage
- **No user content execution**: App doesn't execute user-provided code

## Performance Considerations

- **Lazy metadata fetch**: Favicons fetched on-demand when URL is opened
- **In-memory caching**: Recent URLs kept in memory via DB actor
- **Blocking HTTP**: Uses `reqwest::blocking` to avoid async complexity
- **SQLite indexes**: On `url` column for lookups
- **Icon compression**: Favicons stored as-is (typically small)

## Open Technical Questions

- [ ] Should we add an index on `site_name` for search?
- [ ] What's the max icon size to store? (Currently unbounded)
- [ ] Should we cache metadata TTL? (Currently fetched each time)
- [ ] Error logging strategy? (Currently using `eprintln!`)
