---
inclusion: fileMatch
fileMatchPattern: "**/*.rs"
---

# Error Resolution Guide — RustConn

Common compiler errors and their architecturally correct solutions for this project.
Do not apply superficial fixes — find the root cause.

## Ownership & Borrowing

| Error | Superficial fix ❌ | Correct solution ✅ |
|-------|---------------------|---------------------|
| E0382 (use after move) | `.clone()` | `Rc<T>` / `Arc<T>` for shared data; pass `&T` if ownership is not needed |
| E0505 (borrow while moved) | `.clone()` before borrow | Restructure: borrow first, then move |
| E0502 (mutable + immutable borrow) | `RefCell` everywhere | Split into separate fields or use take-invoke-restore pattern |
| E0597 (lifetime too short) | `'static` | Pass owned data or use `Rc`; in GTK callbacks — `clone!` macro |

## RefCell / Rc Patterns (GTK4 specific)

| Problem | Solution |
|---------|----------|
| `BorrowMutError` at runtime | **take-invoke-restore**: `let val = state.borrow_mut().field.take(); ...; state.borrow_mut().field = val;` |
| Borrow across async boundary | Clone `Rc` before `spawn_local`, do not hold `Ref`/`RefMut` across `.await` |
| Circular Rc references | `Rc::new_cyclic` or `Weak<T>` for back-references |

## Async / Tokio

| Error | Solution |
|-------|----------|
| "Cannot start runtime within runtime" | Use `with_runtime()` helper — thread-local Runtime |
| `Send` bound not satisfied | GTK objects are not Send — use `spawn_local` or channel pattern |
| Timeout on vault operations | Always `tokio::time::timeout(Duration::from_secs(10), ...)` |

## Clippy Lints

| Lint | Solution |
|------|----------|
| `cognitive_complexity` | Extract inner logic into a separate fn; for GTK — builder pattern |
| `too_many_arguments` (>7) | Create a struct: `struct ConnectionParams { ... }` |
| `missing_errors_doc` | Add `/// # Errors\n/// Returns error if...` |
| `significant_drop_tightening` | Explicitly `drop(guard)` before the next operation |
| `option_if_let_else` (allowed) | Ignore — allowed in .clippy.toml |

## thiserror Patterns

| Situation | Pattern |
|-----------|---------|
| Wrapping std::io::Error | `#[error("operation failed: {0}")] Io(#[from] std::io::Error)` |
| Wrapping with context | `#[error("failed to {action}: {source}")] WithContext { action: String, source: Box<dyn std::error::Error + Send + Sync> }` |
| Enum → Display for UI | Implement `display_name()` → wrap in `i18n()` at call site |

## SecretString Patterns

| Situation | Pattern |
|-----------|---------|
| Get password from UI | `SecretString::new(entry.text().to_string().into())` |
| Pass to CLI | `cmd.stdin(Stdio::piped()); child.stdin.write_all(secret.expose_secret().as_bytes())` |
| Temporary String | `let tmp = Zeroizing::new(secret.expose_secret().to_string()); use tmp; // auto-zeroize on drop` |
| Comparison | `secret1.expose_secret() == secret2.expose_secret()` (in scope where both are accessible) |

## GTK4 / libadwaita

| Problem | Solution |
|---------|----------|
| Widget not showing | Check `.set_visible(true)` and that parent has `child`/`append` |
| Signal handler memory leak | `connect_*` with `clone!(@weak self as this =>` |
| Dialog not closing | `dialog.close()` or `dialog.set_visible(false)` + `dialog.destroy()` |
| Wayland: no window position | Do not use `set_position` — Wayland does not support it |
