Title: Remove tauri-plugin-sql and sqlx (security)

Branch: fix/rust-deps-upgrades-2026-02-05

Summary:
- Removed `tauri-plugin-sql` and `sqlx` from `src-tauri/Cargo.toml`.
- Project uses `diesel` / `rusqlite` for SQLite access; `sqlx` was unused and pulled `sqlx-mysql` transitively.
- This removes the transitive advisory RUSTSEC-2023-0071 (`rsa`).

Files changed:
- src-tauri/Cargo.toml

Notes for reviewers:
- Verify no runtime usage of `tauri-plugin-sql` APIs (search done; none found).
- Run `cargo test && cargo clippy -- -D warnings && cargo audit` after pulling branch.

Created by automation: Vibe sentinel patch generator.
