# Design Document: Visual Tunnel Builder

## Overview

Візуальний конструктор SSH-тунелів реалізується як wizard-діалог на базі `adw::NavigationView` (аналогічно існуючому `ConnectionWizard`). Він замінює поточний flat-діалог `show_add_edit_dialog` у `tunnel.rs` на 3-крокову навігацію з візуальною діаграмою шляху тунелю.

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    rustconn (GUI crate)                       │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │         TunnelBuilderDialog (new module)               │   │
│  │                                                        │   │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────┐  │   │
│  │  │ Step1Page   │  │ Step2Page     │  │ Step3Page   │  │   │
│  │  │ Connection  │→ │ Port Forwards │→ │ Review &    │  │   │
│  │  │ & Name      │  │ & Options     │  │ Confirm     │  │   │
│  │  └─────────────┘  └──────────────┘  └─────────────┘  │   │
│  │                                                        │   │
│  │  ┌──────────────────────────────────────────────────┐ │   │
│  │  │         TunnelPathDiagram (widget)                │ │   │
│  │  │  [Localhost:port] ──→ [Bastion] ──→ [Target:port] │ │   │
│  │  └──────────────────────────────────────────────────┘ │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────┐                                    │
│  │ TunnelManagerWindow  │ ← calls TunnelBuilderDialog        │
│  └──────────────────────┘                                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustconn-core (logic crate)                  │
│                                                              │
│  ┌──────────────────┐  ┌───────────────────┐                │
│  │ StandaloneTunnel │  │ TunnelManager     │                │
│  │ PortForward      │  │ start/stop/status │                │
│  │ PortForwardDir.  │  └───────────────────┘                │
│  └──────────────────┘                                        │
│                                                              │
│  ┌──────────────────────────────────────────┐                │
│  │ tunnel_preview (new module)              │                │
│  │ fn build_tunnel_preview_command(...)     │                │
│  │   → String                               │                │
│  └──────────────────────────────────────────┘                │
└─────────────────────────────────────────────────────────────┘
```

### File Structure

```
rustconn/src/dialogs/
├── tunnel.rs                          # Existing — TunnelManagerWindow (keep, modify to call builder)
├── tunnel_builder/
│   ├── mod.rs                         # TunnelBuilderDialog struct, new()/present()/wire_callbacks()
│   ├── step_connection.rs             # Step 1: name + SSH connection + bastion selection
│   ├── step_forwards.rs               # Step 2: port forward rules + options (auto-start/reconnect)
│   ├── step_review.rs                 # Step 3: diagram + SSH preview + confirm
│   └── path_diagram.rs               # TunnelPathDiagram widget (reusable)

rustconn-core/src/
├── tunnel_preview.rs                  # SSH command preview builder (new)
```

## Data Models

### Existing (no changes)

```rust
// rustconn-core/src/models/tunnel.rs
pub struct StandaloneTunnel {
    pub id: Uuid,
    pub name: String,
    pub connection_id: Uuid,
    pub forwards: Vec<PortForward>,
    pub auto_start: bool,
    pub auto_reconnect: bool,
    pub enabled: bool,
}

// rustconn-core/src/models/protocol.rs
pub struct PortForward {
    pub direction: PortForwardDirection,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
}

pub enum PortForwardDirection { Local, Remote, Dynamic }
```

### New: Tunnel Preview (rustconn-core)

```rust
// rustconn-core/src/tunnel_preview.rs

/// Parameters for generating an SSH tunnel command preview
pub struct TunnelPreviewParams<'a> {
    pub host: &'a str,
    pub port: u16,
    pub username: Option<&'a str>,
    pub forwards: &'a [PortForward],
    pub proxy_jump: Option<&'a str>,
    pub identity_file: Option<&'a str>,
}

/// Builds a human-readable SSH command string for preview purposes.
/// Does NOT include passwords or sensitive data.
pub fn build_tunnel_preview_command(params: &TunnelPreviewParams) -> String;
```

### New: Builder Context (rustconn GUI)

```rust
// rustconn/src/dialogs/tunnel_builder/mod.rs

/// Context passed to TunnelBuilderDialog (avoids >6 params)
pub struct TunnelBuilderContext {
    pub state: SharedAppState,
    pub tunnel_manager: SharedTunnelManager,
    pub parent_window: adw::Window,
    /// Callback invoked after successful save (to refresh tunnel list)
    pub on_save: Rc<RefCell<Option<Box<dyn Fn()>>>>,
}

/// Intermediate state held during wizard navigation
struct WizardState {
    /// Tunnel being edited (None = creating new)
    editing_id: Option<Uuid>,
    /// Selected SSH connection
    selected_connection: Option<Connection>,
    /// Resolved bastion host (from jump_host_id or manual selection)
    bastion_connection: Option<Connection>,
    /// Port forwarding rules
    forwards: Vec<PortForward>,
    /// Tunnel name
    name: String,
    /// Options
    auto_start: bool,
    auto_reconnect: bool,
}
```

## UI Design

### Step 1: Connection & Name

```
┌─────────────────────────────────────────────────┐
│ ← New Tunnel                          [Next →]  │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌─ General ──────────────────────────────────┐ │
│  │ Tunnel Name        [________________]      │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
│  ┌─ SSH Connection ───────────────────────────┐ │
│  │ Connection    [▼ prod-server (user@host) ] │ │
│  │ 🔍 Filter    [________________]            │ │
│  │                                            │ │
│  │ Jump Host     [▼ (None)                  ] │ │
│  │               [▼ bastion (admin@10.0.0.1)] │ │
│  │                                            │ │
│  │ [+ New SSH Connection]                     │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
│  ┌─ Path Preview ─────────────────────────────┐ │
│  │  [localhost] ──→ [bastion] ──→ [target]    │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Widgets:**
- `adw::EntryRow` — tunnel name (validation: 1–128 chars)
- `adw::ComboRow` — SSH connection selection (filtered by protocol=SSH)
- `gtk4::SearchEntry` — filter connections by name/host
- `adw::ComboRow` — jump host override (optional, shows "(None)" + SSH connections)
- `gtk4::Button` — "New SSH Connection" (opens ConnectionDialog)
- `TunnelPathDiagram` — live preview of the chain

### Step 2: Port Forwards & Options

```
┌─────────────────────────────────────────────────┐
│ ← Port Forwards                       [Next →]  │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌─ Port Forwards ───────────────────────────┐  │
│  │ ▶ L 3306 → db.internal:3306    [🗑]      │  │
│  │   Direction  [▼ Local (-L)           ]    │  │
│  │   Local Port [3306        ]               │  │
│  │   Remote Host[db.internal ]               │  │
│  │   Remote Port[3306        ]               │  │
│  │                                           │  │
│  │ ▶ D 1080 (SOCKS)               [🗑]      │  │
│  │   Direction  [▼ Dynamic (-D)         ]    │  │
│  │   Local Port [1080        ]               │  │
│  └───────────────────────────────────────────┘  │
│                                                  │
│  [+ Add Forward]                                 │
│                                                  │
│  ┌─ Options ─────────────────────────────────┐  │
│  │ Auto-start on launch           [toggle]   │  │
│  │ Auto-reconnect on failure      [toggle]   │  │
│  └───────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Path Preview ────────────────────────────┐  │
│  │  [:3306] ──→ [bastion] ──→ [db:3306]     │  │
│  └───────────────────────────────────────────┘  │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Widgets:**
- `adw::PreferencesGroup` — forwards list
- `adw::ExpanderRow` — each forward rule (title = display_summary)
- `gtk4::DropDown` — direction selector (Local/Remote/Dynamic)
- `adw::SpinRow` — local port, remote port (1–65535)
- `adw::EntryRow` — remote host (max 253 chars)
- `gtk4::Button` — delete (icon: edit-delete-symbolic)
- `gtk4::Button` — "Add Forward" (flat style)
- `adw::SwitchRow` — auto-start, auto-reconnect
- `TunnelPathDiagram` — updates based on first forward rule

### Step 3: Review & Confirm

```
┌─────────────────────────────────────────────────┐
│ ← Review                            [Create ✓]  │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌─ Tunnel Path ─────────────────────────────┐  │
│  │                                           │  │
│  │  ┌──────────┐    ┌──────────┐    ┌──────┐│  │
│  │  │localhost │───→│ bastion  │───→│target ││  │
│  │  │ :3306    │    │10.0.0.1  │    │db:3306││  │
│  │  └──────────┘    └──────────┘    └──────┘│  │
│  │                                           │  │
│  └───────────────────────────────────────────┘  │
│                                                  │
│  ┌─ Summary ─────────────────────────────────┐  │
│  │ Name:       MySQL prod                    │  │
│  │ Connection: prod-server (user@10.0.1.5)   │  │
│  │ Jump Host:  bastion (admin@10.0.0.1)      │  │
│  │ Forwards:   L 3306→db:3306, D 1080       │  │
│  │ Auto-start: Yes                           │  │
│  │ Reconnect:  Yes                           │  │
│  └───────────────────────────────────────────┘  │
│                                                  │
│  ┌─ SSH Command ─────────────────────────────┐  │
│  │ ssh -N -L 3306:db.internal:3306           │  │
│  │     -D 1080 -J admin@10.0.0.1:22         │  │
│  │     -p 22 user@10.0.1.5          [📋]    │  │
│  └───────────────────────────────────────────┘  │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Widgets:**
- `TunnelPathDiagram` — full diagram with status indicators (edit mode)
- `adw::PreferencesGroup` — summary (ActionRows with read-only values)
- `gtk4::TextView` (monospace, non-editable) — SSH command preview
- `gtk4::Button` — copy to clipboard (icon: edit-copy-symbolic)
- `gtk4::Button` — "Create" / "Save" (suggested-action CSS class)

## TunnelPathDiagram Widget

### Implementation Approach

Використовуємо **gtk4::Box** з кастомними стилізованими віджетами (не DrawingArea/Cairo), щоб залишатися в рамках Libadwaita:

```rust
// rustconn/src/dialogs/tunnel_builder/path_diagram.rs

pub struct TunnelPathDiagram {
    container: gtk4::Box,  // horizontal, spacing=0
    nodes: Vec<DiagramNode>,
    arrows: Vec<gtk4::Label>,  // "→" styled labels
}

struct DiagramNode {
    frame: gtk4::Frame,
    icon: gtk4::Image,
    host_label: gtk4::Label,
    port_label: gtk4::Label,
    status_dot: gtk4::Label,  // "●" with CSS class
}
```

### Visual Structure

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│ 🖥️ localhost│  ────→  │ 🔒 bastion  │  ────→  │ 🖧 target   │
│   :3306     │         │ 10.0.0.1    │         │ db:3306     │
│     ●       │         │     ●       │         │     ●       │
└─────────────┘         └─────────────┘         └─────────────┘
```

- Кожен вузол — `gtk4::Frame` з `gtk4::Box` (vertical) всередині
- Стрілки — `gtk4::Label` з текстом "→" та CSS-стилем (dim, large font)
- Статус-індикатор — `gtk4::Label` з "●" та CSS-класом (success/warning/error)
- Адаптивність: при вузькому вікні переключається на вертикальний layout

### CSS Classes

```css
/* Додати до rustconn/assets/style.css */

.tunnel-diagram {
    padding: 12px;
    margin: 6px 0;
}

.tunnel-node {
    padding: 8px 12px;
    border-radius: 8px;
    border: 1px solid alpha(@borders, 0.5);
    background-color: alpha(@card_bg_color, 0.5);
}

.tunnel-node.success {
    border-color: @success_color;
    background-color: alpha(@success_color, 0.08);
}

.tunnel-node.warning {
    border-color: @warning_color;
    background-color: alpha(@warning_color, 0.08);
}

.tunnel-node.error {
    border-color: @error_color;
    background-color: alpha(@error_color, 0.08);
}

.tunnel-arrow {
    font-size: 1.4em;
    opacity: 0.6;
    margin: 0 8px;
}

.tunnel-arrow.active {
    opacity: 1.0;
    color: @success_color;
}

.tunnel-status-dot {
    font-size: 0.7em;
    margin-top: 4px;
}

@keyframes tunnel-pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 1.0; }
}

.tunnel-node.starting .tunnel-status-dot {
    animation: tunnel-pulse 1.5s ease-in-out infinite;
}
```

## Key Interactions

### Navigation Flow

```
[Add Tunnel] → Step1 → Step2 → Step3 → [Save] → close + refresh list
                 ↑        ↑        │
                 └────────┘        │ (Back buttons via NavigationView)
                          ↑        │
                          └────────┘
```

### Validation Rules

| Step | Field | Rule | Error |
|------|-------|------|-------|
| 1 | Name | 1–128 chars, non-empty | "Tunnel name is required" |
| 1 | Connection | Must be selected | "Select an SSH connection" |
| 2 | Local port | 1–65535 | "Port must be 1–65535" |
| 2 | Local port | <1024 | Warning (non-blocking) |
| 2 | Remote host | Non-empty for L/R | "Remote host is required" |
| 2 | Remote port | 1–65535 for L/R | "Port must be 1–65535" |
| 2 | Rules count | ≤20 | Hide "Add" button at limit |

### Status Update Flow (Edit Mode)

```
TunnelManager.status(id) → TunnelStatus
    → match status:
        Running  → .success CSS class + "●" green
        Starting → .warning CSS class + pulse animation
        Failed   → .error CSS class + tooltip with error text
        Stopped  → dim/insensitive style
```

Status polling: `glib::timeout_add_seconds_local(2, ...)` while dialog is open.

## Integration Points

### Modification to Existing Code

1. **`rustconn/src/dialogs/tunnel.rs`** — Replace `show_add_edit_dialog()` call with `TunnelBuilderDialog::new().present()`:
   ```rust
   // In wire_tunnel_row_actions() and the "Add" button handler:
   // Before: show_add_edit_dialog(parent, state, existing, ...)
   // After:
   let builder = TunnelBuilderDialog::new(TunnelBuilderContext { ... });
   if let Some(tunnel) = existing {
       builder.set_tunnel(tunnel);
   }
   builder.connect_save(move || { refresh_from_context(&ctx); });
   builder.present(&parent);
   ```

2. **`rustconn/src/dialogs/mod.rs`** — Add module registration:
   ```rust
   pub mod tunnel_builder;
   ```

3. **`rustconn-core/src/lib.rs`** — Export new module:
   ```rust
   pub mod tunnel_preview;
   ```

### New Module: tunnel_preview (rustconn-core)

```rust
// rustconn-core/src/tunnel_preview.rs

use crate::models::{PortForward, PortForwardDirection};

pub struct TunnelPreviewParams<'a> {
    pub host: &'a str,
    pub port: u16,
    pub username: Option<&'a str>,
    pub forwards: &'a [PortForward],
    pub proxy_jump: Option<&'a str>,
    pub identity_file: Option<&'a str>,
}

/// Builds a human-readable SSH command for display purposes.
/// Never includes passwords or secrets.
#[must_use]
pub fn build_tunnel_preview_command(params: &TunnelPreviewParams) -> String {
    let mut parts = vec!["ssh".to_string(), "-N".to_string()];

    for fwd in params.forwards {
        parts.extend(fwd.to_ssh_arg());
    }

    if let Some(jump) = params.proxy_jump {
        parts.push("-J".to_string());
        parts.push(jump.to_string());
    }

    if let Some(key) = params.identity_file {
        parts.push("-i".to_string());
        parts.push(key.to_string());
    }

    if params.port != 22 {
        parts.push("-p".to_string());
        parts.push(params.port.to_string());
    }

    let destination = if let Some(user) = params.username {
        format!("{user}@{}", params.host)
    } else {
        params.host.to_string()
    };
    parts.push(destination);

    parts.join(" ")
}
```

## Accessibility

| Element | ATK Property | Value |
|---------|-------------|-------|
| Diagram container | accessible-role | Group |
| Diagram container | accessible-description | Dynamic: "Tunnel: localhost:3306 → bastion (10.0.0.1) → db.internal:3306" |
| Each node frame | accessible-label | "Localhost port 3306" / "Bastion 10.0.0.1" / "Target db.internal:3306" |
| Status dot | accessible-label | "Status: Running" / "Status: Failed: connection refused" |
| Arrow labels | aria-hidden | true (decorative) |
| Copy button | tooltip + label | "Copy SSH command to clipboard" |
| Delete forward btn | tooltip + label | "Remove this port forwarding rule" |

Focus order: NavigationView handles back button focus. Within each page: top-to-bottom through form fields, then action buttons.

## Error Handling

All errors use `thiserror` in rustconn-core. UI displays errors via:
- Inline validation: `label.error` CSS class below the field
- Toast notifications: `adw::Toast` for transient messages (copy success)
- Alert dialogs: `crate::alert::show_error()` for critical failures (save failed)

## Performance Considerations

- SSH connection list filtering: debounce 150ms on SearchEntry input
- Diagram updates: immediate (no debounce needed — lightweight widget updates)
- Status polling: every 2 seconds via `glib::timeout_add_seconds_local`, stopped when dialog closes
- Connection list: loaded once on dialog open, refreshed only after "New SSH Connection" save


## Components and Interfaces

### Component: TunnelBuilderDialog (rustconn/src/dialogs/tunnel_builder/mod.rs)

**Public Interface:**
```rust
pub struct TunnelBuilderDialog { /* ... */ }

impl TunnelBuilderDialog {
    /// Creates a new tunnel builder wizard
    pub fn new(ctx: TunnelBuilderContext) -> Rc<Self>;

    /// Pre-populates the wizard with an existing tunnel (edit mode)
    pub fn set_tunnel(&self, tunnel: &StandaloneTunnel);

    /// Registers a callback invoked after successful save
    pub fn connect_save<F: Fn() + 'static>(&self, f: F);

    /// Presents the dialog as a child of the given widget
    pub fn present(&self, parent: &impl IsA<gtk4::Widget>);
}
```

**Dependencies:** SharedAppState, SharedTunnelManager, ConnectionDialog

### Component: TunnelPathDiagram (rustconn/src/dialogs/tunnel_builder/path_diagram.rs)

**Public Interface:**
```rust
pub struct TunnelPathDiagram { /* ... */ }

impl TunnelPathDiagram {
    /// Creates a new empty diagram
    pub fn new() -> Self;

    /// Returns the root widget for embedding in a container
    pub fn widget(&self) -> &gtk4::Widget;

    /// Updates the diagram with current tunnel configuration
    pub fn update(
        &self,
        local_port: Option<u16>,
        bastion: Option<&str>,
        target_host: Option<&str>,
        target_port: Option<u16>,
        direction: Option<PortForwardDirection>,
    );

    /// Updates the status indicators (edit mode only)
    pub fn set_status(&self, status: &TunnelStatus);

    /// Hides status indicators (create mode)
    pub fn hide_status(&self);

    /// Returns accessible description text for the current state
    pub fn accessible_description(&self) -> String;
}
```

**Dependencies:** None (pure UI widget, receives data via update())

### Component: tunnel_preview (rustconn-core/src/tunnel_preview.rs)

**Public Interface:**
```rust
pub struct TunnelPreviewParams<'a> {
    pub host: &'a str,
    pub port: u16,
    pub username: Option<&'a str>,
    pub forwards: &'a [PortForward],
    pub proxy_jump: Option<&'a str>,
    pub identity_file: Option<&'a str>,
}

/// Builds a human-readable SSH command for display.
/// Never includes passwords or secrets.
#[must_use]
pub fn build_tunnel_preview_command(params: &TunnelPreviewParams) -> String;
```

**Dependencies:** crate::models::PortForward

### Component: Step Pages

Each step page follows the same pattern:

```rust
pub struct StepConnectionPage {
    pub page: adw::NavigationPage,
    // internal widgets...
}

impl StepConnectionPage {
    pub fn new() -> Self;
    pub fn connect_next<F: Fn() + 'static>(&self, f: F);
    pub fn set_connection(&self, conn: &Connection);
    pub fn selected_connection_id(&self) -> Option<Uuid>;
    pub fn tunnel_name(&self) -> String;
    pub fn bastion_connection(&self) -> Option<Connection>;
}
```

Analogous for `StepForwardsPage` and `StepReviewPage`.

## Correctness Properties

### Property 1: Data integrity
**Validates: Requirements 1.4, 4.6, 4.9**

Saving a tunnel always produces a valid `StandaloneTunnel` — non-empty name, valid connection_id referencing an existing connection, all port forwards with ports in 1–65535 range.

### Property 2: No data loss on cancel
**Validates: Requirements 1.7**

Closing the wizard without saving never modifies `AppSettings.standalone_tunnels`. The wizard operates on a local `WizardState` copy until explicit save.

### Property 3: UUID preservation
**Validates: Requirements 6.3**

Editing an existing tunnel preserves its `id` field. The save operation updates in-place rather than delete+create.

### Property 4: Crate boundary
**Validates: Requirements 9.5, 9.7**

`tunnel_preview.rs` in rustconn-core contains zero GTK imports. All UI code lives in `rustconn/src/dialogs/tunnel_builder/`.

### Property 5: No secrets in preview
**Validates: Requirements 7.1**

`build_tunnel_preview_command()` never accepts or outputs passwords, tokens, or SecretString values. Only host/port/username/key-path are included.

### Property 6: Status consistency
**Validates: Requirements 5.5, 5.6**

Status indicators reflect `TunnelManager.status()` with at most 2-second staleness (polling interval). Status is hidden for new tunnels that have no process.

### Property 7: Validation completeness
**Validates: Requirements 1.4, 1.5, 4.6**

The "Next" / "Save" button is disabled whenever validation fails. It is impossible to save a tunnel with an empty name, no selected connection, or invalid port values.

## Testing Strategy

### Unit Tests (rustconn-core)

| Test | Location | What it verifies |
|------|----------|-----------------|
| `test_preview_basic` | `tunnel_preview.rs` | Basic command: `ssh -N -L 8080:localhost:80 -p 22 user@host` |
| `test_preview_with_proxy_jump` | `tunnel_preview.rs` | `-J` argument included when proxy_jump is Some |
| `test_preview_dynamic` | `tunnel_preview.rs` | Dynamic forward: `ssh -N -D 1080 user@host` |
| `test_preview_multiple_forwards` | `tunnel_preview.rs` | Multiple -L/-R/-D in one command |
| `test_preview_no_forwards` | `tunnel_preview.rs` | Just `ssh -N user@host` when forwards is empty |
| `test_preview_no_username` | `tunnel_preview.rs` | Destination is just `host` without `@` |
| `test_preview_default_port` | `tunnel_preview.rs` | No `-p` when port == 22 |
| `test_preview_identity_file` | `tunnel_preview.rs` | `-i /path/to/key` included |

### Integration Tests (manual / UI)

| Scenario | Steps | Expected |
|----------|-------|----------|
| Create tunnel | Add → fill name + connection → add forward → save | Tunnel appears in list |
| Edit tunnel | Click Edit → modify name → save | Name updated, UUID preserved |
| Cancel without save | Add → fill fields → close dialog | No tunnel created |
| Validation blocking | Leave name empty → try Next | Button disabled, error shown |
| Privileged port warning | Enter port 80 | Warning label shown, save still allowed |
| Empty connections | Remove all SSH connections → open wizard | Empty state message shown |
| Status indicators | Start tunnel → open edit | Green indicators on diagram |
| Copy command | Click copy button on step 3 | Command in clipboard, toast shown |
