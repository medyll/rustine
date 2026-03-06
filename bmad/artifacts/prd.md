# PRD – rustine

## Overview

rustine is a cross-platform desktop application for managing quick-access URLs. It provides a native desktop interface to store, organize, and launch frequently-used URLs with auto-fetched metadata (favicons, site names). The app runs in the system tray for quick access and persists data locally in SQLite.

## Goals & Success Metrics

| Goal | Metric | Target |
|---|---|---|
| Launch URLs quickly | Time from click to browser open | < 500ms |
| Auto-fetch metadata | % of URLs with favicon displayed | > 90% |
| Offline capability | App launches without internet | 100% |
| Cross-platform | Runs on Windows/macOS/Linux | All three |

## User Personas

### Persona 1 – Power User (Mydde)
- Role: Developer, productivity-focused
- Needs: Fast URL access without browser, visual favicons for quick recognition
- Pain points: Browser bookmarks get lost, need to search, no offline access

## Use Cases

### UC-01 – Add New URL
**Actor:** User
**Trigger:** User clicks "Add" button or uses tray menu
**Flow:**
1. User enters URL (e.g., "github.com")
2. App validates URL format
3. App fetches metadata (favicon, site name) in background
4. App saves URL + metadata to SQLite
5. UI updates to show new URL with icon
**Expected outcome:** New URL appears in list within 2 seconds
**Edge cases:** Invalid URL → show error; metadata fetch fails → save URL anyway, show placeholder

### UC-02 – Launch URL
**Actor:** User
**Trigger:** User clicks URL in list or selects from tray menu
**Flow:**
1. User clicks URL entry
2. App opens system default browser with URL
**Expected outcome:** Browser opens within 500ms
**Edge cases:** URL invalid → show error toast

### UC-03 – Delete URL
**Actor:** User
**Trigger:** User right-clicks URL → Delete
**Flow:**
1. User selects URL
2. User confirms deletion
3. App removes URL + metadata from SQLite
4. UI updates
**Expected outcome:** URL removed from list immediately

### UC-04 – View URL List
**Actor:** User
**Trigger:** User opens app window
**Flow:**
1. App loads all URLs from SQLite
2. Displays list with favicons, labels, timestamps
**Expected outcome:** List loads within 100ms for < 100 URLs

### UC-05 – System Tray Quick Access
**Actor:** User
**Trigger:** User clicks tray icon
**Flow:**
1. Tray menu shows recent URLs
2. User clicks URL to launch
3. Browser opens
**Expected outcome:** Tray menu appears within 200ms

## Functional Requirements

| ID | Requirement | Priority | Notes |
|---|---|---|---|
| FR-01 | Add URL with optional label | Must | Auto-validate URL format |
| FR-02 | Display URL list with favicons | Must | Fallback to placeholder if fetch fails |
| FR-03 | Launch URL in system browser | Must | Use webbrowser crate |
| FR-04 | Delete URL | Must | Confirm before delete |
| FR-05 | System tray icon with menu | Must | Show recent URLs |
| FR-06 | Auto-fetch favicon from site | Should | Try /favicon.ico first, then parse HTML |
| FR-07 | Auto-fetch site name from meta | Should | Parse og:title or <title> |
| FR-08 | Persist to local SQLite | Must | Use rusqlite with bundled feature |
| FR-09 | Window minimize to tray | Should | Close button minimizes to tray |
| FR-10 | Edit URL label | Could | Future version |

## Non-Functional Requirements

| Category | Requirement | Acceptance Criteria |
|---|---|---|
| Performance | App startup time | < 2 seconds cold start |
| Performance | URL list load | < 100ms for 100 URLs |
| Performance | Browser launch | < 500ms from click |
| Security | No remote data exfiltration | All data stays local |
| Security | No secrets in code | No API keys or credentials |
| Reliability | Graceful metadata failure | App works even if favicon fetch fails |
| Compatibility | Windows 10+ | App runs without admin |
| Compatibility | macOS 11+ | App runs without admin |
| Compatibility | Linux (Ubuntu 20.04+) | App runs without admin |

## Out of Scope

- Cloud sync / backup
- User accounts / authentication
- Browser extension
- Mobile companion app
- URL sharing / collaboration
- Advanced tagging / categories (v1)
- Import / export (v1)

## Dependencies

- **System browser**: Uses system default browser (webbrowser crate)
- **SQLite**: Bundled via rusqlite (no external DB required)
- **No external APIs**: All metadata fetched via HTTP from target sites

## Open Questions

- [ ] Should the app start minimized to tray on launch?
- [ ] Should the window close button minimize to tray or quit the app?
- [ ] Is there a maximum number of URLs to store?
- [ ] Should we add keyboard shortcuts (e.g., Ctrl+N for new URL)?

## Revision History

| Date | Author | Change |
|---|---|---|
| 2026-03-06 | PM Agent | Initial draft from product brief |
