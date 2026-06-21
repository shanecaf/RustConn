//! Workspace profile management
//!
//! Provides CRUD operations for named workspace profiles — saved sets
//! of connections with layout that can be restored on demand.

mod manager;

pub use manager::WorkspaceProfileManager;
