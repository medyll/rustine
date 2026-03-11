# ADR-09 — Packaging & Distribution Strategy

Status: proposed

Context
---------
This is a local desktop app (Windows/macOS/Linux). We need a repeatable packaging strategy to produce user-friendly installers or archives and a reproducible CI pipeline to produce release artifacts.

Decision
---------
Adopt a pragmatic, staged approach:

1. Short-term (first releases): build release binaries for each platform and publish zipped artifacts or platform-idiomatic archives to GitHub Releases (CI produces artifacts).
2. Medium-term: add platform packaging where practical:
   - Linux: AppImage (or .deb via `cargo-deb`) for single-file distribution.
   - Windows: zip of executable or MSI via WiX / `cargo-wix` for an installer.
   - macOS: DMG or zip containing the .app bundle.
3. Ensure assets are included: the Tailwind-built `dist/styles.css` and application icons; add packaging scripts to assemble final bundles.

Consequences
------------
- Pros: quick release path, platform-appropriate packages, reproducible CI artifacts.
- Cons: adds CI complexity and packaging tooling dependencies.

Implementation Notes
--------------------
- Add a GitHub Actions workflow (or extend existing CI) that builds release binaries for each runner and assembles artifacts.
- Add packaging scripts under `scripts/` to create AppImage/MSI/DMG as needed.
- Ensure CSS build (`pnpm run build:css`) runs before packaging and that `assets/` are copied into the bundle.

Acceptance Criteria
-------------------
- CI produces platform release artifacts with CSS and icons included.
- Packaging scripts exist and are documented in the README.

Next Steps / Follow-ups
----------------------
- Dev story: implement CI release workflow, add packaging scripts, test installers on each platform.

(end)
