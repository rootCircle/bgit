use dialoguer::{Confirm, theme::ColorfulTheme};
use git2::{Error, ErrorClass, ErrorCode};
use log::debug;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::global::BGitGlobalConfig;

/// Get the count of identities in SSH agent (platform-agnostic)
pub fn agent_identities_count() -> Result<usize, Error> {
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

/// Interactively add a key to SSH agent (platform-agnostic)
pub fn add_key_interactive(key_path: &Path, key_name: &str) -> Result<bool, Error> {
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

    // Use interactive ssh-add - this will prompt the user directly in the terminal
    let status = Command::new("ssh-add")
        .arg(key_path)
        .env(
            "SSH_AUTH_SOCK",
            std::env::var("SSH_AUTH_SOCK").unwrap_or_default(),
        )
        .stdin(Stdio::inherit()) // Allow user to input passphrase directly
        .stdout(Stdio::inherit()) // Show ssh-add output to user
        .stderr(Stdio::inherit()) // Show ssh-add errors to user
        .status() // Use status() instead of output() to allow real-time interaction
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

/// Add all available SSH keys to the agent (platform-agnostic)
pub fn add_all_ssh_keys(cfg: &BGitGlobalConfig) -> Result<Option<PathBuf>, Error> {
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
