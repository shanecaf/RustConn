# RustConn 0.21.5 — macOS test pass

**Temporary file, scoped to this release.** Delete it once the pass is done.

Give this file to an agent on the Mac, or work through it by hand. Two things in
0.21.5 change what a macOS build contains, and neither can be checked from Linux.

## Why this pass exists

1. **The Homebrew formula no longer writes its feature list out by hand.** It had
   `adw-1-8` hardcoded and selected no GTK or VTE feature at all, so the Command
   monitoring mode could not appear on macOS whatever VTE Homebrew had installed.
   It now asks `pkg-config --atleast-version` for libadwaita, GTK and VTE and
   builds the list from the answers. Nothing here has ever run that code.
2. **The dark-theme workaround is now compiled in on every non-macOS build.** On
   macOS it stays excluded, deliberately: there `gtk-application-prefer-dark-theme`
   is driven by the GTK Quartz backend to mirror the system NSAppearance, so
   touching it would fight macOS' own light/dark switching. Test 3 confirms the
   exclusion still holds — the failure mode is a window that ignores the system
   appearance.

The formula's Ruby was syntax-checked in a container, but the release workflow
copies it into the Homebrew tap **without parsing it**, so a build here is the first
real execution.

## Setup

Build from this branch rather than the tap, so you are testing the formula as
committed:

```bash
brew install --build-from-source --verbose ./packaging/macos/rustconn.rb 2>&1 | tee /tmp/rc-brew.log
```

If `--build-from-source` on a local path fights the `PLACEHOLDER_SHA256` in the
formula, point it at a checkout instead:

```bash
brew install --HEAD --verbose ./packaging/macos/rustconn.rb 2>&1 | tee /tmp/rc-brew.log
```

Record:

```bash
sw_vers
brew list --versions gtk4 libadwaita vte3
pkg-config --modversion gtk4 libadwaita-1 vte-2.91-gtk4
```

---

## Test 1 — the formula selected the right features

This is the test the whole file exists for.

```bash
grep "RustConn feature set" /tmp/rc-brew.log
```

Expected: exactly one line listing the features, all package-qualified, e.g.

```
==> RustConn feature set: rustconn/tray-macos,rustconn/system-keyring,...,rustconn/adw-1-8,rustconn/gtk-4-22,rustconn/vte-0-78
```

Check it against the versions from the setup block:

| Installed | Expected in the list |
|---|---|
| libadwaita ≥ 1.8 / ≥ 1.7 / ≥ 1.6 | `adw-1-8` / `adw-1-7` / `adw-1-6` |
| gtk4 ≥ 4.22 / ≥ 4.20 / ≥ 4.18 | `gtk-4-22` / `gtk-4-20` / `gtk-4-18` |
| vte3 ≥ 0.78 | `vte-0-78` |

Three ways this can go wrong, all worth reporting precisely:

- **The line is missing entirely** — `ohai` did not run, so the loop above it did
  not either. Paste whatever error is near it.
- **A rung is missing although the library is new enough** — pkg-config did not
  find that `.pc` file during the build. Report the output of
  `pkg-config --list-all | grep -E "gtk4|libadwaita|vte"` from inside a
  `brew` build environment if you can get it; otherwise just say which rung.
  Note especially `vte-2.91-gtk4`: Homebrew's formula is called `vte3`, and if it
  ships only the GTK3 `.pc` file the VTE rung will be silently absent.
- **A rung is too high** — a feature named for a version newer than what is
  installed. That would fail at compile time rather than silently, so a successful
  build already rules it out.

## Test 2 — the app launches both ways, and the Command mode is present

1. `open $(brew --prefix)/opt/rustconn/RustConn.app` — the Dock tile must show the
   RustConn icon, not the generic Unix-executable one, and the menu-bar item must
   appear.
2. `rustconn` from a terminal — same app, tray item present.
3. Preferences ▸ Monitoring ▸ Default Mode must contain **Command finished**. If
   test 1 showed `vte-0-78` and this entry is missing, that is a contradiction
   worth reporting.
4. Exercise it: SSH to a host with shell integration sourced, set the tab's Monitor
   to **Command finished**, switch away, run `sleep 20; true`. Expect a
   notification and a needs-attention mark on the tab (a line under it, a dot on
   the Tab Overview thumbnail). Then `sleep 5; exit 1` — the message must name
   status 1.

## Test 3 — system appearance still drives the window

The macOS exclusion described above. Run with logging:

```bash
RUST_LOG=debug rustconn 2>&1 | tee /tmp/rc-mac.log
```

1. Preferences ▸ Appearance ▸ colour scheme → **System**.
2. Preferences ▸ Terminal ▸ Theme → **Follow System**.
3. Flip System Settings ▸ Appearance between Light and Dark while the app runs,
   with a terminal tab open.

Expected: window chrome **and** terminal colours follow, together, without a
reconnect.

Report:

```bash
grep "resolved color scheme at startup" /tmp/rc-mac.log
grep -c "Cleared deprecated gtk-application-prefer-dark-theme" /tmp/rc-mac.log
```

The first should report `dark=` matching the system at launch. The second **must be
`0`** — that block is excluded on macOS on purpose. Any non-zero count means the
`cfg` broke and macOS is now fighting its own appearance handling.

## Test 4 — the stylesheet fix

0.21.5 fixes four declarations that had never applied — the monitoring bar's
horizontal margins were spelled `margin-start` / `margin-end`, which GTK's CSS does
not have.

1. Open a connection with the activity monitor on; the monitoring bar should have a
   visible gap from the window edges. Narrow the window to trigger the compact
   layout and check that too.
2. `RUSTCONN_CSS_WARNINGS=1 RUST_LOG=debug rustconn 2>&1 | grep -i -E "theme parser|gtk\.css"`
   — expected: no output naming `style.css`.

## Test 5 — the macOS-specific paths still work

These have their own `-sys` crates and are the parts most likely to break silently:

- **PTY** (`rustconn-pty-sys`) — open a Local Shell tab, run `stty -a`, confirm
  Backspace erases and `Ctrl-C` interrupts a `sleep 60`.
- **Dock tile** (`rustconn-dock-sys`) — covered by test 2 step 1; also check the
  tile when launched via `rustconn` from a terminal, which is the no-bundle path
  that crate exists for.
- **Locale** (`rustconn-locale-sys`) — switch the language in Preferences, restart,
  confirm the UI is translated and that startup does not re-exec (no double window
  flash).
- **Tray** (`tray-macos`) — menu-bar item opens its menu, and its entries work.

## Test 6 — sweep

Report anything that differs from 0.21.4 on the same Mac:

- SSH, RDP, VNC connect
- split terminals, resize mid-session
- SFTP browser lists a remote directory
- keychain: save a password, restart, reconnect without retyping
- Preferences persist across restart

## What to send back

1. The setup block output.
2. The `RustConn feature set` line from test 1, verbatim.
3. Both greps from test 3, verbatim.
4. Pass/fail per test with a sentence each.
5. `/tmp/rc-brew.log` if the build failed, `/tmp/rc-mac.log` if runtime did.

Tests 1 and 3 are the ones that decide whether 0.21.5's packaging change was
correct. Please paste their output rather than summarising it.
