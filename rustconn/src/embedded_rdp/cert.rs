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
fn remove_from_known_hosts2(known_hosts: &std::path::Path, host: &str, port: u16) {
    if !known_hosts.exists() {
        return;
    }
    let Ok(content) = std::fs::read_to_string(known_hosts) else {
        return;
    };

    let host_pattern = format!("{host} {port}");
    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| !line.contains(&host_pattern))
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
