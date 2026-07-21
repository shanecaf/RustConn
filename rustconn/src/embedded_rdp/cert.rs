//! FreeRDP certificate store management
//!
//! Provides utilities for managing FreeRDP's TOFU certificate store
//! (`known_hosts2`). Used when the server certificate changes and the user
//! accepts the new one via a confirmation dialog.

/// Removes the stored FreeRDP certificate for a host from the known hosts store.
///
/// FreeRDP uses TOFU (Trust On First Use): the first time you connect it saves
/// the server certificate fingerprint. On subsequent connections, if the
/// certificate has changed, FreeRDP rejects the connection. Removing the stored
/// entry lets the next `/cert:tofu` connection accept and save the new
/// certificate — equivalent to removing a line from SSH `known_hosts`.
///
/// Cleans both FreeRDP 2.x (`server/<host>_<port>.pem`) and FreeRDP 3.x
/// (`known_hosts2`) certificate stores.
pub fn remove_known_certificate(host: &str, port: u16) {
    if let Some(config_dir) = dirs::config_dir() {
        // FreeRDP 2.x: individual PEM files per host
        let freerdp_dir = config_dir.join("freerdp").join("server");
        let pem_file = freerdp_dir.join(format!("{host}_{port}.pem"));
        if pem_file.exists() {
            tracing::debug!(
                ?pem_file,
                "Removing old FreeRDP certificate to accept new one"
            );
            let _ = std::fs::remove_file(&pem_file);
        }

        // FreeRDP 3.x: known_hosts2 file (one line per host)
        remove_from_known_hosts2(
            &config_dir.join("freerdp3").join("known_hosts2"),
            host,
            port,
        );
        // Some distros still use the freerdp/ path for FreeRDP 3.x
        remove_from_known_hosts2(&config_dir.join("freerdp").join("known_hosts2"), host, port);
    }
}

/// Removes a host entry from a FreeRDP `known_hosts2` file.
///
/// Uses exact field comparison (first two whitespace-separated fields must
/// match `host` and `port` exactly) to avoid false positives when one hostname
/// is a substring of another (e.g. `db.example.com` vs `my-db.example.com`).
fn remove_from_known_hosts2(known_hosts: &std::path::Path, host: &str, port: u16) {
    if !known_hosts.exists() {
        return;
    }
    let Ok(content) = std::fs::read_to_string(known_hosts) else {
        return;
    };

    let port_str = port.to_string();
    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| {
            let mut fields = line.split_whitespace();
            !(fields.next() == Some(host) && fields.next() == Some(port_str.as_str()))
        })
        .collect();

    if filtered.len() < content.lines().count() {
        tracing::debug!(
            ?known_hosts,
            %host,
            port,
            "Removing host entry from FreeRDP known_hosts2"
        );
        let _ = std::fs::write(known_hosts, filtered.join("\n") + "\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_matching_host_entry() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kh = dir.path().join("known_hosts2");
        std::fs::write(
            &kh,
            "server1.example.com 3389 abc123 CN=server1\nserver2.example.com 3389 def456 CN=server2\n",
        )
        .unwrap();

        remove_from_known_hosts2(&kh, "server1.example.com", 3389);

        let result = std::fs::read_to_string(&kh).unwrap();
        assert!(!result.contains("server1.example.com"));
        assert!(result.contains("server2.example.com"));
    }

    #[test]
    fn does_not_remove_substring_hostname() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kh = dir.path().join("known_hosts2");
        // "db.example.com" is a substring of "my-db.example.com" — must NOT be removed
        std::fs::write(
            &kh,
            "my-db.example.com 3389 aaa CN=my-db\ndb.example.com 3389 bbb CN=db\n",
        )
        .unwrap();

        remove_from_known_hosts2(&kh, "db.example.com", 3389);

        let result = std::fs::read_to_string(&kh).unwrap();
        assert!(
            result.contains("my-db.example.com"),
            "substring hostname should not be removed"
        );
        assert!(!result.contains("\ndb.example.com"));
    }

    #[test]
    fn does_not_remove_different_port() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kh = dir.path().join("known_hosts2");
        std::fs::write(
            &kh,
            "host.example.com 3389 abc CN=host\nhost.example.com 3390 def CN=host\n",
        )
        .unwrap();

        remove_from_known_hosts2(&kh, "host.example.com", 3389);

        let result = std::fs::read_to_string(&kh).unwrap();
        assert!(!result.contains("3389"));
        assert!(result.contains("host.example.com 3390"));
    }

    #[test]
    fn no_op_on_missing_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kh = dir.path().join("nonexistent");
        // Should not panic or error
        remove_from_known_hosts2(&kh, "host", 3389);
        assert!(!kh.exists());
    }

    #[test]
    fn no_op_when_host_not_found() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kh = dir.path().join("known_hosts2");
        let original = "other.host 3389 xyz CN=other\n";
        std::fs::write(&kh, original).unwrap();

        remove_from_known_hosts2(&kh, "missing.host", 3389);

        let result = std::fs::read_to_string(&kh).unwrap();
        assert_eq!(result, original);
    }
}
