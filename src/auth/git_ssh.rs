use git2::{Cred, CredentialType, Error, ErrorClass, ErrorCode};
use log::debug;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::auth::ssh_utils::add_key_interactive;
use crate::constants::{MAX_AUTH_ATTEMPTS, SSH_AGENT_SOCKET_BASENAME};

pub fn ssh_authenticate_git(
    url: &str,
    username_from_url: Option<&str>,
    allowed_types: CredentialType,
    attempt_count: usize,
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

            // If the agent is up but has no identities, try to add keys once.
            if agent_identities_count().unwrap_or(0) == 0 && attempt_count <= MAX_AUTH_ATTEMPTS {
                debug!("ssh-agent has no identities, attempting to add keys from ~/.ssh");
                let _ = add_all_ssh_keys();
            }

            if let Ok(cred) = try_ssh_agent_auth(username) {
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

        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
        let socket_path = Path::new(&home)
            .join(".ssh")
            .join(SSH_AGENT_SOCKET_BASENAME);

        // Create ~/.ssh if needed
        if let Err(e) = std::fs::create_dir_all(Path::new(&home).join(".ssh")) {
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

        // Otherwise, use our fixed socket path
        unsafe {
            std::env::set_var("SSH_AUTH_SOCK", &socket_path);
        }

        let alive = || -> bool {
            if let Ok(md) = std::fs::metadata(&socket_path) {
                if md.file_type().is_socket() {
                    // Probe agent via ssh-add -l
                    return agent_identities_count().is_ok();
                }
            }
            false
        }();

        // Remove stale non-socket file
        if let Ok(md) = std::fs::metadata(&socket_path) {
            if !md.file_type().is_socket() {
                let _ = std::fs::remove_file(&socket_path);
            }
        }

        if !alive {
            start_agent_detached(Some(&socket_path))?;
            // Small wait loop for socket readiness
            for _ in 0..20 {
                if std::fs::metadata(&socket_path)
                    .map(|m| m.file_type().is_socket())
                    .unwrap_or(false)
                {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        Ok(())
    }

    #[cfg(not(unix))]
    {
        // On Windows, rely on existing agent (Pageant or OpenSSH agent). If SSH_AUTH_SOCK is missing, try to start one.
        if std::env::var("SSH_AUTH_SOCK").is_err() {
            start_agent_detached(None)?;
        }
        Ok(())
    }
}

fn start_agent_detached(socket: Option<&Path>) -> Result<(), Error> {
    // Try to start ssh-agent in background without making bgit its parent.
    // Prefer setsid/nohup if available (Unix). On Windows, best effort spawn.
    #[cfg(unix)]
    {
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
            // Use -a to bind to our fixed socket when supported
            let supports_a = Command::new("ssh-agent")
                .arg("-h")
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if supports_a {
                cmd.arg("-a").arg(sock);
            }
            // Keep in foreground and let setsid/nohup detach it; discard stdio
            cmd.arg("-D");
        } else {
            cmd.arg("-D");
        }

        let _child = cmd
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

    #[cfg(not(unix))]
    {
        let mut cmd = Command::new("ssh-agent");
        let _child = cmd
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

fn add_all_ssh_keys() -> Result<(), Error> {
    debug!("Adding all SSH keys from .ssh folder to ssh-agent");

    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    let ssh_dir = Path::new(&home_dir).join(".ssh");

    if !ssh_dir.exists() {
        debug!("SSH directory {ssh_dir:?} does not exist");
        return Ok(());
    }

    let key_files = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];
    let mut added_count = 0;

    for key_name in &key_files {
        let key_path = ssh_dir.join(key_name);

        if key_path.exists() {
            debug!("Found SSH key: {key_path:?}");

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
                    debug!("Successfully added key without interaction: {key_name}");
                    added_count += 1;
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    debug!("Quick add failed for {key_name}: {stderr}");

                    debug!("Key {key_name} appears to need passphrase, trying interactive add");

                    match add_key_interactive(&key_path, key_name) {
                        Ok(true) => {
                            debug!("Successfully added key interactively: {key_name}");
                            added_count += 1;
                        }
                        Ok(false) => {
                            debug!("User skipped key: {key_name}");
                        }
                        Err(e) => {
                            debug!("Interactive add failed for {key_name}: {e}");
                        }
                    }
                }
                Err(e) => {
                    debug!("Error running ssh-add for {key_name}: {e}");
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

    Ok(())
}

fn try_ssh_key_files_directly(username: &str) -> Result<Cred, Error> {
    debug!("Trying SSH key files directly for user: {username}");

    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    let ssh_dir = Path::new(&home_dir).join(".ssh");
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
        let count = stdout.lines().count();
        Ok(count)
    } else {
        Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            format!(
                "ssh-add -l failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}
