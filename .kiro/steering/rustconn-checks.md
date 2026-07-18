---
inclusion: manual
description: "On-demand full quality gate: runs cargo fmt, clippy, and tests. Fixes clippy issues if found."
---

Run the following checks sequentially in the workspace root:

1. `cargo fmt --check` — if formatting errors found, run `cargo fmt --all` to fix them and report what changed.

2. `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings. If warnings found, fix them and re-run clippy to confirm.

3. Before running tests, check `pgrep -f 'cargo test'`. If tests are already running, report "Tests already in progress, skipping" and do NOT start another instance. Otherwise run `cargo test --workspace` directly (NO pipes, NO tail/grep — run the command as-is so progress is visible). Allow up to 180s — argon2 property tests take ~120s in debug mode, this is normal.

Provide a short pass/fail report for each step. If clippy has warnings, fix them and re-run. If tests fail (failed > 0), report the failures to the developer.
