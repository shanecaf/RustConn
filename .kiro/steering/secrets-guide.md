---
inclusion: fileMatch
fileMatchPattern: "rustconn-core/src/secret/**/*.rs"
---

# Secrets — Security Rules

You are editing a file in `rustconn-core/src/secret/`.

## Mandatory

1. Passwords → `secrecy::SecretString`, never plain `String`
2. Intermediate password Strings → `.zeroize()` after use
3. External CLIs → password via stdin pipe (`Command::new().stdin(Stdio::piped())`), NEVER via `.arg(password)`
4. Error messages → MUST NOT include secret values
5. Logging → MUST NOT log password/secret/token variables via `tracing`/`println!`/`dbg!`

## Timeouts

- Vault operations (store/retrieve/delete) → 10s timeout
- Credential resolution → 30s timeout
- Bitwarden auto_unlock → 30s timeout
- has_secret_backend / refresh_cache → 5s timeout

## New Backends

Implement `SecretBackend` trait → register in `secret/mod.rs`
