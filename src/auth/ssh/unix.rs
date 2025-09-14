use git2::{Error, ErrorClass, ErrorCode};
use log::debug;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::agent::SshAgentManager;
use super::ssh_utils;
use crate::constants::SSH_AGENT_SOCKET_BASENAME;

/// Unix implementation of SSH agent management
pub struct UnixSshAgentManager;

impl SshAgentManager for UnixSshAgentManager {
    fn ensure_agent_ready() -> Result<(), Error> {
        let ssh_dir = home::home_dir()
            .map(|p| p.join(".ssh"))
            .unwrap_or_else(|| PathBuf::from(".ssh"));
        let socket_path = ssh_dir.join(SSH_AGENT_SOCKET_BASENAME);

        // Create ~/.ssh if needed
        if let Err(e) = std::fs::create_dir_all(&ssh_dir) {
            debug!("Failed to ensure ~/.ssh dir exists: {e}");
        }

        // If SSH_AUTH_SOCK already points to a working agent, keep it.
        if std::env::var("SSH_AUTH_SOCK")
            .ok()
            .and_then(|_| ssh_utils::agent_identities_count().ok())
            .is_some()
        {
            return Ok(());
        }

        // Remove stale socket if agent is dead
        if socket_path.exists() {
            let is_socket = std::fs::metadata(&socket_path)
                .map(|md| md.file_type().is_socket())
                .unwrap_or(false);
            if is_socket {
                let agent_alive = ssh_utils::agent_identities_count().is_ok();
                if !agent_alive {
                    debug!(
                        "Stale ssh-agent socket detected, removing: {:?}",
                        socket_path
                    );
                    let _ = std::fs::remove_file(&socket_path);
                }
            } else {
                let _ = std::fs::remove_file(&socket_path);
            }
        }

        // Otherwise, try to use our fixed socket path
        unsafe { std::env::set_var("SSH_AUTH_SOCK", &socket_path) };

        let alive = if let Ok(md) = std::fs::metadata(&socket_path)
            && md.file_type().is_socket()
        {
            // Probe agent via ssh-add -l
            ssh_utils::agent_identities_count().is_ok()
        } else {
            false
        };

        if !alive {
            // Try to start agent binding to our socket; if that fails or socket doesn't appear, fallback to parsing env
            if Self::start_agent_detached(Some(&socket_path)).is_err() || {
                // Wait briefly for socket readiness
                let mut ready = false;
                for _ in 0..20 {
                    if std::fs::metadata(&socket_path)
                        .map(|m| m.file_type().is_socket())
                        .unwrap_or(false)
                    {
                        ready = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                !ready
            } {
                // Fallback: spawn ssh-agent normally and parse SSH_AUTH_SOCK from its output
                if let Ok((sock, pid)) = start_agent_and_parse_env() {
                    unsafe { std::env::set_var("SSH_AUTH_SOCK", &sock) };
                    unsafe { std::env::set_var("SSH_AGENT_PID", &pid) };
                }
            }
        }

        Ok(())
    }

    fn start_agent_detached(socket_path: Option<&Path>) -> Result<(), Error> {
        // Try to start ssh-agent in background without making bgit its parent.
        // Prefer setsid/nohup if available (Unix).
        let mut cmd = if which::which("setsid").is_ok() {
            let mut c = Command::new("setsid");
            c.arg("ssh-agent");
            c
        } else if which::which("nohup").is_ok() {
            let mut c = Command::new("nohup");
            c.arg("ssh-agent");
            c
        } else {
            Command::new("ssh-agent")
        };

        if let Some(sock) = socket_path {
            // First try -a to bind to our fixed socket; if it fails, we'll fallback below
            cmd.arg("-a").arg(sock);
            // Keep in foreground and let setsid/nohup detach it; discard stdio
            cmd.arg("-D");
        } else {
            cmd.arg("-D");
        }

        let spawn_res = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                Error::new(
                    ErrorCode::Auth,
                    ErrorClass::Net,
                    format!("Failed to spawn ssh-agent: {e}"),
                )
            });

        if spawn_res.is_err() && socket_path.is_some() {
            // Fallback: try without -a (older agents)
            let mut fallback = if which::which("setsid").is_ok() {
                let mut c = Command::new("setsid");
                c.arg("ssh-agent");
                c
            } else if which::which("nohup").is_ok() {
                let mut c = Command::new("nohup");
                c.arg("ssh-agent");
                c
            } else {
                Command::new("ssh-agent")
            };
            fallback.arg("-D");
            let _ = fallback
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }

        Ok(())
    }
}

/// Start ssh-agent and parse environment variables from its output (Unix-specific)
fn start_agent_and_parse_env() -> Result<(String, String), Error> {
    let output = Command::new("ssh-agent").output().map_err(|e| {
        Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            format!("Failed to spawn ssh-agent: {e}"),
        )
    })?;
    if !output.status.success() {
        return Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            "ssh-agent failed to start",
        ));
    }
    let out = String::from_utf8_lossy(&output.stdout);
    let sock = out
        .lines()
        .find_map(|l| {
            l.split_once("SSH_AUTH_SOCK=")
                .and_then(|(_, r)| r.split(';').next())
        })
        .ok_or_else(|| {
            Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                "Failed to parse SSH_AUTH_SOCK",
            )
        })?;
    let pid = out
        .lines()
        .find_map(|l| {
            l.split_once("SSH_AGENT_PID=")
                .and_then(|(_, r)| r.split(';').next())
        })
        .ok_or_else(|| {
            Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                "Failed to parse SSH_AGENT_PID",
            )
        })?;
    Ok((sock.to_string(), pid.to_string()))
}

/// Convenience wrapper functions for platform-agnostic access
pub fn ensure_agent_ready() -> Result<(), Error> {
    UnixSshAgentManager::ensure_agent_ready()
}
