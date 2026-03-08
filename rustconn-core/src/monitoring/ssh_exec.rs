//! SSH command execution for monitoring
//!
//! Runs monitoring commands on remote hosts via `ssh` with `SSH_ASKPASS`
//! for password-authenticated connections. This uses a separate SSH process
//! (not the VTE terminal session) to avoid interfering with the user's
//! interactive shell.
//!
//! Password authentication uses the `SSH_ASKPASS` mechanism instead of
//! `sshpass`: a temporary script echoes the password from an environment
//! variable, and `SSH_ASKPASS_REQUIRE=force` tells OpenSSH to use it
//! even without a TTY. This eliminates the `sshpass` external dependency.

use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use tokio::process::Command;

/// Default timeout for SSH monitoring commands (seconds)
const SSH_EXEC_TIMEOUT_SECS: u64 = 10;

/// Environment variable name used to pass the password to the askpass script.
/// Intentionally obscure to reduce exposure in `/proc/PID/environ`.
const ASKPASS_ENV_VAR: &str = "_RC_MON_PW";

/// Creates a temporary `SSH_ASKPASS` helper script that echoes the password
/// from `ASKPASS_ENV_VAR`. The script is created with mode 0700 and lives
/// in the system temp directory.
///
/// Returns the path to the script on success.
fn create_askpass_script() -> Result<std::path::PathBuf, String> {
    use std::io::Write;

    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "rc-askpass-{}",
        uuid::Uuid::new_v4().as_hyphenated()
    ));

    let script = format!("#!/bin/sh\necho \"${ASKPASS_ENV_VAR}\"\n");

    let mut file = std::fs::File::create(&path)
        .map_err(|e| format!("Failed to create askpass script: {e}"))?;
    file.write_all(script.as_bytes())
        .map_err(|e| format!("Failed to write askpass script: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| format!("Failed to set askpass script permissions: {e}"))?;
    }

    Ok(path)
}

/// Builds an SSH exec closure for use with [`super::start_collector`].
///
/// The returned closure spawns `ssh` with the given host/port/user and
/// executes the provided shell command, returning stdout as a `String`.
///
/// When a password is provided, the `SSH_ASKPASS` mechanism is used:
/// a temporary script echoes the password from an environment variable,
/// and `SSH_ASKPASS_REQUIRE=force` tells OpenSSH to invoke it. This
/// replaces the previous `sshpass` dependency.
///
/// # Arguments
/// * `host` - Remote hostname or IP
/// * `port` - SSH port
/// * `username` - Optional SSH username
/// * `identity_file` - Optional path to SSH private key
/// * `password` - Optional password (as `SecretString`) for SSH_ASKPASS auth
/// * `jump_host` - Optional jump host chain for `-J` flag (e.g. `"user@bastion:22"`)
pub fn ssh_exec_factory(
    host: String,
    port: u16,
    username: Option<String>,
    identity_file: Option<String>,
    password: Option<SecretString>,
    jump_host: Option<String>,
) -> impl Fn(
    String,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
+ Send
+ 'static {
    // Create the askpass script once at factory creation time.
    // It is reused for every monitoring command invocation.
    let askpass_path = if password.is_some() {
        match create_askpass_script() {
            Ok(p) => Some(p),
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "Failed to create SSH_ASKPASS script; \
                     password auth will not work for monitoring"
                );
                None
            }
        }
    } else {
        None
    };

    move |command: String| {
        let host = host.clone();
        let username = username.clone();
        let identity_file = identity_file.clone();
        let password = password.clone();
        let jump_host = jump_host.clone();
        let askpass_path = askpass_path.clone();

        Box::pin(async move {
            let mut cmd = Command::new("ssh");

            if let (Some(pw), Some(ap)) = (&password, &askpass_path) {
                // SSH_ASKPASS mechanism: OpenSSH calls the script to get
                // the password. DISPLAY must be set (even empty) and
                // SSH_ASKPASS_REQUIRE=force skips the TTY check.
                cmd.env("SSH_ASKPASS", ap);
                cmd.env("SSH_ASKPASS_REQUIRE", "force");
                cmd.env(ASKPASS_ENV_VAR, pw.expose_secret());
                // Ensure DISPLAY is set so SSH considers ASKPASS
                if std::env::var("DISPLAY").is_err() {
                    cmd.env("DISPLAY", "");
                }
            } else if password.is_none() {
                // Batch mode only when NOT using password auth
                cmd.arg("-o").arg("BatchMode=yes");
            }

            // Accept new host keys but reject changed ones (OpenSSH 7.6+).
            // Using `accept-new` instead of `no` prevents MITM attacks on
            // hosts whose key has changed while still allowing first-time
            // connections without manual intervention.
            cmd.arg("-o").arg("StrictHostKeyChecking=accept-new");

            // In Flatpak, ~/.ssh is read-only — use writable known_hosts path
            if let Some(kh_path) = crate::flatpak::get_flatpak_known_hosts_path() {
                let kh_opt = format!("UserKnownHostsFile={}", kh_path.display());
                cmd.arg("-o").arg(kh_opt);
            }

            // Short connection timeout
            cmd.arg("-o").arg("ConnectTimeout=5");

            // Jump host chain for tunneled connections
            if let Some(ref jh) = jump_host {
                cmd.arg("-J").arg(jh);
            }

            if port != 22 {
                cmd.arg("-p").arg(port.to_string());
            }

            if let Some(ref key) = identity_file {
                cmd.arg("-i").arg(key);
            }

            let destination = if let Some(ref user) = username {
                format!("{user}@{host}")
            } else {
                host.clone()
            };
            cmd.arg(&destination);
            cmd.arg(&command);

            // Suppress stderr to avoid noise
            cmd.stderr(std::process::Stdio::piped());
            cmd.stdout(std::process::Stdio::piped());

            let timeout = Duration::from_secs(SSH_EXEC_TIMEOUT_SECS);

            match tokio::time::timeout(timeout, cmd.output()).await {
                Ok(Ok(output)) => {
                    if output.status.success() {
                        String::from_utf8(output.stdout)
                            .map_err(|e| format!("Invalid UTF-8 in SSH output: {e}"))
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        Err(format!(
                            "SSH command failed (exit {}): {}",
                            output.status,
                            stderr.trim()
                        ))
                    }
                }
                Ok(Err(e)) => Err(format!("Failed to spawn SSH process: {e}")),
                Err(_) => Err(format!(
                    "SSH monitoring command timed out after {SSH_EXEC_TIMEOUT_SECS}s"
                )),
            }
        })
    }
}
