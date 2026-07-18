---
inclusion: manual
description: "On-demand test runner. Trigger manually when you want to run cargo test. Checks for duplicate test processes before starting."
---

The developer wants to run tests. IMPORTANT RULES:

1. Before starting tests, check if another cargo process is running: `pgrep -f 'cargo test'`. If one is already running, SKIP and report: "Tests already in progress, skipping duplicate run."

2. Do NOT pipe through tail or grep — run the command directly so you can see progress.

3. Allow up to 180 seconds — argon2 property tests are slow in debug mode, this is normal.

4. Report only the final summary line (e.g. "test result: ok. 42 passed; 0 failed"). If tests fail, report the failing test names.

5. Run: `cargo test --workspace`
