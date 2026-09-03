# RustConn 0.21.5 — KDE / XFCE test pass

**Temporary file, scoped to this release.** It exists because 0.21.5 changes
behaviour that only a KDE or XFCE session exercises, and the machine that
developed it runs GNOME 50. Delete it once the pass is done.

Give this file to an agent on the KDE machine, or work through it by hand. Either
way, report back with the exact log lines asked for — "it looked fine" cannot
distinguish the two outcomes in test 1.

## Why this pass exists

RustConn clears the legacy GTK property `gtk-application-prefer-dark-theme` once,
before `adw::init()`. KDE and XFCE set that property globally — through xsettings
or `~/.config/gtk-4.0/settings.ini` — and older libadwaita warned when it found it
already true. GNOME never sets it; it expresses the preference through
`org.gnome.desktop.interface color-scheme`.

Until 0.21.5 that clear was compiled out on any build targeting GTK 4.20 or newer,
on the reasoning that such a build links libadwaita 1.8+, which no longer warns.
0.21.5 rejects that reasoning — it holds for the warning, not for the behaviour —
and keeps the clear on every build, guarded at runtime by whether the property is
actually set. On GNOME the guard is false and the block does nothing, which is
confirmed. **What no one has observed is the guard being true.** That is test 1.

## Setup

Use the **Flatpak** build as the primary target. It is the channel where this
changed: 0.21.5 builds it with `gtk-4-22`, which before this release would have
removed the workaround entirely.

```bash
flatpak install --user io.github.totoshko88.RustConn   # or the local build
```

Record, for the report:

```bash
flatpak info io.github.totoshko88.RustConn | head -20
plasmashell --version 2>/dev/null || xfce4-session --version 2>/dev/null
pkg-config --modversion gtk4 libadwaita-1 vte-2.91-gtk4 2>/dev/null
cat ~/.config/gtk-4.0/settings.ini 2>/dev/null
```

That last file matters: if it does **not** contain
`gtk-application-prefer-dark-theme=1`, test 1 cannot fire and you need to switch
the desktop to a dark Breeze/XFCE theme first and re-check.

---

## Test 1 — the workaround actually runs, and the window matches the desktop

The one test this whole file is for.

1. Set the desktop to a **dark** theme (Plasma: Breeze Dark; XFCE: a dark GTK
   theme). Confirm `~/.config/gtk-4.0/settings.ini` now has
   `gtk-application-prefer-dark-theme=1`.
2. Launch with logging:

   ```bash
   flatpak run --env=RUST_LOG=debug io.github.totoshko88.RustConn 2>&1 | tee /tmp/rc-kde-dark.log
   ```

3. In Preferences ▸ Appearance, set the colour scheme to **System**. Restart the
   app and capture the log again.

**Report these three greps verbatim:**

```bash
grep "Cleared deprecated gtk-application-prefer-dark-theme" /tmp/rc-kde-dark.log
grep "resolved color scheme at startup" /tmp/rc-kde-dark.log
grep -i -E "prefer-dark|Adw.*deprecated|legacy" /tmp/rc-kde-dark.log
```

Expected:

- The first grep prints **exactly one** line ending
  `(desktop had set it; GNOME does not)`. Zero lines means the guard did not fire
  and the workaround is dead code on KDE too — say so, that is a real finding.
  More than one line means something re-set the property and a handler is fighting
  libadwaita again, which is the 0.21.4 bug returning.
- The second prints `dark=true`.
- The third should show **no** libadwaita warning about the legacy property. If one
  appears, note its exact wording — the workaround exists to avoid it, so a warning
  means the clear happened too late or not at all.

**And the visible check that matters more than any log:** the window chrome,
sidebar and terminal must all be dark. A light window on a dark Plasma desktop is
the 0.21.4 bug and blocks the release.

## Test 2 — light desktop, and switching mid-session

1. Switch Plasma/XFCE to a **light** theme while RustConn is running with a
   terminal tab open.
2. The window and the terminal colours should both follow, without reconnecting.
3. Switch back to dark. Same.

Report whether the terminal background followed the window, or lagged behind it.
With Preferences ▸ Terminal ▸ Theme set to **Follow System** they must move
together; a theme chosen by name must not move at all.

Then repeat once from a cold start on the light theme and report:

```bash
grep -c "Cleared deprecated gtk-application-prefer-dark-theme" /tmp/rc-kde-light.log
```

On a light desktop KDE usually does not set the property, so `0` is the expected
and correct answer here.

## Test 3 — the stylesheet fix is visible

0.21.5 fixes four CSS declarations that had never applied: the monitoring bar's
horizontal margins were spelled `margin-start` / `margin-end`, which GTK's CSS does
not have.

1. Open a connection with the activity monitor enabled so the monitoring bar shows.
2. The bar should now have a visible gap from the window edges — 6 px normally,
   4 px in the narrow (compact) layout. Resize the window narrow enough to trigger
   compact and check both.
3. Confirm no parser complaints about our own stylesheet:

   ```bash
   flatpak run --env=RUSTCONN_CSS_WARNINGS=1 --env=RUST_LOG=debug \
       io.github.totoshko88.RustConn 2>&1 | grep -i -E "theme parser|gtk\.css"
   ```

   Expected: no output. Any line naming `style.css` is a real parse error — report
   it with the line number it names.

## Test 4 — Command monitoring mode is present and fires

New in this release, and it needs VTE 0.78+ in the build. The Flatpak has it.

1. Preferences ▸ Monitoring ▸ Default Mode — the list must contain **Command
   finished** alongside Off, Activity and Silence. If it is missing, the build did
   not get the `vte-0-78` feature; report that and stop this test.
2. On the remote host, source shell integration (the OSC 133 hooks — bash-preexec,
   or the vte script your distro ships). Open an SSH session to it.
3. Set that tab's Monitor to **Command finished**, switch to another tab, and run
   something slow remotely, e.g. `sleep 20; true`.
4. On completion you should get a notification saying the command finished, and the
   tab should carry a **needs-attention** mark: libadwaita draws a line under the
   tab, and a dot appears on the Tab Overview thumbnail.
5. Run `sleep 5; exit 1` the same way — the notification must say it failed with
   status 1, not merely that it finished.
6. Selecting the tab must clear the mark.

Report which of the six steps behaved and which did not. If step 2's shell
integration is not available on any host you can reach, say so rather than guessing
— the mode legitimately does nothing without it, and that is not a bug.

## Test 5 — nothing else regressed on this desktop

Quick sweep, report anything that differs from 0.21.4 on the same machine:

- tray icon appears and its menu opens (KDE uses the StatusNotifier path)
- sidebar right-click menu opens and stays open
- split terminals, and a window resize mid-session
- `mc` inside an SSH session, mouse included
- SFTP browser opens and lists a remote directory
- Preferences saves and survives a restart

## What to send back

1. The setup block output.
2. The three greps from test 1, verbatim.
3. Pass/fail per test with a sentence each.
4. `/tmp/rc-kde-dark.log` and `/tmp/rc-kde-light.log` if anything failed.

The single answer that decides whether the 0.21.5 change was right is test 1's
first grep. Please do not summarise it — paste it.
