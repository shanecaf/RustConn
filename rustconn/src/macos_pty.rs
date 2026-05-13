//! macOS-specific PTY spawn for VTE terminals.
//!
//! VTE's built-in `spawn_async` does not work on macOS (Homebrew build) —
//! the PTY is created but never connected to the child process output.
//! This module works around the issue by creating a native PTY via
//! `nix::pty::openpty()` and manually spawning the child process with
//! the slave fd as stdin/stdout/stderr, then handing the master fd to VTE.

use std::process::{Command, Stdio};

use gtk4::gio;
use gtk4::glib;
use vte4::prelude::*;
use vte4::{Pty, Terminal};

/// Spawns a command in a native macOS PTY and connects it to the VTE terminal.
///
/// Returns `Ok(child_pid)` on success, or an error string on failure.
pub fn spawn_native_pty(
    terminal: &Terminal,
    argv: &[&str],
    envv: &[&str],
    working_directory: Option<&str>,
) -> Result<u32, String> {
    if argv.is_empty() {
        return Err("argv is empty".to_string());
    }

    // 1. Create a PTY pair via openpty (safe wrapper around libc::openpty)
    let pty_pair = nix::pty::openpty(None, None).map_err(|e| format!("openpty failed: {e}"))?;

    let master_fd = pty_pair.master;
    let slave_fd = pty_pair.slave;

    // 2. Build the child process command
    let mut cmd = Command::new(argv[0]);
    if argv.len() > 1 {
        cmd.args(&argv[1..]);
    }

    // Set working directory
    if let Some(dir) = working_directory {
        cmd.current_dir(dir);
    }

    // Set environment: parse "KEY=VALUE" pairs
    cmd.env_clear();
    for env_str in envv {
        if let Some(eq_pos) = env_str.find('=') {
            let key = &env_str[..eq_pos];
            let value = &env_str[eq_pos + 1..];
            cmd.env(key, value);
        }
    }

    // If envv is empty, inherit parent environment
    if envv.is_empty() {
        for (key, value) in std::env::vars() {
            cmd.env(&key, &value);
        }
    }

    // Ensure TERM is set
    cmd.env("TERM", "xterm-256color");

    // 3. Connect slave fd as stdin/stdout/stderr for the child
    let stdin_fd = nix::unistd::dup(&slave_fd).map_err(|e| format!("dup stdin failed: {e}"))?;
    let stdout_fd = nix::unistd::dup(&slave_fd).map_err(|e| format!("dup stdout failed: {e}"))?;
    let stderr_fd = nix::unistd::dup(&slave_fd).map_err(|e| format!("dup stderr failed: {e}"))?;

    cmd.stdin(Stdio::from(stdin_fd));
    cmd.stdout(Stdio::from(stdout_fd));
    cmd.stderr(Stdio::from(stderr_fd));

    // 4. Spawn the child process
    // Note: Without pre_exec(setsid + TIOCSCTTY), the child does not become
    // a session leader. This means fzf-completion and job control (Ctrl-Z)
    // won't work, but basic shell operation and tab completion are functional.
    // Full PTY session setup requires unsafe pre_exec which is forbidden.
    let child = cmd.spawn().map_err(|e| format!("spawn failed: {e}"))?;
    let child_pid = child.id();

    tracing::info!(
        command = %argv[0],
        pid = child_pid,
        "macOS native PTY: child spawned"
    );

    // 5. Close slave fd in parent (child has its own copies)
    drop(slave_fd);

    // 6. Create VTE Pty from master fd and attach to terminal
    let vte_pty = Pty::foreign_sync(master_fd, gio::Cancellable::NONE)
        .map_err(|e| format!("Failed to create VTE Pty from fd: {e}"))?;

    terminal.set_pty(Some(&vte_pty));

    // 7. Watch for child exit and notify VTE
    let terminal_weak = terminal.downgrade();
    glib::child_watch_add_local(glib::Pid(child_pid as i32), move |_pid, status| {
        tracing::debug!(status = status, "macOS native PTY: child exited");
        if let Some(terminal) = terminal_weak.upgrade() {
            // Emit child-exited signal so VTE/RustConn handles cleanup
            terminal.emit_by_name::<()>("child-exited", &[&status]);
        }
    });

    Ok(child_pid)
}
