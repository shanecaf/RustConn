---
inclusion: fileMatch
fileMatchPattern: "rustconn/src/**/*.rs"
---

# GNOME HIG — RustConn Adaptation

Adaptation of [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/) for RustConn (GTK4 + libadwaita).
Supplements `dialogs-guide.md` and `window-guide.md`. Only lists points missing from other steering files.

## Writing Style — UI language

GNOME HIG: brief, human, no jargon. Ukrainian localization — see `po/uk.po` style guide
(`uk-translation-reviewer` agent). General rules:

- **Sentence case** for everything: buttons, headings, menus, toggles. Dialog title: "Properties of connection", NOT "Properties Of Connection".
- **Address the user directly** via imperative ("Save", "Connect"), not "Please save".
- **Do not use exclamation marks** "!" in normal UI — sounds alarming. Exceptions: critical errors.
- **Avoid abbreviations** like "info", "config" — write full words.
- **Button labels are action verbs**: "Connect", "Save", "Delete" — not "OK" when you can be more specific.
- **Errors** — explain what happened + what to do. Not "Error 0x80070005", but "Connection refused. Check that the host is reachable."

Everything still wrapped in `i18n()` / `i18n_f()`.

## UI Styling — CSS classes from libadwaita

Buttons carry semantics via CSS class:

```rust
let connect_button = gtk4::Button::with_label(&i18n("Connect"));
connect_button.add_css_class("suggested-action");   // primary action — blue

let delete_button = gtk4::Button::with_label(&i18n("Delete"));
delete_button.add_css_class("destructive-action");  // red
```

Other semantic classes (libadwaita 1.5+):
- `flat` — borderless button (icon-only in header bar),
- `pill` — rounded button (welcome screens),
- `circular` — circular button (close, add),
- `accent` — on banners and styles.

**Rule**: one `suggested-action` per dialog (primary action). `destructive-action` — only for irreversible operations (delete, revoke).

## Dialogs — use `adw::AlertDialog`

For confirm/alert (yes/no, OK) — `adw::AlertDialog`, NOT `gtk::MessageDialog` (deprecated):

```rust
let dialog = adw::AlertDialog::new(
    Some(&i18n("Delete connection?")),
    Some(&i18n_f("This will permanently remove '{}'.", &[&conn.name])),
);
dialog.add_response("cancel", &i18n("Cancel"));
dialog.add_response("delete", &i18n("Delete"));
dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
dialog.set_default_response(Some("cancel"));
dialog.set_close_response("cancel");
```

- `set_response_appearance` → `Suggested` or `Destructive`.
- Default response — the safest action (usually Cancel).
- Close response (Escape) — also Cancel.

For larger forms — `adw::Dialog` with custom content (Properties, Connection editor).

## Header bars

- `adw::HeaderBar` — standard; do not use `gtk::HeaderBar` directly in new widgets.
- Title widget → `adw::WindowTitle` with title + subtitle, or `adw::ViewSwitcher` for tabs.
- Primary action in headerbar — left side (e.g. New connection); secondary/menu — right side.
- Burger menu (☰) — `gtk::MenuButton` with `adw::PopoverMenu`, opens with F10.

## Toasts vs Banners vs Dialogs — when to use what

| Pattern | When |
|---------|------|
| `adw::Toast` (via `adw::ToastOverlay`) | Transient messages about results ("Connected", "Saved"). Non-blocking. |
| `adw::Banner` | Persistent state requiring attention: "You are offline", "Update available". Integrated into the window. |
| `adw::AlertDialog` | Action confirmation or modal decision. Blocking. |

Do not show a toast for critical errors — use a banner or alert dialog.

## Boxed lists — settings and lists

Any settings list → `adw::PreferencesGroup` with `adw::ActionRow` / `adw::EntryRow` /
`adw::SwitchRow` / `adw::ComboRow` / `adw::SpinRow`. Do not combine with raw `gtk::ListBox`.

```rust
let group = adw::PreferencesGroup::new();
group.set_title(&i18n("Connection details"));

let host_row = adw::EntryRow::new();
host_row.set_title(&i18n("Host"));
group.add(&host_row);
```

## Keyboard — mandatory shortcuts

Every GTK4 application must support:

| Shortcut | Action |
|----------|--------|
| `Ctrl+W` | Close current window/tab |
| `Ctrl+Q` | Quit application |
| `Ctrl+,` | Open Preferences (if available) |
| `F10` | Open primary menu |
| `Ctrl+?` or `F1` | Show shortcuts window |
| `Escape` | Close dialog / popover / cancel mode |
| `Ctrl+F` | Search (where relevant) |

Register via `gtk::Application::set_accels_for_action()`.

Shortcuts window → `gtk::ShortcutsWindow` from `.ui` file or `gtk::Builder`.

## Adaptive design — Wayland-first, mobile-friendly

- Minimum window size — support 360×294px (phone size). Verify via `adw::WindowResizable`.
- Sidebar → `adw::OverlaySplitView` (auto-collapse), not `gtk::Paned`.
- Toolbar → `adw::ToolbarView` instead of manual `gtk::Box`.

## Pointer & Touch

- Minimum tap target: 44×44px (via `set_size_request` for icon-only buttons).
- Long-press for context menu — add via `gtk::GestureLongPress` alongside right-click.
- Hover state — decoration only; do not rely on hover for important functionality (touch screens have no hover).

## Accessibility

- Every icon-only button → `set_tooltip_text(Some(&i18n("...")))` AND
  `update_property(&[gtk4::accessible::Property::Label(&i18n("..."))])`. Already documented in `dialogs-guide.md`.
- All form widgets → `set_accessible_role(Role::TextBox)` (usually set automatically, verify with Inspector).
- Test with high-contrast and large-text — `gsettings set org.gnome.desktop.a11y.interface high-contrast true`.
- Min contrast ratio 4.5:1 for text, 3:1 for UI elements (WCAG AA).
- Do not convey information by color alone (connection status — color + icon).

## Icons

- Symbolic icons (`*-symbolic`) for inline UI (toolbar, lists). Colorful — only for app icon and decorative.
- Size: 16px for inline, 24px for toolbar, 32px for grid items.
- Check availability in Adwaita icon theme: <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/named-icons.html>

## Spacing — quick reference

| Context | Spacing |
|---------|---------|
| Window margin / `adw::Clamp` | 12px |
| Between related elements (label + entry) | 6px |
| Between groups | 18–24px |
| Header bar internal padding | automatic |
| Boxed list rows | automatic via AdwListBox |

Width clamp: 600px for preferences, 800px for content (messages).

## Anti-patterns (do not do this)

- ❌ `gtk::MessageDialog` — deprecated, use `adw::AlertDialog`.
- ❌ `gtk::Notebook` for main UI — use `adw::TabView` + `adw::TabBar`.
- ❌ `gtk::Statusbar` — use `adw::Toast` or `adw::Banner`.
- ❌ `gtk::Dialog` without `set_modal(true)` — on Wayland looks like a separate window.
- ❌ Hardcoded RGB colors in code — use CSS classes (suggested-action, error, success).
- ❌ Custom window sizes via `set_default_size` without `adw::WindowResizable`.

## References

- HIG entry: <https://developer.gnome.org/hig/>
- Patterns: <https://developer.gnome.org/hig/patterns.html>
- Accessibility: <https://developer.gnome.org/hig/guidelines/accessibility.html>
- Writing style: <https://developer.gnome.org/hig/guidelines/writing-style.html>
- libadwaita docs: <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/>
- libadwaita named icons: <https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/named-icons.html>
