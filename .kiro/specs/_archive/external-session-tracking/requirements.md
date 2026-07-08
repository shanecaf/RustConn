# Requirements Document

## Introduction

This specification describes the external session tracking feature for RustConn 0.18.3.

The trigger is issue #209: when a VNC connection uses Window Mode = External Window, RustConn correctly launches the external viewer (TigerVNC) but also opens a "dead" tab in the main window. Expected behavior: no tab is created; the session is surfaced through a lightweight status indicator. The same problem affects RDP and SPICE.

The feature is split into two phases, both shipping in release 0.18.3:

- **Phase 1 (mandatory, closes #209)** — graphical sessions (VNC, RDP, SPICE) whose display is fully delegated to a separate external viewer process (TigerVNC, xfreerdp, remote-viewer/virt-viewer) no longer create a notebook tab. Instead, the session is registered in a lightweight process registry and surfaced in the sidebar with a dedicated icon emblem. A single shared poll timer watches all child processes and updates state when they exit.
- **Phase 2 (UX enhancement)** — extend the indication of existing sessions in the sidebar with orthogonal signals (shape + icon, never color alone), and add a "smart" double-click that focuses an existing session instead of creating a duplicate.

All pure decision logic (the predicate "does this connection use an external viewer") belongs to the `rustconn-core` crate (no gtk4/adw/vte4). GUI and process-handling code stays in the `rustconn` crate.

## Glossary

- **External Viewer**: a separate operating-system process (TigerVNC, xfreerdp, remote-viewer/virt-viewer, remmina, krdc) that renders the remote desktop in its own window, outside the RustConn main window.
- **External Viewer Session**: an active connection whose display is fully delegated to an external viewer and that has no notebook tab in the RustConn main window.
- **External Session Registry**: a lightweight component that tracks active external viewer sessions and their child-process handles (proposed name per M-CONCISE-NAMES: `ExternalSessionRegistry` / `ExternalSessionTracker`).
- **Detaching Viewer**: an external viewer that hands control to a daemon or separate process and exits its initial child process while the window stays open (for example `remmina -c`, `krdc`, sometimes `remote-viewer`). For such viewers `try_wait()` reports exit while the window is still running.
- **Window Mode**: the `WindowMode` enum in `rustconn-core` with values `Embedded` (default), `External`, `Fullscreen`. `Connection::supports_window_mode()` returns true for RDP, VNC, and SPICE.
- **Client Mode**: a protocol setting (for example `VncClientMode`) that selects an embedded (`Embedded`) or external (`External`) viewer.
- **External Viewer Emblem**: an icon badge on the connection entry in the sidebar that marks an active external viewer session (icon-based, for example `window-new-symbolic`, not color alone).
- **Split Color**: a color index mapped to a session in the existing `split_colors` map (session_id → color index), used for split tab indicator icons.
- **Session Focus**: an action that selects an existing session's tab (and, for a split, selects the owner tab and focuses the correct pane via `focus_pane`) instead of creating a new session.
- **Poll Timer**: a periodic timer (a generalization of the existing 2-second `glib::timeout_add_local` RDP process monitor in `rustconn/src/window/rdp_vnc.rs`) that watches the child processes of tracked sessions.
- **Session Count**: the existing sidebar mechanism (`increment_session_count` / `decrement_session_count`) that drives a connection's "connected" status.
- **RustConn**: the connection-manager application (the system these requirements apply to).

## Requirements

### Requirement 1: Suppress the tab for external viewer sessions (Phase 1, closes #209)

**User Story:** As a user opening VNC/RDP/SPICE in an external window, I want RustConn not to create a dead tab in the main window, so that the interface is not cluttered with unusable tabs.

#### Acceptance Criteria

1. WHERE a connection uses the VNC, RDP, or SPICE protocol, WHEN `window_mode` equals `External` OR the protocol's `client_mode` equals `External`, THE RustConn SHALL launch the external viewer and not create any notebook tab in the main window for that session.
2. THE RustConn SHALL decide tab suppression through a single shared predicate in the `rustconn-core` crate that takes the protocol, `window_mode`, and `client_mode` and returns the same boolean decision for the same inputs.
3. WHEN tab suppression logic is applied during a VNC launch, THE RustConn SHALL use the same shared predicate in both VNC launch paths (`rustconn/src/window/protocols.rs` and `rustconn/src/window/rdp_vnc.rs`) with an identical result for the same inputs.
4. IF an embedded VNC session falls back to an external viewer at runtime in `session/vnc.rs::connect_external_with_config` after a tab has already been created, THEN THE RustConn SHALL keep the existing tab and show in it a placeholder indicating the session runs in an external window.
5. WHERE the protocol is embedded RDP (`start_embedded_rdp_session`), THE RustConn SHALL keep the existing behavior of creating an embedded tab unchanged.
6. IF launching the external viewer fails, THEN THE RustConn SHALL not create a notebook tab for that session and SHALL show an error message indicating the external viewer could not be launched.

### Requirement 2: Sidebar indication of an active external viewer session (Phase 1)

**User Story:** As a user, I want to see in the sidebar that a connection is active in an external viewer, so that I understand the connection state without a tab.

#### Acceptance Criteria

1. WHEN an external viewer session is registered, THE RustConn SHALL increment the connection's session count via `increment_session_count` and display the existing green "connected" status icon within 200 ms.
2. WHEN an external viewer session is registered, THE RustConn SHALL show a dedicated external viewer icon emblem (for example `window-new-symbolic`) on the connection entry, next to the "connected" icon and not replacing it.
3. THE RustConn SHALL convey the external viewer session state with at least two color-independent visual cues (a distinct icon and shape) that remain distinguishable in monochrome mode.
4. THE RustConn SHALL give each status icon and emblem a non-empty accessible text label via `i18n()` that is reachable by screen readers.
5. WHILE the connection's external viewer session count is greater than zero, THE RustConn SHALL show the external viewer emblem on that connection entry.
6. WHEN the connection's session count reaches zero, THE RustConn SHALL remove the external viewer emblem from that connection entry within 200 ms.
7. IF registering or ending an external viewer session fails, THEN THE RustConn SHALL keep the prior session-count and emblem state and log the error.

### Requirement 3: Connection history recording for external viewer sessions (Phase 1)

**User Story:** As a user, I want connection history to record external sessions the same way as embedded ones, so that the connection log stays complete.

#### Acceptance Criteria

1. WHEN an external viewer session is successfully established, THE RustConn SHALL record the connection start via `record_connection_start` within 1 second, in the same record format used for embedded sessions.
2. WHEN an external viewer session ends, THE RustConn SHALL record the connection end via `record_connection_end` within 1 second.
3. THE RustConn SHALL record exactly one start entry and exactly one end entry per external viewer session.
4. IF a call to `record_connection_start` or `record_connection_end` fails, THEN THE RustConn SHALL log the error and not interrupt the session.
5. IF a session ends without an existing start record, THEN THE RustConn SHALL skip `record_connection_end` and log a warning.

### Requirement 4: Shared poll timer for external viewer processes (Phase 1)

**User Story:** As a user, I want RustConn to detect when an external viewer closes, so that the connection state updates automatically.

#### Acceptance Criteria

1. THE RustConn SHALL use a single shared poll timer with a 2-second (2000 ms) interval, generalized from the existing RDP process monitor in `rustconn/src/window/rdp_vnc.rs`, to watch all tracked child processes.
2. WHILE the poll timer is active, THE RustConn SHALL check the exit state of each tracked child process on every cycle.
3. WHEN a tracked child process exits, THE RustConn SHALL decrement the session count via `decrement_session_count` no later than one poll cycle (2 s) and exactly once for that process.
4. WHEN a tracked child process exits, THE RustConn SHALL record the connection end via `record_connection_end` exactly once for that process.
5. WHEN a tracked child process exits, THE RustConn SHALL remove the external viewer emblem from the connection entry associated with that specific process.
6. WHEN a new child process is registered while the poll timer is stopped, THE RustConn SHALL restart the poll timer.
7. WHILE there are no tracked child processes, THE RustConn SHALL stop the poll timer.

### Requirement 5: Sidebar context menu for external viewer sessions (Phase 1)

**User Story:** As a user, I want to manage external viewer sessions from the sidebar context menu, so that I can properly end both RustConn-owned and detaching viewers.

#### Acceptance Criteria

1. WHERE a connection has an active external viewer session (registered in the registry and not marked ended), THE RustConn SHALL show "Disconnect" and "Stop tracking" actions in the sidebar context menu; when no such session exists, these items are not shown.
2. WHEN the user selects "Disconnect" and RustConn owns the child process, THE RustConn SHALL gracefully terminate the viewer child process, waiting up to 5 seconds for it to exit.
3. IF the child process does not exit within 5 seconds after graceful termination, THEN THE RustConn SHALL force-kill the process and deregister the session.
4. WHEN the user selects "Stop tracking", THE RustConn SHALL deregister the session and mark it ended without terminating the child process, within 1 second.
5. WHERE RustConn does not own the child process of a detaching viewer, WHEN the user selects "Disconnect", THE RustConn SHALL inform the user that the process cannot be terminated and leave the session unchanged.
6. THE RustConn SHALL order context menu items per GNOME HIG: primary action at the top, destructive action at the bottom.
7. IF a detaching viewer hands control to a daemon and `try_wait()` reports exit while the window is still open, THEN THE RustConn SHALL not close the session automatically via the poll timer and SHALL keep it active until "Stop tracking" is selected.

### Requirement 6: Orthogonal state indication for existing sessions (Phase 2)

**User Story:** As a user with multiple sessions, I want to distinguish connection states in the sidebar by shape and icon, so that perception does not depend on color.

#### Acceptance Criteria

1. WHILE a connection has an active session, THE RustConn SHALL show a "connected" indicator that combines the existing green icon with a shape that is unique among all states in this list, for the entire lifetime of the active session.
2. WHERE a session is in a split, THE RustConn SHALL show a marker whose shape differs from all other state indicators, filled with the corresponding split pane color from the existing `split_colors` map, sized no larger than the connection's main status icon.
3. WHILE a connection has an active external viewer session, THE RustConn SHALL show the external viewer emblem whose shape differs from all other state indicators.
4. THE RustConn SHALL convey each state signal so that icon and shape remain mutually distinguishable when rendered in grayscale (no color), not by color alone.
5. IF a single connection matches multiple states at once (active session, split, external viewer), THEN THE RustConn SHALL show an indicator for each present state separately while keeping their shapes distinguishable.

### Requirement 7: Smart double-click that focuses an existing session (Phase 2)

**User Story:** As a user, I want double-clicking a connection with an existing session to focus that session instead of creating a duplicate, so that I avoid redundant identical sessions.

#### Acceptance Criteria

1. WHEN the user double-clicks a connection that has exactly one active session (a session in the "connecting" or "connected" state that has not ended), THE RustConn SHALL focus that existing session — select the session's owner tab and set input focus on the session — instead of creating a new one.
2. WHERE the focused session is in a split, THE RustConn SHALL select the owner tab and focus the correct pane via `focus_pane`.
3. WHEN the user double-clicks a connection that has no active session, THE RustConn SHALL start exactly one new session, as in the existing behavior.
4. WHEN the user double-clicks a connection that has two or more active sessions, THE RustConn SHALL focus the session with the latest creation time (the most recent).
5. WHERE the user holds a modifier key (Shift or Ctrl) during the double-click OR selects the "Open new session" context menu item, THE RustConn SHALL force-create a new session regardless of the number of existing active sessions.
6. IF the session selected for focusing ends between its selection and the focus attempt, THEN THE RustConn SHALL start exactly one new session for that connection.

### Requirement 8: Non-functional constraints and standards compliance

**User Story:** As the project maintainer, I want the feature to respect crate boundaries, GNOME HIG, localization, and security requirements, so that codebase quality is preserved.

#### Acceptance Criteria

1. THE RustConn SHALL place pure decision logic (the external viewer predicate) in the `rustconn-core` crate without any of the tokens `use gtk4`, `use adw`, `use vte4`, `gtk4::`, `adw::`, `vte4::` in the `rustconn-core` and `rustconn-cli` crates.
2. THE RustConn SHALL wrap all UI strings at the call sites `.set_label()`, `.set_title()`, `.set_tooltip_text()`, `Button::with_label()`, and `display_name()` via `i18n()` or `i18n_f()`, excluding logging strings, CSS, icon names, and action names.
3. WHEN a new UI string is added, THE RustConn SHALL update the translation template via `po/update-pot.sh` and run `msgmerge --update` for the 16 languages (be, cs, da, de, es, fr, it, kk, nl, pl, pt, sk, sv, uk, uz, zh-cn).
4. WHEN a confirmation dialog is needed, THE RustConn SHALL use `adw::AlertDialog` and not use `gtk::MessageDialog`.
5. THE RustConn SHALL define `rustconn-core` crate errors via `thiserror` and not use `unwrap()` or `expect()` in non-test code.
6. THE RustConn SHALL log via `tracing` without writing secrets to the log and not use `println!` or `eprintln!`.
7. THE RustConn SHALL default to no tab without adding a settings toggle to choose between a placeholder and no tab (YAGNI).
