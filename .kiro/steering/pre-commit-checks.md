---
inclusion: manual
description: "On-demand quality gate before committing. Run manually before git commit/push. Delegates to rust-quality-check sub-agent."
---

The developer wants to run pre-commit checks. Invoke the rust-quality-check sub-agent with prompt: "Run fmt and clippy checks. Do NOT run tests unless explicitly requested." Report the result and remind the developer to commit if all checks pass.
