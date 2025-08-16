use dialoguer::{Confirm, theme::ColorfulTheme};
use git2::{Cred, CredentialType, Error, ErrorClass, ErrorCode};
use log::debug;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::auth::auth_utils::prompt_persist_preferred_auth;
use crate::auth::ssh_utils::add_key_interactive;
use crate::config::global::{BGitGlobalConfig, PreferredAuth};
use crate::constants::MAX_AUTH_ATTEMPTS;

#[cfg(unix)]
use crate::constants::SSH_AGENT_SOCKET_BASENAME;
#[cfg(unix)]
use std::path::Path;

pub fn ssh_authenticate_git(
    url: &str,
    username_from_url: Option<&str>,
    allowed_types: CredentialType,
    attempt_count: usize,
    cfg: &BGitGlobalConfig,
) -> Result<Cred, Error> {
    debug!("Git authentication attempt #{attempt_count} for URL: {url}");
    debug!("Username from URL: {username_from_url:?}");
    debug!("Allowed credential types: {allowed_types:?}");

    // Prevent infinite loops
    if attempt_count > MAX_AUTH_ATTEMPTS {
        debug!(
            "Too many authentication attempts ({attempt_count}), failing to prevent infinite loop"
        );
        return Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            "Too many authentication attempts",
        ));
    }

    if allowed_types.contains(CredentialType::SSH_KEY) {
        if let Some(username) = username_from_url {
            debug!("SSH key authentication is allowed, trying SSH agent");

            // Before auth attempt 1, ensure an agent is available and has at least 1 identity.
            ensure_agent_ready()?;

            // If the agent is up but has no identities, try to add common keys once.
            let mut added_key_path: Option<PathBuf> = None;
            if agent_identities_count().unwrap_or(0) == 0 && attempt_count <= MAX_AUTH_ATTEMPTS {
                debug!("ssh-agent has no identities, attempting to add keys from ~/.ssh");
                if let Ok(first_added) = add_all_ssh_keys(cfg) {
                    added_key_path = first_added;
                }
            }

            if let Ok(cred) = try_ssh_agent_auth(username) {
                // Offer to set preferred auth to SSH
                prompt_persist_preferred_auth(cfg, PreferredAuth::Ssh);
                if let Some(added) = added_key_path.as_deref() {
                    // Persist only if it differs from currently configured key
                    if cfg.get_ssh_key_file().as_deref() != Some(added) {
                        prompt_persist_key_file(cfg, added);
                    }
                }
                return Ok(cred);
            }
        } else {
            debug!("No username provided for SSH authentication");
        }
    }

    debug!("All authentication methods failed for attempt {attempt_count}");
    Err(Error::new(
        ErrorCode::Auth,
        ErrorClass::Net,
        format!("Authentication failed - attempt {attempt_count}"),
    ))
}

fn try_ssh_agent_auth(username: &str) -> Result<Cred, Error> {
    debug!("Attempting SSH agent authentication for user: {username}");
    ensure_agent_ready()?;

    match Cred::ssh_key_from_agent(username) {
        Ok(cred) => {
            debug!("SSH agent authentication succeeded");

            Ok(cred)
        }
        Err(e) => {
            debug!("SSH agent authentication failed: {e}");

            // Fallback to trying SSH key files directly
            debug!("Falling back to direct SSH key file authentication");
            try_ssh_key_files_directly(username)
        }
    }
}

// Ensure an ssh-agent is available and exported in the environment.
// On Unix (Linux/macOS), prefer a persistent socket at $HOME/.ssh/ssh-agent.sock to avoid duplicates.
fn ensure_agent_ready() -> Result<(), Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;

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
            .and_then(|_| agent_identities_count().ok())
            .is_some()
        {
            return Ok(());
        }

        // Otherwise, try to use our fixed socket path
        unsafe { std::env::set_var("SSH_AUTH_SOCK", &socket_path) };

        let alive = if let Ok(md) = std::fs::metadata(&socket_path)
            && md.file_type().is_socket()
        {
            // Probe agent via ssh-add -l
            agent_identities_count().is_ok()
        } else {
            false
        };

        // Remove stale non-socket file
        if let Ok(md) = std::fs::metadata(&socket_path)
            && !md.file_type().is_socket()
        {
            let _ = std::fs::remove_file(&socket_path);
        }

        if !alive {
            // Try to start agent binding to our socket; if that fails or socket doesn't appear, fallback to parsing env
            if start_agent_detached(Some(&socket_path)).is_err() || {
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
                #[cfg(unix)]
                {
                    if let Ok((sock, pid)) = start_agent_and_parse_env() {
                        unsafe { std::env::set_var("SSH_AUTH_SOCK", &sock) };
                        unsafe { std::env::set_var("SSH_AGENT_PID", &pid) };
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(not(unix))]
    {
        // On Windows, rely on existing agent (Pageant or OpenSSH agent). If SSH_AUTH_SOCK is missing, try to start one.
        if std::env::var("SSH_AUTH_SOCK").is_err() {
            start_agent_detached()?;
        }
        Ok(())
    }
}

#[cfg(unix)]
fn start_agent_detached(socket: Option<&Path>) -> Result<(), Error> {
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

    if let Some(sock) = socket {
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

    if spawn_res.is_err() && socket.is_some() {
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

#[cfg(not(unix))]
fn start_agent_detached() -> Result<(), Error> {
    // On Windows, best-effort spawn
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

fn add_all_ssh_keys(cfg: &BGitGlobalConfig) -> Result<Option<PathBuf>, Error> {
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
    if let Some(configured_key) = cfg.get_ssh_key_file() {
        candidates.push(configured_key);
    }
    for name in &key_files {
        candidates.push(ssh_dir.join(name));
    }

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
            let quick_result = Command::new("ssh-add")
                .arg(&key_path)
                .env(
                    "SSH_AUTH_SOCK",
                    std::env::var("SSH_AUTH_SOCK").unwrap_or_default(),
                )
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

                    match add_key_interactive(&key_path, display_name) {
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

fn try_ssh_key_files_directly(username: &str) -> Result<Cred, Error> {
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

            match Cred::ssh_key(
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

// Returns Ok(count) of identities if agent is reachable, else Err.
fn agent_identities_count() -> Result<usize, Error> {
    let output = Command::new("ssh-add")
        .arg("-l")
        .env(
            "SSH_AUTH_SOCK",
            std::env::var("SSH_AUTH_SOCK").unwrap_or_default(),
        )
        .output()
        .map_err(|e| {
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

#[cfg(unix)]
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

fn prompt_persist_key_file(cfg: &BGitGlobalConfig, path: &std::path::Path) {
    // Only set if not already configured
    if cfg.auth.ssh.key_file.as_deref() == Some(path) {
        return;
    }

    let path_str = path.to_string_lossy();
    let question = format!(
        "Use '{}' as your default SSH key and save it to global config?",
        path_str
    );
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(true)
        .interact()
        .unwrap_or(false);
    if !confirm {
        debug!("User declined persisting ssh key_file");
        return;
    }

    let mut cfg_owned = cfg.clone();
    cfg_owned.auth.ssh.key_file = Some(path.to_path_buf());
    if let Err(e) = cfg_owned.save_global() {
        debug!("Failed to persist ssh key_file: {:?}", e);
    } else {
        println!("Saved default SSH key to global config: {}", path_str);
        debug!("Persisted ssh key_file to {:?}", path);
    }
}
