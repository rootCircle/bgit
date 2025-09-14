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

        debug!("Starting SSH agent management with preferred persistent strategy");
        debug!("Target persistent socket path: {:?}", socket_path);

        // STRATEGY: Prefer persistent agent -> retry creating persistent -> check env -> create with parse_env

        // STEP 1: Try to use existing persistent agent (bgit's own agent)
        if Self::try_persistent_agent(&socket_path)? {
            debug!("Using existing persistent agent");
            Self::finalize_agent_setup();
            return Ok(());
        }
        debug!("No working persistent agent found");

        // STEP 2: Retry creating persistent agent
        debug!("Attempting to create persistent agent");
        if Self::create_persistent_agent(&socket_path)? {
            debug!("Successfully created persistent agent");
            Self::finalize_agent_setup();
            return Ok(());
        }
        debug!("Failed to create persistent agent");

        // STEP 3: Check if environment has working SSH_AUTH_SOCK
        let env_sock = std::env::var("SSH_AUTH_SOCK").ok();

        if let Some(sock) = &env_sock {
            debug!("Environment has SSH_AUTH_SOCK: {:?}", sock);
            if ssh_utils::agent_identities_count_with_auth(Some(sock)).is_ok() {
                debug!("Using existing environment SSH agent");
                Self::finalize_agent_setup();
                return Ok(());
            }
            debug!("Environment SSH agent not working");
        } else {
            debug!("Environment missing SSH_AUTH_SOCK");
        }

        // STEP 4: Create and set using start_agent_and_parse_env
        debug!("Creating new agent using start_agent_and_parse_env");
        match start_agent_and_parse_env() {
            Ok(sock) => {
                debug!("Successfully created agent via parse_env: sock={}", sock);
                unsafe {
                    std::env::set_var("SSH_AUTH_SOCK", &sock);
                }
                Self::finalize_agent_setup();
                Ok(())
            }
            Err(e) => {
                debug!("Failed to create agent via parse_env: {}", e);
                // Final fallback: try any working agent in environment
                if env_sock.is_some()
                    && ssh_utils::agent_identities_count_with_auth(env_sock.as_deref()).is_ok()
                {
                    debug!("Falling back to environment agent without PID requirement");
                    Self::finalize_agent_setup();
                    return Ok(());
                }
                Err(e)
            }
        }
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

impl UnixSshAgentManager {
    /// Try to use an existing persistent agent (bgit's own agent with fixed socket)
    fn try_persistent_agent(socket_path: &Path) -> Result<bool, Error> {
        debug!(
            "Checking for existing persistent agent at {:?}",
            socket_path
        );

        // Check if bgit agent state exists and is valid
        if let Some(state) = ssh_utils::load_bgit_agent_state() {
            let socket_str = state.socket_path.to_string_lossy();
            debug!("Found bgit agent state - socket: {:?}", socket_str);

            // Verify the agent is actually working
            if ssh_utils::agent_identities_count_with_auth(Some(&socket_str)).is_ok() {
                debug!("Persistent agent is working");
                return Ok(true);
            } else {
                debug!("Persistent agent not working, cleaning up stale state");
                ssh_utils::cleanup_bgit_agent_state();
                // Brief pause after cleanup to avoid race conditions
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        // Also check if socket exists and is working (even without saved state)
        if socket_path.exists() {
            let is_socket = std::fs::metadata(socket_path)
                .map(|md| md.file_type().is_socket())
                .unwrap_or(false);

            if is_socket {
                let socket_str = socket_path.to_string_lossy();
                if ssh_utils::agent_identities_count_with_auth(Some(&socket_str)).is_ok() {
                    debug!("Found working socket without saved state, adopting it");
                    return Ok(true);
                }

                debug!("Socket exists but agent not working, removing stale socket");
                if let Err(e) = std::fs::remove_file(socket_path) {
                    debug!("Failed to remove stale socket {:?}: {}", socket_path, e);
                }
            } else {
                debug!("Non-socket file at agent path, removing: {:?}", socket_path);
                if let Err(e) = std::fs::remove_file(socket_path) {
                    debug!("Failed to remove non-socket file {:?}: {}", socket_path, e);
                }
            }
        }

        Ok(false)
    }

    /// Create a new persistent agent bound to the fixed socket
    pub fn create_persistent_agent(socket_path: &Path) -> Result<bool, Error> {
        debug!("Creating persistent agent at {:?}", socket_path);

        // Try to start agent with fixed socket binding
        if Self::start_agent_detached(Some(socket_path)).is_err() {
            debug!("Failed to start detached agent with socket binding");
            return Ok(false);
        }

        // Wait for socket to appear and become ready
        let mut socket_ready = false;
        for attempt in 0..30 {
            // Increased attempts for better reliability
            if std::fs::metadata(socket_path)
                .map(|m| m.file_type().is_socket())
                .unwrap_or(false)
            {
                let socket_str = socket_path.to_string_lossy();
                if ssh_utils::agent_identities_count_with_auth(Some(&socket_str)).is_ok() {
                    debug!(
                        "Persistent agent socket ready after {} attempts",
                        attempt + 1
                    );
                    socket_ready = true;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if !socket_ready {
            debug!("Persistent agent socket not ready after waiting");
            return Ok(false);
        }

        Ok(true)
    }

    /// Finalize agent setup by setting global environment for libgit2
    fn finalize_agent_setup() {
        let effective_socket = ssh_utils::get_effective_ssh_auth();
        ssh_utils::set_global_ssh_env_for_libgit2(effective_socket.as_deref());
        debug!("Finalized SSH agent setup - socket: {:?}", effective_socket);
    }
}

/// Start ssh-agent and parse socket from its output (Unix-specific)
fn start_agent_and_parse_env() -> Result<String, Error> {
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
    Ok(sock.to_string())
}

/// Convenience wrapper functions for platform-agnostic access
pub fn ensure_agent_ready() -> Result<(), Error> {
    UnixSshAgentManager::ensure_agent_ready()
}
