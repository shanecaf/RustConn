//! Property-based tests for Workspace Profile serialization and CRUD
//!
//! Validates that workspace profiles survive roundtrip serialization
//! and that the manager enforces name uniqueness and auto-cleanup.

use proptest::prelude::*;
use rustconn_core::ConfigManager;
use rustconn_core::models::{WorkspaceEntry, WorkspaceProfile, WorkspaceSplitLayout};
use rustconn_core::session::SessionType;
use rustconn_core::workspace::WorkspaceProfileManager;
use tempfile::TempDir;
use uuid::Uuid;

// ========== Generators ==========

fn arb_workspace_name() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 _-]{0,30}".prop_map(|s| s.trim().to_string())
}

fn arb_protocol() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("ssh".to_string()),
        Just("rdp".to_string()),
        Just("vnc".to_string()),
        Just("spice".to_string()),
        Just("telnet".to_string()),
    ]
}

fn arb_session_type() -> impl Strategy<Value = SessionType> {
    prop_oneof![Just(SessionType::Embedded), Just(SessionType::External),]
}

fn arb_workspace_entry() -> impl Strategy<Value = WorkspaceEntry> {
    (
        any::<u128>().prop_map(|n| Uuid::from_u128(n)),
        "[a-z][a-z0-9-]{0,15}",
        arb_protocol(),
        arb_session_type(),
        0usize..20,
    )
        .prop_map(|(id, name, proto, session_type, tab_idx)| {
            WorkspaceEntry::new(id, name.to_string(), proto, session_type, tab_idx)
        })
}

fn arb_split_layout() -> impl Strategy<Value = WorkspaceSplitLayout> {
    (
        any::<bool>(),
        any::<bool>(),
        0.0f64..=1.0f64,
        proptest::option::of(0usize..8),
        prop::collection::vec(0usize..8, 0..4),
        0usize..4,
        prop::collection::vec(any::<bool>(), 0..4),
        proptest::option::of(0usize..8),
    )
        .prop_map(
            |(is_split, horizontal, ratio, guest_idx, guests, extra, dirs, owner_idx)| {
                WorkspaceSplitLayout {
                    is_split,
                    horizontal,
                    split_ratio: ratio,
                    split_guest_entry_index: guest_idx,
                    split_guests: guests,
                    extra_splits: extra,
                    split_directions: dirs,
                    split_owner_entry_index: owner_idx,
                }
            },
        )
}

fn arb_workspace_profile() -> impl Strategy<Value = WorkspaceProfile> {
    (
        arb_workspace_name(),
        prop::collection::vec(arb_workspace_entry(), 0..8),
        arb_split_layout(),
    )
        .prop_map(|(name, entries, layout)| {
            let mut ws = WorkspaceProfile::new(name);
            for e in entries {
                ws.add_entry(e);
            }
            ws.set_split_layout(layout);
            ws
        })
}

// ========== Tests ==========

proptest! {
    #[test]
    fn workspace_profile_serialization_roundtrip(ws in arb_workspace_profile()) {
        let toml_str = toml::to_string(&ws).expect("serialize");
        let restored: WorkspaceProfile = toml::from_str(&toml_str).expect("deserialize");
        prop_assert_eq!(&ws.name, &restored.name);
        prop_assert_eq!(ws.entries.len(), restored.entries.len());
        for (orig, rest) in ws.entries.iter().zip(restored.entries.iter()) {
            prop_assert_eq!(orig.connection_id, rest.connection_id);
            prop_assert_eq!(&orig.connection_name, &rest.connection_name);
            prop_assert_eq!(&orig.protocol, &rest.protocol);
            prop_assert_eq!(orig.tab_index, rest.tab_index);
        }
        prop_assert_eq!(ws.split_layout.is_split, restored.split_layout.is_split);
        prop_assert_eq!(ws.split_layout.horizontal, restored.split_layout.horizontal);
    }

    #[test]
    fn workspace_remove_connection_reindexes(
        entries in prop::collection::vec(arb_workspace_entry(), 2..10)
    ) {
        let mut ws = WorkspaceProfile::new("Test");
        for e in &entries {
            ws.add_entry(e.clone());
        }
        let target_id = entries[0].connection_id;
        let removed = ws.remove_connection(target_id);
        prop_assert_eq!(removed, 1);
        // All remaining entries have sequential tab_index
        for (i, entry) in ws.entries.iter().enumerate() {
            prop_assert_eq!(entry.tab_index, i);
        }
        prop_assert!(!ws.contains_connection(target_id));
    }

    #[test]
    fn workspace_manager_name_uniqueness(
        name in arb_workspace_name()
    ) {
        let temp_dir = TempDir::new().expect("temp dir");
        let config_manager = ConfigManager::with_config_dir(temp_dir.path().to_path_buf());
        let mut mgr = WorkspaceProfileManager::new_empty(config_manager);

        let ws1 = WorkspaceProfile::new(&name);
        mgr.create(ws1).expect("first create");

        let ws2 = WorkspaceProfile::new(&name);
        let result = mgr.create(ws2);
        prop_assert!(result.is_err());
    }

    #[test]
    fn workspace_manager_rename(
        original_name in arb_workspace_name(),
        new_name in arb_workspace_name()
    ) {
        let temp_dir = TempDir::new().expect("temp dir");
        let config_manager = ConfigManager::with_config_dir(temp_dir.path().to_path_buf());
        let mut mgr = WorkspaceProfileManager::new_empty(config_manager);

        let ws = WorkspaceProfile::new(&original_name);
        let id = mgr.create(ws).expect("create");

        let result = mgr.rename(id, new_name.clone());
        if original_name.eq_ignore_ascii_case(&new_name) || mgr.find_by_name(&new_name).is_none_or(|p| p.id == id) {
            // Same name (case-insensitive) or no conflict → should succeed
            prop_assert!(result.is_ok());
            let profile = mgr.get(id).expect("profile exists");
            prop_assert_eq!(&profile.name, &new_name);
        }
    }

    #[test]
    fn workspace_manager_on_connection_deleted(
        entries in prop::collection::vec(arb_workspace_entry(), 1..5)
    ) {
        let temp_dir = TempDir::new().expect("temp dir");
        let config_manager = ConfigManager::with_config_dir(temp_dir.path().to_path_buf());
        let mut mgr = WorkspaceProfileManager::new_empty(config_manager);

        let mut ws = WorkspaceProfile::new("Test WS");
        for e in &entries {
            ws.add_entry(e.clone());
        }
        let ws_id = mgr.create(ws).expect("create");

        let deleted_conn = entries[0].connection_id;
        mgr.on_connection_deleted(deleted_conn).expect("cleanup");

        let profile = mgr.get(ws_id).expect("profile");
        prop_assert!(!profile.contains_connection(deleted_conn));
    }
}
