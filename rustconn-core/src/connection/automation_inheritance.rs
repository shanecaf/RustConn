//! Automation settings inheritance resolution.
//!
//! Resolves expect rules and post-login scripts by walking the group hierarchy
//! from a connection up to the root group. If the connection has its own
//! automation config (non-empty), it takes precedence. Otherwise, the first
//! group in the parent chain with non-empty rules is used.
//!
//! Cycle detection via `HashSet<Uuid>` ensures termination even with
//! malformed parent_id chains.

use std::collections::HashSet;

use uuid::Uuid;

use crate::automation::ExpectRule;
use crate::models::{AutomationConfig, Connection, ConnectionGroup};

/// Finds a group by ID in the slice.
fn find_group(id: Uuid, groups: &[ConnectionGroup]) -> Option<&ConnectionGroup> {
    groups.iter().find(|g| g.id == id)
}

/// Resolves the effective automation config for a connection.
///
/// # Algorithm
///
/// 1. If the connection has non-empty `expect_rules` or `post_login_scripts`,
///    return the connection's own config (no inheritance).
/// 2. Otherwise, walk the group hierarchy collecting rules from the first
///    group that has them.
/// 3. Expect rules and post-login scripts are resolved independently —
///    rules may come from one group and scripts from another (or the same).
///
/// # Returns
///
/// The effective `AutomationConfig` combining connection-level and inherited settings.
#[must_use]
pub fn resolve_automation(connection: &Connection, groups: &[ConnectionGroup]) -> AutomationConfig {
    let expect_rules = if connection.automation.expect_rules.is_empty() {
        resolve_expect_rules(connection.group_id, groups)
    } else {
        connection.automation.expect_rules.clone()
    };

    let post_login_scripts = if connection.automation.post_login_scripts.is_empty() {
        resolve_post_login_scripts(connection.group_id, groups)
    } else {
        connection.automation.post_login_scripts.clone()
    };

    AutomationConfig {
        expect_rules,
        post_login_scripts,
    }
}

/// Walks the group hierarchy to find inherited expect rules.
///
/// Returns the first non-empty `expect_rules` found in the parent chain,
/// or an empty vec if none found.
fn resolve_expect_rules(
    start_group_id: Option<Uuid>,
    groups: &[ConnectionGroup],
) -> Vec<ExpectRule> {
    let mut visited = HashSet::new();
    let mut current = start_group_id;

    while let Some(gid) = current {
        if !visited.insert(gid) {
            break; // Cycle detected
        }
        let Some(group) = find_group(gid, groups) else {
            break;
        };
        if !group.expect_rules.is_empty() {
            return group.expect_rules.clone();
        }
        current = group.parent_id;
    }

    Vec::new()
}

/// Walks the group hierarchy to find inherited post-login scripts.
///
/// Returns the first non-empty `post_login_scripts` found in the parent chain,
/// or an empty vec if none found.
fn resolve_post_login_scripts(
    start_group_id: Option<Uuid>,
    groups: &[ConnectionGroup],
) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut current = start_group_id;

    while let Some(gid) = current {
        if !visited.insert(gid) {
            break; // Cycle detected
        }
        let Some(group) = find_group(gid, groups) else {
            break;
        };
        if !group.post_login_scripts.is_empty() {
            return group.post_login_scripts.clone();
        }
        current = group.parent_id;
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::ExpectRule;
    use crate::models::{Connection, ConnectionGroup};

    fn make_rule(pattern: &str, response: &str) -> ExpectRule {
        ExpectRule::new(pattern, response)
    }

    #[test]
    fn connection_with_own_rules_does_not_inherit() {
        let group = ConnectionGroup::new("G".into());
        let mut conn = Connection::new_ssh("test".into(), "host".into(), 22);
        conn.group_id = Some(group.id);
        conn.automation.expect_rules = vec![make_rule("local", "response")];

        let mut group_with_rules = group;
        group_with_rules.expect_rules = vec![make_rule("group", "group-response")];

        let result = resolve_automation(&conn, &[group_with_rules]);
        assert_eq!(result.expect_rules.len(), 1);
        assert_eq!(result.expect_rules[0].pattern, "local");
    }

    #[test]
    fn empty_connection_inherits_from_direct_group() {
        let mut group = ConnectionGroup::new("G".into());
        group.expect_rules = vec![make_rule("password:", "secret{ENTER}")];

        let mut conn = Connection::new_ssh("test".into(), "host".into(), 22);
        conn.group_id = Some(group.id);

        let result = resolve_automation(&conn, &[group]);
        assert_eq!(result.expect_rules.len(), 1);
        assert_eq!(result.expect_rules[0].pattern, "password:");
    }

    #[test]
    fn inherits_from_grandparent_when_parent_empty() {
        let mut root = ConnectionGroup::new("Root".into());
        root.expect_rules = vec![make_rule("yes/no", "yes{ENTER}")];

        let child = ConnectionGroup::with_parent("Child".into(), root.id);

        let mut conn = Connection::new_ssh("test".into(), "host".into(), 22);
        conn.group_id = Some(child.id);

        let result = resolve_automation(&conn, &[root, child]);
        assert_eq!(result.expect_rules.len(), 1);
        assert_eq!(result.expect_rules[0].pattern, "yes/no");
    }

    #[test]
    fn cycle_detection_terminates() {
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();

        let mut group_a = ConnectionGroup::new("A".into());
        group_a.id = id_a;
        group_a.parent_id = Some(id_b);

        let mut group_b = ConnectionGroup::new("B".into());
        group_b.id = id_b;
        group_b.parent_id = Some(id_a);

        let mut conn = Connection::new_ssh("test".into(), "host".into(), 22);
        conn.group_id = Some(id_a);

        let result = resolve_automation(&conn, &[group_a, group_b]);
        assert!(result.expect_rules.is_empty());
    }

    #[test]
    fn post_login_scripts_inherited_independently() {
        let mut group = ConnectionGroup::new("G".into());
        group.post_login_scripts = vec!["echo hello".into()];
        // No expect_rules on group

        let mut conn = Connection::new_ssh("test".into(), "host".into(), 22);
        conn.group_id = Some(group.id);
        conn.automation.expect_rules = vec![make_rule("local", "resp")];
        // No post_login_scripts on connection

        let result = resolve_automation(&conn, &[group]);
        // Expect rules from connection (non-empty)
        assert_eq!(result.expect_rules.len(), 1);
        assert_eq!(result.expect_rules[0].pattern, "local");
        // Post-login scripts inherited from group
        assert_eq!(result.post_login_scripts, vec!["echo hello".to_string()]);
    }

    #[test]
    fn no_group_returns_empty() {
        let mut conn = Connection::new_ssh("test".into(), "host".into(), 22);
        conn.group_id = None;

        let result = resolve_automation(&conn, &[]);
        assert!(result.expect_rules.is_empty());
        assert!(result.post_login_scripts.is_empty());
    }
}
