//! Split view module for tab-scoped split layouts
//!
//! This module provides the GUI layer implementation for the split view redesign.
//! It bridges the core data models from `rustconn-core::split` with GTK4/libadwaita
//! widgets.
//!
//! # Architecture
//!
//! The split view system is divided between two crates:
//!
//! - **`rustconn-core::split`**: Core data models (`SplitLayoutModel`, `PanelNode`, etc.)
//! - **`rustconn::split_view`**: GUI adapters and GTK widget management
//!
//! This separation ensures that business logic can be tested without GTK dependencies.
//!
//! # Module Structure
//!
//! - `adapter` - `SplitViewAdapter` bridging core models to GTK widgets
//! - `types` - GUI-specific types (`DropSource`, `ConnectionId`)
//! - `bridge` - `SplitViewBridge` providing legacy-compatible API over new system
//!
//! # Example
//!
//! ```ignore
//! use rustconn::split_view::{DropSource, ConnectionId, SplitViewAdapter};
//! use rustconn_core::split::{SessionId, SplitDirection};
//!
//! // Create a new split view adapter
//! let mut adapter = SplitViewAdapter::new();
//!
//! // Split the focused panel vertically
//! let new_panel_id = adapter.split(SplitDirection::Vertical).unwrap();
//!
//! // Create a drop source for a sidebar item
//! let connection_id = ConnectionId::new();
//! let source = DropSource::sidebar_item(connection_id);
//!
//! // Create a drop source for a root tab
//! let session_id = SessionId::new();
//! let source = DropSource::root_tab(session_id);
//! ```

mod adapter;
mod bridge;
pub mod types;

// Re-export the new adapter
pub use adapter::SplitViewAdapter;

// Re-export the bridge for legacy-compatible API (replaces SplitTerminalView)
pub use bridge::{
    SPLIT_COLOR_VALUES, SPLIT_PANE_COLORS, SessionColorMap, SharedSessions, SharedTerminals,
    SplitDirection, SplitViewBridge, create_colored_circle_icon, get_split_color_class,
    get_split_indicator_class, get_tab_color_class,
};

// Re-export GUI-specific types
pub use types::{ConnectionId, DropOutcome, DropSource, EvictionAction, SourceCleanup};

use gtk4::prelude::*;
use rustconn_core::models::WorkspaceSplitLayout;

/// Restores a saved workspace split layout onto the active window.
///
/// Creates the initial split plus any extra splits needed for multi-panel
/// layouts. All splits fire in a single idle iteration so the active tab
/// does not change between them (SSH tabs connecting in the background
/// could steal focus between timeouts).
///
/// ponytail: restores split direction only, not `split_ratio` (panes open 50/50);
/// upgrade path: expose a ratio setter on `SplitViewBridge` and apply it post-split.
pub fn apply_layout(window: &gtk4::Window, layout: &WorkspaceSplitLayout) {
    if !layout.is_split {
        return;
    }
    let extra = layout.extra_splits;
    // Use per-split directions if available, otherwise fall back to the
    // single `horizontal` field for all splits (backward compat).
    let directions: Vec<bool> = if layout.split_directions.is_empty() {
        vec![layout.horizontal; extra + 1]
    } else {
        layout.split_directions.clone()
    };
    let window_weak = window.downgrade();
    gtk4::glib::idle_add_local_once(move || {
        let Some(win) = window_weak.upgrade() else {
            return;
        };
        // Fire all splits in one go — the bridge accumulates panels
        // synchronously within the same main-loop iteration.
        for i in 0..=extra {
            let action = if directions.get(i).copied().unwrap_or(true) {
                "win.split-horizontal"
            } else {
                "win.split-vertical"
            };
            let _ = WidgetExt::activate_action(&win, action, None);
        }
    });
}
