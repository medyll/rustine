# ADR-06 — Database File Location & Migrations

Status: proposed

Context
---------
The repository currently contains a runtime SQLite DB file (`rustine.db`). This risks leaking local data and polluting repository history. In addition, we need a deterministic migration strategy for schema changes.

Decision
---------
1. Store the runtime database in an OS-appropriate user data directory instead of the repo root. Use `directories` or `dirs-next` to compute a path such as:
   - Windows: `%APPDATA%/Rustine/rustine.db`
   - macOS: `~/Library/Application Support/Rustine/rustine.db`
   - Linux: `$XDG_DATA_HOME/rustine/rustine.db` (fall back to `~/.local/share`)
2. Add a small, embedded migration system. Keep SQL migration files under `bmad/migrations/` (e.g. `0001_create_tables.sql`) and apply them at startup. Use a lightweight approach (loop over files and execute) or a crate such as `rusqlite_migration`.
3. Provide an environment variable override `RUSTINE_DB_PATH` for testing and CI to allow `:memory:` or custom paths.

Consequences
------------
- Pros: avoids committing runtime data, clearer upgrade path for schema changes, reproducible local installs.
- Cons: small change to startup code and need to manage migrations.

Implementation Notes
--------------------
- Update `db::init_db()` to compute the DB path via `directories::ProjectDirs::from("com", "medyll", "rustine")` and create the directory if missing.
- Implement migration runner: list files in `bmad/migrations/` ordered by name and execute each SQL file atomically; record applied migrations in a `schema_migrations` table.
- Add `rustine.db` to `.gitignore` (separate small patch / dev story).

Acceptance Criteria
-------------------
- Default runtime DB is created under the OS app data directory.
- Migration files live in `bmad/migrations/` and are executed at startup where needed.
- Tests and CI override DB path via `RUSTINE_DB_PATH` where necessary.

Next Steps / Follow-ups
----------------------
- Dev story: implement path resolution and migration runner, add `rustine.db` to `.gitignore`, add seed/example migration files.

(end)
