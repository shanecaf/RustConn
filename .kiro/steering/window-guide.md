---
inclusion: fileMatch
fileMatchPattern: "rustconn/src/window/**/*.rs"
---

# Window / Sessions — Development Rules

You are editing a file in `rustconn/src/window/`.

## State Management

- `SharedAppState = Rc<RefCell<AppState>>` — pass as `&SharedAppState`
- NEVER hold a borrow across async boundaries or GTK callbacks
- Use `with_state()` / `with_state_mut()` helpers instead of direct `.borrow()`
- For callbacks with RefCell → take-invoke-restore pattern (as in `handle_ironrdp_error`)

## Sidebar

- Statuses: yellow = connecting, green = connected, red = failed, gray = disconnected
- Reconnect → reuse existing tab (don't create a new one)
- Context menu → GNOME HIG order: primary action at top, destructive at bottom

## Toasts

- `adw::ToastOverlay` with severity icons
- Use `i18n_f()` with `{}` placeholders for dynamic values

## Tabs

- Tab Overview → `AdwTabOverview`, terminals always inside `TabPage`
- Split view → layout lives inside TabPage, not in a global container
