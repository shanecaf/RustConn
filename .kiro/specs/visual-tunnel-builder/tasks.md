# Implementation Plan: Visual Tunnel Builder

## Overview

Реалізація візуального конструктора SSH-тунелів як wizard-діалогу на базі `adw::NavigationView`. Замінює поточний flat-діалог на 3-крокову навігацію з діаграмою шляху тунелю, preview SSH-команди та індикацією стану.

## Tasks

- [x] 1. Create `tunnel_preview` module in rustconn-core — `TunnelPreviewParams` struct and `build_tunnel_preview_command()` function with unit tests (basic, proxy_jump, dynamic, multiple forwards, no forwards, no username, default port, identity file). Export via `pub mod tunnel_preview` in lib.rs. Verify: no gtk4/adw imports, no unsafe, no secrets in output.
- [x] 2. Create `TunnelPathDiagram` widget in `rustconn/src/dialogs/tunnel_builder/path_diagram.rs` — horizontal `gtk4::Box` with styled Frame nodes (localhost, bastion, target) connected by arrow labels. Implement `new()`, `widget()`, `update()`, `set_status()`, `hide_status()`, `accessible_description()`. Add CSS classes to `rustconn/assets/style.css`: `.tunnel-diagram`, `.tunnel-node`, `.tunnel-node.success/warning/error`, `.tunnel-arrow`, `.tunnel-status-dot`, `@keyframes tunnel-pulse`. All labels use `i18n()`.
- [x] 3. Create Step 1 page — Connection & Name in `rustconn/src/dialogs/tunnel_builder/step_connection.rs`. `adw::NavigationPage` with: `adw::EntryRow` (tunnel name, 1–128 chars), `adw::ComboRow` (SSH connections), `gtk4::SearchEntry` (filter, debounce 150ms), `adw::ComboRow` (jump host override), "New SSH Connection" button, embedded `TunnelPathDiagram`. Auto-detect bastion from `jump_host_id`/`proxy_jump`. Empty state when no SSH connections. Validate before "Next". All strings `i18n()`.
- [x] 4. Create Step 2 page — Port Forwards & Options in `rustconn/src/dialogs/tunnel_builder/step_forwards.rs`. `adw::NavigationPage` with: `adw::PreferencesGroup` for forward rules, `adw::ExpanderRow` per rule (direction dropdown, SpinRow ports, EntryRow remote host, delete button). Show/hide fields for Dynamic. Dynamic title via `display_summary()`. "Add Forward" button (max 20). Port validation (1–65535 error, <1024 warning). Remote host required for L/R. `adw::SwitchRow` for auto-start/reconnect. Embedded `TunnelPathDiagram`. All strings `i18n()`.
- [x] 5. Create Step 3 page — Review & Confirm in `rustconn/src/dialogs/tunnel_builder/step_review.rs`. `adw::NavigationPage` with: `TunnelPathDiagram` (full, with status in edit mode), summary `adw::PreferencesGroup` (ActionRows), monospace `gtk4::TextView` with SSH command preview via `build_tunnel_preview_command()`, copy button (clipboard + Toast "Copied"), info message when no forwards. "Create"/"Save" button (suggested-action). All strings `i18n()`.
- [x] 6. Create `TunnelBuilderDialog` in `rustconn/src/dialogs/tunnel_builder/mod.rs` — `TunnelBuilderContext` struct, `WizardState`, `adw::Dialog` with `adw::NavigationView`. Wire step navigation (Step1→Step2→Step3→Save). Implement `new()`, `set_tunnel()` (edit mode), `connect_save()`, `present()`. Status polling every 2s in edit mode. Running tunnel warning on save. Missing connection handling. Register `pub mod tunnel_builder` in `dialogs/mod.rs`.
- [x] 7. Integrate with `TunnelManagerWindow` — replace `show_add_edit_dialog()` calls with `TunnelBuilderDialog` in `rustconn/src/dialogs/tunnel.rs`. Update "Add Tunnel" and "Edit" button handlers. Pass `TunnelBuilderContext`. Remove old `show_add_edit_dialog()` and `ForwardRowWidgets`. Verify: list refreshes after save, cancel is safe, edit preserves UUID.
- [x] 8. i18n update — run `bash po/update-pot.sh` to regenerate POT, then `msgmerge --update` for all 16 languages (be, cs, da, de, es, fr, it, kk, nl, pl, pt, sk, sv, uk, uz, zh-cn). Verify no untranslated strings in tunnel_builder.
- [x] 9. Documentation & Changelog — add "Visual Tunnel Builder" to `CHANGELOG.md` `[0.14.7]` → `### Added`. Update `docs/USER_GUIDE.md` with wizard usage documentation. Update `README.md` features table to mention visual tunnel builder.

## Task Dependency Graph

```json
{
  "waves": [
    { "wave": 1, "tasks": [1, 2] },
    { "wave": 2, "tasks": [3, 4] },
    { "wave": 3, "tasks": [5] },
    { "wave": 4, "tasks": [6] },
    { "wave": 5, "tasks": [7] },
    { "wave": 6, "tasks": [8] },
    { "wave": 7, "tasks": [9] }
  ]
}
```

- Task 1 and Task 2 can be done in parallel (no dependencies between them)
- Tasks 3, 4 depend on Task 2 (use TunnelPathDiagram) and can be done in parallel
- Task 5 depends on Task 1 (uses `build_tunnel_preview_command`) and Task 2
- Task 6 depends on Tasks 3, 4, 5 (wires all pages together)
- Task 7 depends on Task 6 (replaces old dialog with new builder)
- Task 8 depends on Task 7 (all strings must be finalized)
- Task 9 depends on Task 8 (documentation reflects final state)

## Notes

- Existing `StandaloneTunnel`, `PortForward`, `TunnelManager` models are NOT modified
- The old `show_add_edit_dialog()` is removed in Task 7 (breaking change within the module, not public API)
- CSS animations (`@keyframes tunnel-pulse`) require GTK 4.14+ which is already the minimum version
- Pattern follows existing `ConnectionWizard` (`adw::NavigationView` + step pages)
