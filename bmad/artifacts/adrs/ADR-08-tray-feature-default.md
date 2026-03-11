# ADR-08 — Default for `real_tray` Feature

Status: proposed

Context
---------
Tray/menu integration is gated behind the `real_tray` feature. The audit noted the feature is enabled by default which can complicate CI and headless testing environments (AUDIT-007).

Decision
---------
Make `real_tray` an optional feature and disable it in the default feature set. Document how to enable it for developer machines and for packaged releases.

Rationale:
- CI and headless environments should be able to build and run tests without platform-specific tray dependencies or requiring a GUI environment.
- Packaged user-facing builds can enable `real_tray` during packaging or in release manifests.

Consequences
------------
- Pros: simpler CI, fewer platform-specific build failures.
- Cons: local developer experience requires enabling the feature to get tray functionality; documentation must be clear.

Implementation Notes
--------------------
- Update `Cargo.toml` to make `real_tray` optional (remove from default features).
- Update README with `--features real_tray` instructions and document how packaging enables the feature.

Acceptance Criteria
-------------------
- Default builds (CI) compile without `real_tray` and pass tests.
- README documents how to enable tray support locally and in release builds.

Next Steps / Follow-ups
----------------------
- Dev story: update `Cargo.toml` and CI templates; run a full CI build to validate.

(end)
