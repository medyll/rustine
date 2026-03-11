# PRD – rustine (webapp mode)

## Overview

rustine lets users transform websites into lightweight desktop webapps. Each webapp runs in an embedded, sandboxed webview inside rustine and can be installed, launched, and managed from the main application. On Windows, a resident tray icon enables quick actions: add a site, open a site (in-app), or open the main application UI.

## Goals & Success Metrics

| Goal | Metric | Target |
|---|---|---|
| Convert websites into usable desktop webapps | Number of installed webapps per user | >= 3 for engaged users (30 days) |
| Fast webapp launch | Time from action to webapp window shown | < 500ms (cold/warm depends on OS) |
| Tray-driven adoption | % of installs performed via tray quick-add | > 25% |
| Local privacy | No cloud storage for webapp configs by default | 100% local by default |

## User Personas

### Persona 1 – Power User (Mydde)
- Role: Developer / power user
- Needs: Convert dashboards and tools into app-like windows, open them quickly from tray, keep configs local
- Pain points: Too many browser tabs; no simple way to turn a site into a single-purpose window

## Primary Use Cases

### UC-01 – Install (Add) Webapp
**Actor:** User
**Trigger:** User selects "Add site" from tray or clicks "Install webapp" in main UI
**Flow:**
1. User inputs a URL or chooses "From current site"
2. App validates URL and creates a webapp entry
3. App fetches metadata (icon, title) and saves webapp config to SQLite
4. App optionally creates a desktop shortcut or an entry in the app list
**Expected outcome:** A new webapp appears in the app list and can be launched immediately

### UC-02 – Open Webapp (In-App)
**Actor:** User
**Trigger:** User selects a webapp from the main UI or tray menu
**Flow:**
1. App opens the webapp in an embedded, sandboxed webview window
2. Window presents site content with minimal chrome (back/forward/refresh when relevant)
3. User interacts with site as if it were a native app
**Expected outcome:** Webapp loads and is usable within 500ms (warm) / < 2s (cold)

### UC-03 – Tray Quick Actions
**Actor:** User
**Trigger:** User clicks system tray icon (Windows)
**Flow:**
1. Tray menu shows quick actions: Add site, Open app, Recent webapps
2. User chooses "Add site" to install quickly, or clicks a recent webapp to open it in-app
**Expected outcome:** Tray actions perform within 200ms for menu display and <500ms for launching

### UC-04 – Manage / Uninstall Webapp
**Actor:** User
**Trigger:** User opens main app UI and selects a webapp → Settings / Uninstall
**Flow:**
1. User edits name, icon, or launch options, or uninstalls the webapp
2. App updates SQLite and removes associated shortcuts
**Expected outcome:** Changes persist locally and take effect immediately

## Functional Requirements

| ID | Requirement | Priority | Notes |
|---|---|---|---|
| FR-01 | Install a website as a webapp (create managed entry) | Must | Accept URL or discovery flow |
| FR-02 | Manage installed webapps (list, edit, uninstall) | Must | Show icon, name, URL, launch options |
| FR-03 | Open webapp inside embedded webview by default | Must | Option to open externally in system browser |
| FR-04 | System tray icon with quick-add and quick-launch | Must | On Windows: menu includes Add site, Open app, recent webapps |
| FR-05 | Persist webapp configs locally (SQLite) | Must | Use rusqlite bundled feature |
| FR-06 | Fetch webapp metadata (icon, title) | Should | Fallback to placeholder icons if needed |
| FR-07 | Create OS-level shortcut (optional) | Could | Platform-specific behavior |
| FR-08 | Webview sandboxing and per-app permission prompts | Must | Prevent cross-app data leakage |

## Non-Functional Requirements

| Category | Requirement | Acceptance Criteria |
|---|---|---|
| Performance | Webapp launch latency | < 500ms warm, < 2000ms cold on typical hardware |
| Security | Webview content isolation | Each webapp runs in its own context; no shared cookies unless explicitly enabled |
| Privacy | Local-first storage | No telemetry or cloud sync by default |
| Compatibility | Windows 10+ priority | Support macOS/Linux with equivalent UI behaviors |

## Out of Scope

- Cloud sync / multi-device sync (v1)
- Mobile companion apps
- Built-in browser extension

## Dependencies

- Embedded webview engine (wry or equivalent)
- rusqlite for local persistence
- tray integration library for cross-platform tray/menu support

## Open Questions

- Should installs optionally register a protocol handler or OS shortcut automatically?
- What are the default per-webapp permission rules (media, notifications)?

## Revision History

| Date | Author | Change |
|---|---|---|
| 2026-03-07 | PM Agent | PRD updated to webapp/webview model and tray quick-actions |

