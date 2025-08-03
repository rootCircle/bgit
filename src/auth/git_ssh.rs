use git2::{Cred, CredentialType, Error, ErrorClass, ErrorCode};
use log::debug;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::auth::ssh_utils::{add_key_interactive, parse_ssh_agent_output};

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
    if attempt_count > 3 {
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

            // handling the case where ssh-agent is running but empty
            if attempt_count == 2 {
                debug!("Second attempt: trying to add SSH keys to agent before authentication");
                if std::env::var("SSH_AUTH_SOCK").is_ok() {
                    if let Err(e) = add_all_ssh_keys() {
                        debug!("Failed to add keys to ssh-agent on second attempt: {e}");
                    } else {
                        debug!("Keys added to ssh-agent, proceeding with authentication");
                    }
                }
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

    if std::env::var("SSH_AUTH_SOCK").is_err() {
        debug!("SSH_AUTH_SOCK not set, attempting to spawn ssh-agent and add keys");
        spawn_ssh_agent_and_add_keys()?;
    }

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

fn spawn_ssh_agent_and_add_keys() -> Result<(), Error> {
    debug!("SSH_AUTH_SOCK not set, spawning ssh-agent");

    let output = Command::new("ssh-agent").arg("-s").output().map_err(|e| {
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
            format!("ssh-agent failed with status: {}", output.status),
        ));
    }

    let agent_output = String::from_utf8_lossy(&output.stdout);
    debug!("ssh-agent output: {agent_output}");

    let env_vars = parse_ssh_agent_output(&agent_output);

    for (key, value) in &env_vars {
        unsafe {
            std::env::set_var(key, value);
        }
        debug!("Set environment variable: {key}={value}");
    }

    if !env_vars.contains_key("SSH_AUTH_SOCK") {
        return Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            "Failed to parse SSH_AUTH_SOCK from ssh-agent output",
        ));
    }

    add_all_ssh_keys()?;

    Ok(())
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
