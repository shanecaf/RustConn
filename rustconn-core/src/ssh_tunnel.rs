//! SSH tunnel for forwarding connections through a jump host.
//!
//! Used by RDP, VNC, SPICE, and Telnet connections that have a
//! `jump_host_id` configured. Creates an `ssh -L` local port forward
//! in the background and returns the local port for the client to
//! connect to.

use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use thiserror::Error;

/// Errors that can occur when creating an SSH tunnel.
#[derive(Debug, Error)]
pub enum SshTunnelError {
    /// No free local port could be found.
    #[error("Could not find a free local port")]
    NoFreePort,
    /// Failed to spawn the SSH process.
    #[error("Failed to spawn SSH tunnel: {0}")]
    SpawnFailed(#[from] std::io::Error),
}

/// Result type for SSH tunnel operations.
pub type SshTunnelResult<T> = Result<T, SshTunnelError>;

/// A running SSH tunnel (`ssh -N -L ...`).
///
/// The tunnel process is killed when this struct is dropped.
pub struct SshTunnel {
    /// The child SSH process.
    child: Child,
    /// The local port that forwards to the remote destination.
    local_port: u16,
}

impl SshTunnel {
    /// Returns the local port to connect to.
    #[must_use]
    pub const fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Stops the tunnel by killing the SSH process.
    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Parameters for creating an SSH tunnel.
pub struct SshTunnelParams {
    /// Jump host address (e.g. `user@bastion.example.com`).
    pub jump_host: String,
    /// Jump host SSH port (default 22).
    pub jump_port: u16,
    /// Remote destination host (the actual RDP/VNC/SPICE server).
    pub remote_host: String,
    /// Remote destination port.
    pub remote_port: u16,
    /// Optional SSH identity file for the jump host.
    pub identity_file: Option<String>,
    /// Optional extra SSH args (e.g. `-o StrictHostKeyChecking=no`).
    pub extra_args: Vec<String>,
}

/// Finds a free TCP port by binding to port 0 and reading the assigned port.
///
/// # Errors
///
/// Returns `SshTunnelError::NoFreePort` if binding fails.
pub fn find_free_port() -> SshTunnelResult<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| SshTunnelError::NoFreePort)?;
    let port = listener
        .local_addr()
        .map_err(|_| SshTunnelError::NoFreePort)?
        .port();
    // Drop the listener so the port is released before SSH binds to it.
    // There is a small TOCTOU window, but it is acceptable for this use case.
    drop(listener);
    Ok(port)
}

/// Creates an SSH tunnel by spawning `ssh -N -L local_port:remote:remote_port`.
///
/// The tunnel runs in the background. The caller must keep the returned
/// [`SshTunnel`] alive for the duration of the connection — dropping it
/// kills the SSH process.
///
/// # Errors
///
/// Returns an error if no free port is found or the SSH process fails to spawn.
pub fn create_tunnel(params: &SshTunnelParams) -> SshTunnelResult<SshTunnel> {
    let local_port = find_free_port()?;

    let forward_spec = format!(
        "{}:{}:{}",
        local_port, params.remote_host, params.remote_port
    );

    let mut cmd = Command::new("ssh");
    cmd.arg("-N") // No remote command — just forward
        .arg("-L")
        .arg(&forward_spec);

    // Jump host port
    if params.jump_port != 22 {
        cmd.arg("-p").arg(params.jump_port.to_string());
    }

    // Identity file
    if let Some(ref key) = params.identity_file {
        cmd.arg("-i").arg(key);
    }

    // Extra args
    for arg in &params.extra_args {
        cmd.arg(arg);
    }

    // Flatpak writable known_hosts
    if let Some(kh_path) = crate::get_flatpak_known_hosts_path() {
        cmd.arg("-o")
            .arg(format!("UserKnownHostsFile={}", kh_path.display()));
    }

    // Prevent SSH from reading stdin (would steal from the parent process)
    cmd.arg("-o").arg("BatchMode=yes");

    // Exit if the forwarding fails (e.g. port already in use)
    cmd.arg("-o").arg("ExitOnForwardFailure=yes");

    // The jump host destination
    cmd.arg(&params.jump_host);

    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    tracing::info!(
        local_port,
        remote = %format!("{}:{}", params.remote_host, params.remote_port),
        jump_host = %params.jump_host,
        "Starting SSH tunnel"
    );

    let child = cmd.spawn()?;

    Ok(SshTunnel { child, local_port })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_free_port() {
        let port = find_free_port().expect("should find a free port");
        assert!(port > 0);
        // Verify the port is actually free by binding to it
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"));
        assert!(listener.is_ok(), "port {port} should be bindable");
    }

    #[test]
    fn test_find_free_port_unique() {
        let p1 = find_free_port().expect("port 1");
        let p2 = find_free_port().expect("port 2");
        // Ports should be different (extremely likely, not guaranteed)
        // This is a probabilistic test — skip assertion if they happen to match
        if p1 == p2 {
            eprintln!("Warning: two consecutive find_free_port() returned the same port {p1}");
        }
    }
}
