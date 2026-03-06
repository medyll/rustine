# Product Brief – rustine

## Problem Statement

Users need quick, organized access to frequently-used URLs. Existing solutions are either browser-based (bookmarks get lost, require browser), web-based (need internet to discover), or simple lists (no metadata, no visual feedback). The user wants a **fast, offline-capable desktop app** to store, organize, and launch URLs with auto-fetched metadata (favicons, site names).

## Target Users

- Power users who frequently access a fixed set of URLs
- Developers and productivity-focused individuals
- Users who prefer native desktop apps over browser bookmarks

## Expected Outcome

- **Launch URLs instantly** from a native desktop UI
- **Auto-metadata**: fetch and display site favicons and names automatically
- **System tray integration**: background access, quick-add functionality
- **Offline-first**: works without internet (metadata cached in local SQLite DB)

## Scope (in / out)

| In Scope | Out of Scope |
|---|---|
| Add/edit/delete URLs | Cloud sync |
| Auto-fetch favicons + site names | User accounts/auth |
| System tray with quick menu | Browser extension |
| Desktop notifications | Mobile companion |
| Open URLs in system browser | URL sharing/collaboration |
| Local SQLite persistence | Advanced tagging/categories (v1) |

## Constraints

- **Platform**: Windows/macOS/Linux (cross-platform desktop)
- **Tech Stack**: Rust + Dioxus (already chosen)
- **Timeline**: MVP - ship when functional
- **No external services**: all local, no cloud dependencies

## Stakeholders

| Name / Role | Involvement |
|---|---|
| Mydde (Owner/Dev) | Primary user, defines features, implements |

## Open Questions

- Should URLs support custom labels/categories in v1?
- Preferred default browser behavior on each platform?
- Export/import functionality needed?
