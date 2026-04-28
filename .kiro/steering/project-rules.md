---
inclusion: always
---

# RustConn — Project Rules

Communication language: Ukrainian.

## Architecture (3 crates)

| Crate | Purpose | Restrictions |
|-------|---------|-------------|
| `rustconn-core` | Business logic, models, protocols | **FORBIDDEN**: gtk4, adw, vte4 |
| `rustconn-cli` | CLI interface | Only rustconn-core |
| `rustconn` | GTK4/libadwaita GUI | May import everything |

## Absolute Rules

- `unsafe_code = "forbid"` — no unsafe whatsoever
- Passwords/keys → `secrecy::SecretString`, never plain String
- Errors → `thiserror::Error`, never `unwrap()`/`expect()`
- Logging → `tracing`, never `println!`/`eprintln!`
- i18n → `i18n()` / `i18n_f()` with `{}` placeholders for all user-facing strings
- After new i18n strings → `bash po/update-pot.sh` + `msgmerge --update` for 16 languages
- Rust 2024 edition: let-chains instead of collapsible_if
- Never `set_var`/`remove_var` (unsafe in Rust 2024)

## Quick Commands

```
cargo fmt --all                    # Format
cargo clippy --all-targets         # Lint (0 warnings)
cargo test --workspace             # Tests (~120s, argon2 is slow)
cargo test -p rustconn-core --test property_tests  # Property tests only
bash po/update-pot.sh              # Regenerate POT after new i18n strings
```

## Quality Checks

Delegate to `rust-quality-check` sub-agent for fmt+clippy+tests instead of running in main context.
For quick single-file validation → `getDiagnostics`.

## 16 Translation Languages

be, cs, da, de, es, fr, it, kk, nl, pl, pt, sk, sv, uk, uz, zh-cn
