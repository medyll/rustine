# Product Brief – rustine

## Problem Statement

Users and teams need a simple, privacy-minded way to convert frequently-used websites into standalone desktop webapps (single-purpose windows with their own icon and window controls). Existing approaches (browser PWAs, manual shortcuts, or cloud services) are either browser-specific, hard to manage, or require cloud sync; there is no lightweight cross-platform desktop tool that installs, manages, and launches webapps locally with tray-based quick actions.

## Target Users

- Power users who want web tools to behave like native apps
- Teams using SaaS dashboards, chat, or admin consoles that benefit from single-purpose windows
- Privacy-focused users who prefer all configuration and metadata stored locally

## Expected Outcome

- Install a website as a standalone webapp with its own window and icon
- Manage installed webapps (open, edit settings, uninstall) from a central UI
- Quick-add and quick-launch from the Windows system tray: Add site, Open site (in-app), Open app window
- Local-first persistence (SQLite) with optional cached metadata (icon, display name)

## Scope (in / out)

| In Scope | Out of Scope |
|---|---|
| Install a website as a webapp (embedded webview) | Cloud sync / multi-device sync |
| Manage list of installed webapps (edit/uninstall) | User accounts / auth |
| Tray icon (Windows) with quick-add and quick-launch actions | Browser extension |
| Open webapp inside the app (embedded webview) | Native OS packaging/publishing (optional feature) |
| Persist webapp configs locally (SQLite) | Mobile companion app |

## Constraints

- Cross-platform desktop (Windows primary for tray behavior; macOS/Linux supported with equivalent UI)
- Tech stack: Rust + Dioxus + embedded webview (wry or equivalent)
- No cloud dependencies for core features; local-first design

## Stakeholders

| Name / Role | Involvement |
|---|---|
| Mydde (Owner/Dev) | Product owner, implements features |

## Open Questions

- Should an installed webapp be able to create an OS-level shortcut/icon by default?
- Which webview engine should be the default on each platform (security/performance tradeoffs)?
- Should per-webapp permission prompts be surfaced (camera/mic/notifications)?
