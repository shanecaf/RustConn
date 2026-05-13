# macOS Port — Changelog

## Changes for macOS Support

### Added

- **Native PTY spawn for macOS** (`rustconn/src/macos_pty.rs`) — VTE's built-in `spawn_async` does not work on macOS (Homebrew build); the PTY is created but never connected to child process output. New module creates PTY via `nix::pty::openpty()`, spawns the child with slave fd as stdin/stdout/stderr, and hands the master fd to VTE via `Pty::foreign_sync()`. Conditional compilation `#[cfg(target_os = "macos")]` ensures zero impact on Linux.
- **macOS PATH extension in `get_extended_path()`** — GUI apps launched via `.app` bundle have minimal PATH (`/usr/bin:/bin`). Added `/opt/homebrew/bin`, `/opt/homebrew/sbin`, `/usr/local/bin`, and `/Applications/KeePassXC.app/Contents/MacOS` to the extended PATH on macOS. This fixes detection of all CLI tools (keepassxc-cli, bw, op, pass, gcloud, kubectl, etc.) without using `set_var`.
- **KeePassXC detection fallback paths** — Added `/opt/homebrew/bin/keepassxc-cli` and `/Applications/KeePassXC.app/Contents/MacOS/keepassxc-cli` to both `status.rs` and `detection.rs` so KeePassXC is found on macOS even when not in PATH.
- **macOS .app bundle** — `RustConn.app` with proper `Info.plist`, `.icns` icon, wrapper script setting environment variables, and self-contained binary.
- **Homebrew formula** (`packaging/macos/rustconn.rb`) — Complete formula for Homebrew Tap distribution with all dependencies, locale compilation, icon generation, and .app bundle creation.
- **DMG build script** (`packaging/macos/build-dmg.sh`) — Automated script to build release `.dmg` with self-contained `.app` bundle including Adwaita icons, locales, and GSettings schemas.

### Fixed

- **Cross-platform `statvfs` types** (`rustconn-core/src/rdp_client/rdpdr.rs`) — `fragment_size()`, `blocks()`, `blocks_available()` return different integer types on macOS vs Linux. Added `u64::from()` with `#[allow(clippy::useless_conversion)]` for cross-platform compatibility.
- **Local Shell on macOS** — launches with `--login` flag so `.zprofile`/`.zshrc` are sourced (macOS-only via `#[cfg]`).

### Known Limitations

- **VTE `spawn_async` broken on macOS** — Homebrew VTE build does not connect PTY to child process. Workaround: native PTY via `openpty()` + `Pty::foreign_sync()`.
- **Tray icon not available** — `ksni` requires D-Bus StatusNotifierItem protocol which doesn't exist on macOS. Build without `tray` feature.
- **Wayland not available** — Build without `wayland-native` feature.
- **CSS parser warnings** — libadwaita 1.9 CSS uses features not yet supported by GTK4 4.22 CSS parser. Cosmetic only, no functional impact.
- **libsecret not available** — GNOME Keyring doesn't exist on macOS. Use KeePassXC, Bitwarden, 1Password, or Pass backends instead. Future: macOS Keychain integration.
- **fzf-completion** — Works when launched via `.app` bundle (`open RustConn.app`). May show "job table full" error when launched directly from terminal without proper session setup.

### Build Configuration for macOS

```bash
cargo build -p rustconn --no-default-features \
  --features "vnc-embedded,rdp-embedded,rdp-audio,spice-embedded"
```

Disabled features: `tray`, `wayland-native`

### Dependencies (Homebrew)

```bash
brew install gtk4 libadwaita vte3 adwaita-icon-theme openssl@3 dbus gettext
```
