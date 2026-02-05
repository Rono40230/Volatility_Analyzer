PR: Remove tauri-plugin-sql and sqlx

Branch: fix/rust-deps-upgrades-2026-02-05

Summary:
- Remove unused `tauri-plugin-sql` and `sqlx` from `src-tauri/Cargo.toml` to eliminate a transitive advisory (RUSTSEC-2023-0071 via `rsa`).
- No runtime usages found after repo scan. Changes already committed and pushed to the branch.

Why manual PR:
- The environment lacks the `gh` CLI and no `GITHUB_TOKEN` is set, so automatic PR creation is not possible here.

How to open the PR (two options):

1) Web UI (recommended):
- Go to the repository: https://github.com/Rono40230/Analyses-historiques
- Click "Compare & pull request" for branch `fix/rust-deps-upgrades-2026-02-05`.
- Paste this file's content as PR description and attach `.vibe/PR_NOTES/remove-tauri-plugin-sql.md` if desired.

2) CLI using a token (example curl using GitHub API):
- Set `GITHUB_TOKEN` with a personal access token that has repo rights.

curl -X POST \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github+json" \
  https://api.github.com/repos/Rono40230/Analyses-historiques/pulls \
  -d '{"title":"Remove unused tauri-plugin-sql and sqlx (security)","head":"fix/rust-deps-upgrades-2026-02-05","base":"main","body":"See .vibe/PR_NOTES/remove-tauri-plugin-sql.md for rationale."}'

Notes:
- After PR creation, run CI (clippy, cargo-audit) on the branch and request review.
