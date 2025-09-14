use dialoguer::{Confirm, theme::ColorfulTheme};
use git2::{Error, ErrorClass, ErrorCode};
use log::debug;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::global::BGitGlobalConfig;
use crate::constants::SSH_AGENT_SOCKET_BASENAME;
use std::os::unix::fs::FileTypeExt;

/// Get the count of identities in SSH agent with specific auth environment
pub fn agent_identities_count_with_auth(
    socket_path: Option<&str>,
    agent_pid: Option<&str>,
) -> Result<usize, Error> {
    let mut cmd = Command::new("ssh-add");
    cmd.arg("-l");

    if let Some(socket) = socket_path {
        cmd.env("SSH_AUTH_SOCK", socket);
    }

    if let Some(pid) = agent_pid {
        cmd.env("SSH_AGENT_PID", pid);
    }

    let output = cmd.output().map_err(|e| {
        Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            format!("Failed to run ssh-add -l: {e}"),
        )
    })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Filter out informational lines and blanks; count actual keys
        let count = stdout
            .lines()
            .filter(|l| !l.contains("The agent has no identities") && !l.trim().is_empty())
            .count();
        Ok(count)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Could not open a connection to your authentication agent") {
            Err(Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                "ssh-agent not reachable",
            ))
        } else {
            Err(Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                format!("ssh-add -l failed: {stderr}"),
            ))
        }
    }
}

/// Interactively add a key to SSH agent with explicit auth environment
pub fn add_key_interactive_with_auth(
    key_path: &Path,
    key_name: &str,
    socket_path: Option<&str>,
    agent_pid: Option<&str>,
) -> Result<bool, Error> {
    debug!("Trying interactive ssh-add for key: {key_name}");

    // Ask user if they want to add this key interactively
    let should_add = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Add SSH key '{key_name}' to ssh-agent? (you may be prompted for passphrase)"
        ))
        .default(true)
        .interact()
        .map_err(|e| {
            Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                format!("Failed to get user confirmation: {e}"),
            )
        })?;

    if !should_add {
        debug!("User chose not to add key: {key_name}");
        return Ok(false);
    }

    println!("Adding SSH key: {key_name}");
    println!("If the key is passphrase-protected, you will be prompted to enter it.");

    let mut cmd = Command::new("ssh-add");
    cmd.arg(key_path);

    if let Some(socket) = socket_path {
        cmd.env("SSH_AUTH_SOCK", socket);
    }

    if let Some(pid) = agent_pid {
        cmd.env("SSH_AGENT_PID", pid);
    }

    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| {
            Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                format!("Failed to spawn ssh-add: {e}"),
            )
        })?;

    if status.success() {
        debug!("Successfully added key: {key_name}");
        println!("✓ SSH key '{key_name}' added successfully!");
        Ok(true)
    } else {
        debug!("Interactive ssh-add failed for key: {key_name}");
        println!("✗ Failed to add SSH key '{key_name}'");
        Ok(false)
    }
}

/// Try SSH key files directly without agent (platform-agnostic)
pub fn try_ssh_key_files_directly(username: &str) -> Result<git2::Cred, Error> {
    debug!("Trying SSH key files directly for user: {username}");

    let ssh_dir = home::home_dir()
        .map(|p| p.join(".ssh"))
        .unwrap_or_else(|| PathBuf::from(".ssh"));
    let key_files = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];

    for key_name in &key_files {
        let private_key_path = ssh_dir.join(key_name);
        let public_key_path = ssh_dir.join(format!("{key_name}.pub"));

        if private_key_path.exists() && public_key_path.exists() {
            debug!("Trying SSH key pair: {key_name} / {key_name}.pub");

            match git2::Cred::ssh_key(
                username,
                Some(&public_key_path),
                &private_key_path,
                None, // No passphrase for now
            ) {
                Ok(cred) => {
                    debug!("SSH key authentication succeeded with {key_name}");
                    return Ok(cred);
                }
                Err(e) => {
                    debug!("SSH key authentication failed with {key_name}: {e}");
                }
            }
        }
    }

    Err(Error::new(
        ErrorCode::Auth,
        ErrorClass::Net,
        "No valid SSH key pairs found or all failed authentication",
    ))
}

/// Add all available SSH keys to the agent with explicit auth environment
pub fn add_all_ssh_keys_with_auth(
    cfg: &BGitGlobalConfig,
    socket_path: Option<&str>,
    agent_pid: Option<&str>,
) -> Result<Option<PathBuf>, Error> {
    debug!("Adding all SSH keys from .ssh folder to ssh-agent");

    let ssh_dir = home::home_dir()
        .map(|p| p.join(".ssh"))
        .unwrap_or_else(|| PathBuf::from(".ssh"));

    if !ssh_dir.exists() {
        debug!("SSH directory {ssh_dir:?} does not exist");
        return Ok(None);
    }

    let key_files = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];
    let mut added_count = 0;
    let mut first_added: Option<PathBuf> = None;

    let mut candidates: Vec<PathBuf> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Some(configured_key) = cfg.get_ssh_key_file()
        && seen.insert(configured_key.clone())
    {
        candidates.push(configured_key);
    }
    for name in &key_files {
        let path = ssh_dir.join(name);
        if seen.insert(path.clone()) {
            candidates.push(path);
        }
    }

    drop(seen);

    for key_path in candidates {
        let display_name = key_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("ssh_key");

        if key_path.exists() {
            debug!("Found SSH key: {key_path:?}");
            // Skip if it's not a regular file or if a corresponding .pub is being considered
            if let Ok(md) = std::fs::metadata(&key_path)
                && !md.is_file()
            {
                continue;
            }
            // Also skip accidental public key files
            if key_path.extension().and_then(|s| s.to_str()) == Some("pub") {
                continue;
            }

            // First try a quick non-interactive add (for keys without passphrase)
            let mut cmd = Command::new("ssh-add");
            cmd.arg(&key_path);

            if let Some(socket) = socket_path {
                cmd.env("SSH_AUTH_SOCK", socket);
            }

            if let Some(pid) = agent_pid {
                cmd.env("SSH_AGENT_PID", pid);
            }

            let quick_result = cmd
                .stdin(Stdio::null()) // No input for quick try
                .stdout(Stdio::null()) // Suppress output for quick try
                .stderr(Stdio::piped()) // Capture errors to check if passphrase is needed
                .output();

            match quick_result {
                Ok(output) if output.status.success() => {
                    debug!("Successfully added key without interaction: {display_name}");
                    added_count += 1;
                    if first_added.is_none() {
                        first_added = Some(key_path.clone());
                    }
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    debug!("Quick add failed for {display_name}: {stderr}");

                    debug!("Key {display_name} appears to need passphrase, trying interactive add");

                    match add_key_interactive_with_auth(
                        &key_path,
                        display_name,
                        socket_path,
                        agent_pid,
                    ) {
                        Ok(true) => {
                            debug!("Successfully added key interactively: {display_name}");
                            added_count += 1;
                            if first_added.is_none() {
                                first_added = Some(key_path.clone());
                            }
                        }
                        Ok(false) => {
                            debug!("User skipped key: {display_name}");
                        }
                        Err(e) => {
                            debug!("Interactive add failed for {display_name}: {e}");
                        }
                    }
                }
                Err(e) => {
                    debug!("Error running ssh-add for {display_name}: {e}");
                }
            }
        } else {
            debug!("SSH key not found: {key_path:?}");
        }
    }

    debug!("Added {added_count} SSH keys to ssh-agent");

    if added_count == 0 {
        debug!("No SSH keys were added");
        println!("No SSH keys were added to ssh-agent.");
        println!("You may need to generate SSH keys or check your ~/.ssh directory.");
    } else {
        println!("Successfully added {added_count} SSH key(s) to ssh-agent.");
    }

    Ok(first_added)
}

/// SSH agent state management helpers
#[derive(Debug, Clone)]
pub struct BgitSshAgentState {
    pub socket_path: PathBuf,
    pub pid: Option<String>,
}

/// Get the expected paths for bgit SSH agent files
fn get_bgit_agent_paths() -> (PathBuf, PathBuf) {
    let ssh_dir = home::home_dir()
        .map(|p| p.join(".ssh"))
        .unwrap_or_else(|| PathBuf::from(".ssh"));
    let socket_path = ssh_dir.join(SSH_AGENT_SOCKET_BASENAME);
    let pid_file_path = ssh_dir.join(format!("{}.pid", SSH_AGENT_SOCKET_BASENAME));
    (socket_path, pid_file_path)
}

/// Load bgit SSH agent state from files if both socket and PID exist
pub fn load_bgit_agent_state() -> Option<BgitSshAgentState> {
    let (socket_path, pid_file_path) = get_bgit_agent_paths();

    // Both socket and PID file must exist to be considered valid
    if !socket_path.exists() || !pid_file_path.exists() {
        debug!(
            "Bgit agent state incomplete - socket exists: {}, pid file exists: {}",
            socket_path.exists(),
            pid_file_path.exists()
        );
        return None;
    }

    // On Unix, ensure the socket path is actually a Unix domain socket
    #[cfg(unix)]
    {
        match std::fs::metadata(&socket_path) {
            Ok(md) => {
                if !md.file_type().is_socket() {
                    debug!(
                        "Bgit agent socket path exists but is not a socket: {:?}",
                        socket_path
                    );
                    return None;
                }
            }
            Err(e) => {
                debug!("Failed to stat socket path {:?}: {}", socket_path, e);
                return None;
            }
        }
    }

    // Read PID from file
    let pid = match std::fs::read_to_string(&pid_file_path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                debug!("PID file is empty: {:?}", pid_file_path);
                return None;
            }
            Some(trimmed.to_string())
        }
        Err(e) => {
            debug!("Failed to read PID file {:?}: {}", pid_file_path, e);
            return None;
        }
    };

    debug!(
        "Loaded bgit agent state - socket: {:?}, pid: {:?}",
        socket_path, pid
    );
    Some(BgitSshAgentState { socket_path, pid })
}

/// Save bgit SSH agent state to files
pub fn save_bgit_agent_state(socket_path: &Path, pid: Option<&str>) -> Result<(), Error> {
    let (_, pid_file_path) = get_bgit_agent_paths();

    if let Some(pid_str) = pid {
        if let Err(e) = std::fs::write(&pid_file_path, pid_str) {
            debug!("Failed to write PID file {:?}: {}", pid_file_path, e);
            return Err(Error::new(
                git2::ErrorCode::GenericError,
                git2::ErrorClass::Os,
                format!("Failed to save agent PID: {}", e),
            ));
        }
        debug!(
            "Saved bgit agent state - socket: {:?}, pid: {}",
            socket_path, pid_str
        );
    } else {
        debug!("No PID provided, not saving state");
    }

    Ok(())
}

/// Clean up bgit SSH agent state files
pub fn cleanup_bgit_agent_state() {
    let (socket_path, pid_file_path) = get_bgit_agent_paths();

    if socket_path.exists() {
        if let Err(e) = std::fs::remove_file(&socket_path) {
            debug!("Failed to remove socket file {:?}: {}", socket_path, e);
        } else {
            debug!("Cleaned up socket file: {:?}", socket_path);
        }
    }

    if pid_file_path.exists() {
        if let Err(e) = std::fs::remove_file(&pid_file_path) {
            debug!("Failed to remove PID file {:?}: {}", pid_file_path, e);
        } else {
            debug!("Cleaned up PID file: {:?}", pid_file_path);
        }
    }
}

/// Direct agent verification without recursion - used internally to avoid infinite loops
fn verify_agent_socket_direct(socket_path: &str, agent_pid: Option<&str>) -> bool {
    let mut cmd = Command::new("ssh-add");
    cmd.arg("-l").env("SSH_AUTH_SOCK", socket_path);

    if let Some(pid) = agent_pid {
        cmd.env("SSH_AGENT_PID", pid);
    }

    cmd.output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Set global SSH environment variables for libgit2 compatibility
/// This is needed because libgit2's Cred::ssh_key_from_agent() uses global environment
/// WARNING: This modifies global process state - use carefully!
pub fn set_global_ssh_env_for_libgit2(socket_path: Option<&str>, agent_pid: Option<&str>) {
    if let Some(socket) = socket_path {
        debug!("Setting global SSH_AUTH_SOCK for libgit2: {}", socket);
        unsafe { std::env::set_var("SSH_AUTH_SOCK", socket) };
    } else {
        debug!("No SSH_AUTH_SOCK provided - libgit2 will use existing environment");
    }

    if let Some(pid) = agent_pid {
        debug!("Setting global SSH_AGENT_PID for libgit2: {}", pid);
        unsafe { std::env::set_var("SSH_AGENT_PID", pid) };
    } else {
        debug!("No SSH_AGENT_PID provided - libgit2 will use existing environment");
    }
}

/// Get the current effective SSH auth configuration
/// Returns (socket_path, pid) - uses bgit state if available, otherwise current environment
pub fn get_effective_ssh_auth() -> (Option<String>, Option<String>) {
    // First try to load bgit agent state
    if let Some(state) = load_bgit_agent_state() {
        // Verify the socket is actually working - using direct verification to avoid recursion
        let socket_str = state.socket_path.to_string_lossy();
        if verify_agent_socket_direct(&socket_str, state.pid.as_deref()) {
            debug!("Using bgit agent state: {:?}", state.socket_path);
            return (Some(socket_str.to_string()), state.pid);
        } else {
            debug!("Bgit agent socket not working, cleaning up stale state");
            cleanup_bgit_agent_state();
        }
    }

    // Fallback to current environment
    let current_sock = std::env::var("SSH_AUTH_SOCK").ok();
    let current_pid = std::env::var("SSH_AGENT_PID").ok();

    // Validate environment-provided socket on Unix (must be a socket and working)
    if let Some(ref sock) = current_sock {
        #[cfg(unix)]
        {
            let path = std::path::Path::new(sock);
            let is_socket = std::fs::metadata(path)
                .map(|m| m.file_type().is_socket())
                .unwrap_or(false);
            if !is_socket {
                debug!(
                    "Environment SSH_AUTH_SOCK is not a socket or missing: {:?}",
                    sock
                );
                return (None, None);
            }
        }

        if verify_agent_socket_direct(sock, current_pid.as_deref()) {
            debug!(
                "Using current environment auth - socket: {:?}, pid: {:?}",
                current_sock, current_pid
            );
            return (current_sock, current_pid);
        } else {
            debug!(
                "Environment SSH agent not working for socket {:?}, ignoring",
                sock
            );
            return (None, None);
        }
    }

    debug!("No SSH agent environment available");
    (None, None)
}
