use git2::{Error, ErrorClass, ErrorCode};
use std::path::Path;
use std::process::{Command, Stdio};

use super::agent::SshAgentManager;

/// Windows implementation of SSH agent management
pub struct WindowsSshAgentManager;

impl SshAgentManager for WindowsSshAgentManager {
    fn ensure_agent_ready() -> Result<(), Error> {
        // On Windows, rely on existing agent (Pageant or OpenSSH agent). If SSH_AUTH_SOCK is missing, try to start one.
        if std::env::var("SSH_AUTH_SOCK").is_err() {
            Self::start_agent_detached(None)?;
        }
        Ok(())
    }

    fn start_agent_detached(_socket_path: Option<&Path>) -> Result<(), Error> {
        // On Windows, best-effort spawn (ignore socket_path as it's not used)
        let _child = Command::new("ssh-agent")
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
            })?;
        Ok(())
    }
}

/// Convenience wrapper functions for platform-agnostic access
pub fn ensure_agent_ready() -> Result<(), Error> {
    WindowsSshAgentManager::ensure_agent_ready()
}
