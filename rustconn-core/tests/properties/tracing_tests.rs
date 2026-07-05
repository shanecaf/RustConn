//! Tests for the tracing span-name constants.
//!
//! The `tracing` subscriber is initialised by the application entry point;
//! only the canonical span names are exercised here.

use rustconn_core::span_names;

#[test]
fn span_names_are_defined() {
    // Verify all required span names are defined and non-empty
    assert!(!span_names::CONNECTION_ESTABLISH.is_empty());
    assert!(!span_names::CONNECTION_DISCONNECT.is_empty());
    assert!(!span_names::SEARCH_EXECUTE.is_empty());
    assert!(!span_names::SEARCH_CACHE_LOOKUP.is_empty());
    assert!(!span_names::IMPORT_EXECUTE.is_empty());
    assert!(!span_names::EXPORT_EXECUTE.is_empty());
    assert!(!span_names::CREDENTIAL_RESOLVE.is_empty());
    assert!(!span_names::CREDENTIAL_STORE.is_empty());
    assert!(!span_names::CONFIG_LOAD.is_empty());
    assert!(!span_names::CONFIG_SAVE.is_empty());
    assert!(!span_names::SESSION_START.is_empty());
    assert!(!span_names::SESSION_END.is_empty());
}

#[test]
fn span_names_follow_naming_convention() {
    // Verify span names follow the "category.operation" convention
    assert!(span_names::CONNECTION_ESTABLISH.contains('.'));
    assert!(span_names::CONNECTION_DISCONNECT.contains('.'));
    assert!(span_names::SEARCH_EXECUTE.contains('.'));
    assert!(span_names::IMPORT_EXECUTE.contains('.'));
    assert!(span_names::EXPORT_EXECUTE.contains('.'));
    assert!(span_names::CREDENTIAL_RESOLVE.contains('.'));
    assert!(span_names::SESSION_START.contains('.'));
}
