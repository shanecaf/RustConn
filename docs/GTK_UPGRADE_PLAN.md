# GTK4/libadwaita/VTE Upgrade Plan + GNOME Platform 50

Target release: **0.10.0**
Date: 2026-03

## 1. Overview

Two-part upgrade: gtk-rs crate generation 0.10→0.11 AND Flatpak runtime 49→50.

GNOME 50 releases **March 18, 2026**. Ubuntu 26.04 LTS (April 2026) and
Fedora 44 (April 2026) will ship GNOME 50 as default desktop.

### 1.1 Crate upgrades

| Crate | Current | Target | C library min |
|-------|---------|--------|---------------|
| `gtk4` | 0.10.2 | **0.11.0** | GTK 4.16+ |
| `libadwaita` | 0.8 (v1_5) | **0.9.1** (v1_9) | libadwaita 1.6+ |
| `vte4` | 0.9 (v0_72) | **0.10.0** (v0_80) | VTE 0.74+ |
| `gdk4-wayland` | 0.10 | **0.11.0** | GDK 4.16+ |

### 1.2 Platform upgrade

| Component | GNOME 49 (current) | GNOME 50 (target) |
|-----------|-------------------|-------------------|
| GTK | 4.20 | **4.22** |
| libadwaita | 1.8 | **1.9** |
| VTE | 0.78 | **0.80** |
| X11 session | Supported (fallback) | **Dropped** (Wayland-only) |
| VRR | — | **Variable Refresh Rate** |
| Fractional scaling | Basic | **125/150/175/200%** |

Flatpak runtime `org.gnome.Platform 50` will be available on Flathub
after March 18 release (currently on `50beta` branch).

## 2. Prerequisites

### 2.1 MSRV bump: 1.88 → 1.92

`gtk4 0.11.0` and `gdk4-wayland 0.11.0` require `rust-version = "1.92"`.

Update in `Cargo.toml`:
```toml
rust-version = "1.92"
```

### 2.2 No blocking third-party dependencies

All protocol crates are GTK-independent:
- `vnc-rs` 0.5 — pure async, no gtk/glib deps
- `ironrdp` 0.14 — pure protocol, no gtk/glib deps
- `spice-client` 0.2 — `backend-gtk4` feature NOT used by RustConn
- `ksni` 0.3 — D-Bus only (zbus)
- `cpal` 0.17 — audio only
- `resvg` 0.47 — SVG rendering only

### 2.3 Transitive dependency chain

`libadwaita 0.9.1` → requires `gtk4 ^0.11`, `glib ^0.22`
`vte4 0.10.0` → requires `gtk4 ^0.11`, `gdk4 ^0.11`

All four crates MUST be updated atomically in one commit.

## 3. Cargo.toml Changes

### Workspace `Cargo.toml`
```toml
# Before
gtk4 = "0.10"
vte4 = "0.9"
libadwaita = { version = "0.8", features = ["v1_5"] }

# After
gtk4 = "0.11"
vte4 = "0.10"
libadwaita = { version = "0.9", features = ["v1_9"] }
```

### `rustconn/Cargo.toml`
```toml
# Before
gtk4 = { workspace = true, features = ["v4_14"] }
gdk4-wayland = { version = "0.10", optional = true }
vte4 = { workspace = true, features = ["v0_72"] }

# After
gtk4 = { workspace = true, features = ["v4_14"] }  # keep v4_14 as minimum
gdk4-wayland = { version = "0.11", optional = true }
vte4 = { workspace = true, features = ["v0_80"] }   # unlock VTE 0.80 API
```

## 4. Breaking Changes to Address

### 4.1 glib/gio 0.22 (from 0.20)

- `ObjectExt` trait methods may have changed signatures
- `glib::clone!` macro syntax may differ
- `Value`/`Variant` conversion traits updated
- `IsA<>` bounds may be stricter

### 4.2 gtk4 0.11 (from 0.10)

- Builder pattern changes — some builder methods renamed/removed
- Signal handler signatures may change
- `WidgetExt` trait updates
- Deprecated widgets removed (check for any `#[deprecated]` usage)
- `Expression` API changes

### 4.3 libadwaita 0.9 (from 0.8)

- New `v1_6`, `v1_7`, `v1_8` feature gates available
- Deprecated widgets from 1.5 may have API changes
- `AdwDialog` API may have additions

### 4.4 vte4 0.10 (from 0.9)

- New `v0_74`, `v0_76`, `v0_78` feature gates
- Terminal API additions for newer VTE features

## 5. Execution Plan

### Phase 1: Version bump (1 commit)
1. Update `rust-version` to `"1.92"` in workspace `Cargo.toml`
2. Update all four crate versions in workspace deps
3. Update `gdk4-wayland` in `rustconn/Cargo.toml`
4. Update `vte4` features to `v0_80`
5. Update `libadwaita` features to `v1_9`
6. Run `cargo update` to resolve dependency tree
7. Fix all compile errors from breaking API changes
8. Run `cargo clippy --all-targets` — zero warnings
9. Run `cargo test` — all tests pass
10. Run `cargo fmt --check`

### Phase 2: Adopt new features (separate commits)
See Section 6 below.

### Phase 3: Update packaging — Flatpak runtime 49→50
1. Update `runtime-version: '50'` in both Flatpak manifests:
   - `packaging/flatpak/io.github.totoshko88.RustConn.yml`
   - `packaging/flathub/io.github.totoshko88.RustConn.yml`
2. Update VTE module source to 0.80.x (or remove if bundled in runtime 50)
3. Regenerate `cargo-sources.json` for Flatpak builds
4. Test Flatpak build against `org.gnome.Platform//50`
5. Update OBS/Debian packaging if MSRV affects build deps

## 6. New Features to Adopt

### 6.1 AdwSpinner — replace GtkSpinner (libadwaita 1.6, `v1_6`)

**Impact: 3 files**

`GtkSpinner` has known issues: invisible by default, requires `:spinning = true`,
freezes when system animations are off, poor scaling above 32×32.
`AdwSpinner` fixes all of these — auto-sizes, keeps spinning with animations off,
better visuals.

| File | Current | Migration |
|------|---------|-----------|
| `rustconn/src/dialogs/settings/ssh_agent_tab.rs` | `Spinner::new()` | `adw::Spinner::new()` — remove `.set_spinning(true)` |
| `rustconn/src/sidebar/mod.rs` | `gtk4::Spinner::new()` | `adw::Spinner::new()` — remove `.set_spinning(true)` |
| `rustconn/src/session/vnc.rs` | `Spinner::new()` + `.set_spinning(false/true)` | `adw::Spinner::new()` + `.set_visible(false/true)` |

### 6.2 CSS variables — replace named colors (libadwaita 1.6, `v1_6`)

**Impact: `rustconn/assets/style.css`**

Libadwaita 1.6 provides CSS variables (`--accent-bg-color`, `--accent-color`, etc.)
alongside the old named colors (`@accent_bg_color`). The old syntax still works,
but CSS variables enable `color-mix()`, relative colors, and media queries for
light/dark/high-contrast in a single file.

Migration (non-breaking, can be gradual):
```css
/* Before */
background-color: @accent_color;
color: @accent_fg_color;

/* After */
background-color: var(--accent-bg-color);
color: var(--accent-fg-color);
```

Also available: `--dim-opacity`, `--disabled-opacity`, `--border-opacity`,
`--window-radius` — useful for consistent styling.

**New capability: CSS media queries for theme variants**
```css
@media (prefers-color-scheme: dark) {
    .my-widget { background: var(--dark-3); }
}
@media (prefers-color-scheme: light) {
    .my-widget { background: var(--light-3); }
}
```

### 6.3 AdwShortcutsDialog — replace custom shortcuts dialog (libadwaita 1.8, `v1_8`)

**Impact: `rustconn/src/dialogs/shortcuts.rs`**

Current implementation: custom `adw::Window` with manual ListBox, search, keycap CSS.
Libadwaita 1.8 provides `AdwShortcutsDialog` — a native replacement for the
deprecated `GtkShortcutsWindow` with simpler structure (sections + items, no views).

Also provides `AdwShortcutLabel` — styled keycap widget replacing custom `.keycap` CSS.

Migration: rewrite `ShortcutsDialog` to use `adw::ShortcutsDialog` with
`adw::ShortcutsSection` and `adw::ShortcutsShortcut` items. Remove custom
`.keycap` CSS class.

### 6.4 System accent color support (libadwaita 1.6, `v1_6`)

**Impact: automatic, but CSS review needed**

Libadwaita 1.6 adds system accent color support. Apps automatically follow the
user's chosen accent color. The CSS in `style.css` already uses `@accent_color`
/ `@accent_bg_color` which will automatically adapt.

**Review needed:**
- Ensure no hardcoded blue (`#3584e4`) is used where accent color is intended
- Split panel colors (0-5) are intentionally fixed — these are fine
- Filter button `.suggested-action` uses `@accent_color` — will adapt correctly

### 6.5 AdwButtonRow (libadwaita 1.6, `v1_6`)

**Impact: potential improvement for action lists**

`AdwButtonRow` is a list row styled as a button — useful for destructive/suggested
actions in preference-style lists. Could improve:
- "Add shared folder" buttons in connection dialogs
- "Add expect rule" buttons in automation tab
- "Remove all" destructive actions

Also: `AdwPreferencesGroup` gains `:separate-rows` property and
`.boxed-list-separate` CSS class for visually separated rows.

### 6.6 AdwToggleGroup (libadwaita 1.7, `v1_7`) — SKIPPED

**Status: NOT APPLICABLE** — `AdwToggleGroup` uses `GTK_ACCESSIBLE_ROLE_RADIO_GROUP`
(single-select / exclusive toggles). The protocol filter bar requires multi-select
(multiple protocols active simultaneously). Using `AdwToggleGroup` would break the
existing multi-filter UX. No suitable workaround exists without fighting the widget's
semantics.

### 6.7 AdwWrapBox (libadwaita 1.7, `v1_7`) — ✅ DONE

**Impact: `rustconn/src/sidebar/mod.rs`**

`AdwWrapBox` replaces the `GtkBox` + `.linked` container for protocol filter buttons.
Buttons now wrap to the next line on narrow sidebars instead of being hidden.
Cfg-gated behind `adw-1-7` feature with `GtkBox` fallback for libadwaita < 1.7.

The manual responsive hide/show logic (`width < 280` → hide Telnet/Serial/ZeroTrust/K8s)
is removed when `adw-1-7` is enabled — `AdwWrapBox` handles wrapping automatically.

### 6.8 AdwSidebar (libadwaita 1.9, `v1_9`)

**Impact: potential future improvement for connection sidebar**

Libadwaita 1.9 introduces `AdwSidebar` — a native adaptive sidebar widget,
and `AdwViewSwitcherSidebar` as a replacement for `GtkStackSidebar`.
Could be evaluated for the connection tree sidebar in a future release.

### 6.9 AdwBottomSheet (libadwaita 1.6, `v1_6`)

**Impact: potential future use**

Standalone bottom sheet widget — could be useful for mobile-friendly layouts
if RustConn ever targets mobile/convergent form factors. Low priority for now.

### 6.10 VTE 0.80 features (`v0_80`)

**Impact: `rustconn/src/terminal/`**

VTE 0.74-0.80 includes:
- GPU-accelerated rendering (GTK4 version delegates drawing to GPU)
- 60 FPS frame clock updates (vs ~20-30 FPS before)
- Faster bidirectional text processing
- Reduced memory allocations
- Performance improvements are automatic — no code changes needed

New API to evaluate:
- Check for new terminal configuration options in VTE 0.76/0.78/0.80
- Potential new signal handlers or properties

### 6.11 Destructive button style update (libadwaita 1.6)

**Impact: automatic**

`.destructive-action` buttons now have a distinct style (less prominent than
`.suggested-action`) to work correctly with non-blue accent colors.
No code changes needed — automatic with the upgrade.

## 7. CSS Migration Decision

**Status: NOT MIGRATING** — `@named_color` syntax works on all libadwaita versions
(1.5 through 1.9). CSS variables (`var(--accent-color)`) require libadwaita ≥ 1.6
and would break Ubuntu 24.04 LTS native packages. The visual result is identical.
The only benefit of `var()` is `color-mix()` and CSS media queries for theme
variants, which are not used in RustConn's stylesheet.

If CSS variables are needed in the future, use progressive enhancement (double
declarations) or a cfg-gated second CSS file.

| Pattern | Count | Status |
|---------|-------|--------|
| `@accent_color` | ~20 | Keeping — works on all versions |
| `@error_color` | ~10 | Keeping — works on all versions |
| `@success_color` | ~5 | Keeping — works on all versions |
| `@warning_color` | ~4 | Keeping — works on all versions |
| `@borders` | ~4 | Keeping — works on all versions |
| `.keycap` custom CSS | 1 block | Replaced by `AdwShortcutLabel` when `adw-1-8` enabled |

## 8. Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| gtk-rs 0.11 breaking API changes | Medium | Compile-fix iteratively; most changes are mechanical |
| MSRV 1.92 not available in CI | Low | Rust 1.92 is stable; update CI toolchain |
| Flatpak build breaks | Low | Regenerate cargo-sources.json; test against runtime 50 |
| OBS distro builds need newer GTK | Medium | Ubuntu 24.04 has GTK 4.14 (sufficient for v4_14 feature) |
| New VTE API incompatibility | Low | VTE 0.80 is backward-compatible; new features are additive |
| spice-client future gtk4 dep | Low | `backend-gtk4` not used; monitor upstream |
| GNOME 50 runtime not yet on stable Flathub | Medium | Wait for `org.gnome.Platform//50` to land on stable branch after March 18 release; use `50beta` for testing only |
| X11 session dropped in GNOME 50 | Low | RustConn is Wayland-first; `--socket=fallback-x11` in Flatpak finish-args is harmless but unused on GNOME 50 |
| VTE 0.80 source module in Flatpak | Medium | If runtime 50 bundles VTE 0.80, remove the custom VTE module from manifests; otherwise update URL to `vte-0.80.x.tar.xz` |
| Flathub x-checker-data VTE version cap | Low | Update `versions: <: '0.79.0'` → `<: '0.81.0'` in Flathub manifest |

## 9. Distro Compatibility & Feature Flag Strategy

### Tiered delivery model

**Tier 1 — Full features (GNOME 50+, `adw-1-8`):**
All new libadwaita widgets enabled (AdwSpinner, AdwShortcutsDialog).

| Distro | libadwaita | VTE | Feature flags | Delivery |
|--------|-----------|-----|---------------|----------|
| Flatpak (GNOME 50) | 1.9 | 0.80 | `adw-1-8` | Flathub |
| Ubuntu 26.04 LTS (GNOME 50) | 1.9 | 0.80 | `adw-1-8` | GitHub .deb |
| openSUSE Tumbleweed (GNOME 50) | 1.9 | 0.82 | `adw-1-8` | OBS |
| openSUSE Slowroll | 1.8→1.9 | 0.78+ | `adw-1-8` | OBS |
| Fedora 44 (GNOME 50) | 1.9 | 0.80 | `adw-1-8` | OBS |
| Fedora 43 (GNOME 49) | 1.8 | 0.78 | `adw-1-8` | OBS |

**Tier 1b — Partial features (GNOME 48, `adw-1-7`):**
AdwSpinner + AdwWrapBox enabled, legacy shortcuts dialog.

| Distro | libadwaita | VTE | Feature flags | Delivery |
|--------|-----------|-----|---------------|----------|
| openSUSE Leap 16.0 (GNOME 48) | 1.7 | 0.78 | `adw-1-7` | OBS |
| Fedora 42 (GNOME 48) | 1.7 | 0.78 | `adw-1-7` | OBS |

**Tier 2 — Baseline (libadwaita 1.5, no extra features):**
GtkSpinner fallback, legacy shortcuts dialog. Delivered via Flatpak.

| Distro | libadwaita | VTE | Feature flags | Delivery |
|--------|-----------|-----|---------------|----------|
| Ubuntu 24.04 LTS | 1.5 | 0.76 | (none) | Flatpak (GNOME 50 runtime) |

### Build configuration per packaging system

| System | Build command | Notes |
|--------|-------------|-------|
| Flatpak (all manifests) | `cargo build --release -p rustconn --features adw-1-8` | GNOME 50 runtime has libadwaita 1.9 |
| OBS Tumbleweed/Slowroll | `cargo build --release -p rustconn --features adw-1-8` | libadwaita 1.8+ |
| OBS Leap 16.0 | `cargo build --release -p rustconn --features adw-1-7` | libadwaita 1.7 |
| OBS Fedora 43+ | `cargo build --release -p rustconn --features adw-1-8` | libadwaita 1.8+ |
| OBS Fedora 42 | `cargo build --release -p rustconn --features adw-1-7` | libadwaita 1.7 |
| GitHub .deb (Ubuntu 26.04) | `cargo build --release -p rustconn --features adw-1-8` | libadwaita 1.9 |

### Compatibility notes

- The `v4_14` feature flag in `gtk4` ensures runtime compatibility with GTK 4.14+.
- VTE feature `v0_76` chosen for broad compatibility (Ubuntu 24.04 has VTE 0.76).
  VTE 0.80 performance improvements (GPU rendering, 60 FPS) are automatic from
  the C library — no Rust feature gate needed.
- CSS uses `@named_color` syntax which works on all libadwaita versions (1.5–1.9).
- Ubuntu 24.04 LTS users get full functionality via Flatpak with GNOME 50 runtime.

## 10. Recommended Commit Sequence

1. ~~`chore: bump MSRV to 1.92`~~ ✅
2. ~~`chore: upgrade gtk4 0.11, libadwaita 0.9, vte4 0.10, gdk4-wayland 0.11`~~ ✅
   - Fix all compile errors
   - Zero clippy warnings
   - All tests pass
3. ~~`refactor: replace GtkSpinner with AdwSpinner`~~ ✅ (cfg-gated `adw-1-6`)
4. ~~`refactor: CSS migration`~~ — **SKIPPED**: `@named_color` works on all versions; no visual benefit from `var()` migration; would break Ubuntu 24.04 LTS
5. ~~`feat: use AdwShortcutsDialog for keyboard shortcuts`~~ ✅ (cfg-gated `adw-1-8`)
6. ~~`chore: bump Flatpak runtime to GNOME 50`~~ ✅
   - `runtime-version: '50'` in all three manifests
   - VTE module updated to 0.80.0
   - Flathub `x-checker-data` VTE cap updated to `< 0.81.0`
   - Flatpak builds use `--features adw-1-8`
7. ~~`chore: update OBS/Debian packaging with conditional feature flags`~~ ✅
   - OBS spec: conditional `adw-1-8` / `adw-1-6` per distro
   - Debian rules: `--features adw-1-8` for Ubuntu 26.04+
8. `chore: regenerate Flatpak cargo-sources.json` — before release
9. `chore: update packaging metadata for 0.10.0` — version bump, changelog
