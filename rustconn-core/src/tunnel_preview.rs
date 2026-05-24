//! SSH tunnel command preview builder.
//!
//! Generates a human-readable SSH command string for display purposes.
//! This module intentionally does NOT handle passwords or secrets —
//! only host, port, username, key path, and forwarding rules.

use crate::models::PortForward;

/// Parameters for generating an SSH tunnel command preview.
pub struct TunnelPreviewParams<'a> {
    /// Remote SSH host
    pub host: &'a str,
    /// Remote SSH port
    pub port: u16,
    /// SSH username (omitted from destination if `None`)
    pub username: Option<&'a str>,
    /// Port forwarding rules
    pub forwards: &'a [PortForward],
    /// ProxyJump host (`-J` argument)
    pub proxy_jump: Option<&'a str>,
    /// Path to identity file (`-i` argument)
    pub identity_file: Option<&'a str>,
}

/// Builds a human-readable SSH command string for preview purposes.
///
/// The output is suitable for display in a UI or copying to clipboard.
/// It does **not** include passwords or sensitive data.
///
/// # Examples
///
/// ```
/// use rustconn_core::tunnel_preview::{TunnelPreviewParams, build_tunnel_preview_command};
/// use rustconn_core::models::{PortForward, PortForwardDirection};
///
/// let params = TunnelPreviewParams {
///     host: "example.com",
///     port: 22,
///     username: Some("user"),
///     forwards: &[PortForward {
///         direction: PortForwardDirection::Local,
///         local_port: 8080,
///         remote_host: "localhost".to_string(),
///         remote_port: 80,
///     }],
///     proxy_jump: None,
///     identity_file: None,
/// };
///
/// let cmd = build_tunnel_preview_command(&params);
/// assert_eq!(cmd, "ssh -N -L 8080:localhost:80 user@example.com");
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PortForwardDirection;

    #[test]
    fn test_preview_basic() {
        let params = TunnelPreviewParams {
            host: "host",
            port: 2222,
            username: Some("user"),
            forwards: &[PortForward {
                direction: PortForwardDirection::Local,
                local_port: 8080,
                remote_host: "localhost".to_string(),
                remote_port: 80,
            }],
            proxy_jump: None,
            identity_file: None,
        };
        let cmd = build_tunnel_preview_command(&params);
        assert_eq!(cmd, "ssh -N -L 8080:localhost:80 -p 2222 user@host");
    }

    #[test]
    fn test_preview_with_proxy_jump() {
        let params = TunnelPreviewParams {
            host: "target.internal",
            port: 22,
            username: Some("admin"),
            forwards: &[PortForward {
                direction: PortForwardDirection::Local,
                local_port: 3306,
                remote_host: "db.internal".to_string(),
                remote_port: 3306,
            }],
            proxy_jump: Some("bastion@10.0.0.1"),
            identity_file: None,
        };
        let cmd = build_tunnel_preview_command(&params);
        assert!(cmd.contains("-J bastion@10.0.0.1"));
        assert_eq!(
            cmd,
            "ssh -N -L 3306:db.internal:3306 -J bastion@10.0.0.1 admin@target.internal"
        );
    }

    #[test]
    fn test_preview_dynamic() {
        let params = TunnelPreviewParams {
            host: "proxy.example.com",
            port: 22,
            username: Some("user"),
            forwards: &[PortForward {
                direction: PortForwardDirection::Dynamic,
                local_port: 1080,
                remote_host: String::new(),
                remote_port: 0,
            }],
            proxy_jump: None,
            identity_file: None,
        };
        let cmd = build_tunnel_preview_command(&params);
        assert_eq!(cmd, "ssh -N -D 1080 user@proxy.example.com");
    }

    #[test]
    fn test_preview_multiple_forwards() {
        let params = TunnelPreviewParams {
            host: "server",
            port: 22,
            username: Some("user"),
            forwards: &[
                PortForward {
                    direction: PortForwardDirection::Local,
                    local_port: 3306,
                    remote_host: "db".to_string(),
                    remote_port: 3306,
                },
                PortForward {
                    direction: PortForwardDirection::Remote,
                    local_port: 8080,
                    remote_host: "web".to_string(),
                    remote_port: 80,
                },
                PortForward {
                    direction: PortForwardDirection::Dynamic,
                    local_port: 1080,
                    remote_host: String::new(),
                    remote_port: 0,
                },
            ],
            proxy_jump: None,
            identity_file: None,
        };
        let cmd = build_tunnel_preview_command(&params);
        assert_eq!(
            cmd,
            "ssh -N -L 3306:db:3306 -R 8080:web:80 -D 1080 user@server"
        );
    }

    #[test]
    fn test_preview_no_forwards() {
        let params = TunnelPreviewParams {
            host: "server",
            port: 22,
            username: Some("user"),
            forwards: &[],
            proxy_jump: None,
            identity_file: None,
        };
        let cmd = build_tunnel_preview_command(&params);
        assert_eq!(cmd, "ssh -N user@server");
    }

    #[test]
    fn test_preview_no_username() {
        let params = TunnelPreviewParams {
            host: "192.168.1.100",
            port: 22,
            username: None,
            forwards: &[PortForward {
                direction: PortForwardDirection::Local,
                local_port: 5432,
                remote_host: "localhost".to_string(),
                remote_port: 5432,
            }],
            proxy_jump: None,
            identity_file: None,
        };
        let cmd = build_tunnel_preview_command(&params);
        assert_eq!(cmd, "ssh -N -L 5432:localhost:5432 192.168.1.100");
        assert!(!cmd.contains('@'));
    }

    #[test]
    fn test_preview_default_port() {
        let params = TunnelPreviewParams {
            host: "example.com",
            port: 22,
            username: Some("user"),
            forwards: &[PortForward {
                direction: PortForwardDirection::Local,
                local_port: 8080,
                remote_host: "localhost".to_string(),
                remote_port: 80,
            }],
            proxy_jump: None,
            identity_file: None,
        };
        let cmd = build_tunnel_preview_command(&params);
        // Port 22 should NOT produce a -p argument
        assert!(!cmd.contains("-p"));
        assert_eq!(cmd, "ssh -N -L 8080:localhost:80 user@example.com");
    }

    #[test]
    fn test_preview_identity_file() {
        let params = TunnelPreviewParams {
            host: "server",
            port: 22,
            username: Some("deploy"),
            forwards: &[PortForward {
                direction: PortForwardDirection::Local,
                local_port: 9090,
                remote_host: "app".to_string(),
                remote_port: 9090,
            }],
            proxy_jump: None,
            identity_file: Some("/home/user/.ssh/id_ed25519"),
        };
        let cmd = build_tunnel_preview_command(&params);
        assert!(cmd.contains("-i /home/user/.ssh/id_ed25519"));
        assert_eq!(
            cmd,
            "ssh -N -L 9090:app:9090 -i /home/user/.ssh/id_ed25519 deploy@server"
        );
    }
}
