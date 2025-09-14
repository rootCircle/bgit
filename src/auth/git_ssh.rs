use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use git2::{Cred, CredentialType, Error, ErrorClass, ErrorCode};
use log::debug;
use std::path::PathBuf;

use crate::auth::auth_utils::prompt_persist_preferred_auth;
use crate::auth::ssh::{
    add_all_ssh_keys_with_auth, add_key_interactive_with_auth, agent_identities_count_with_auth,
    ensure_agent_ready, get_effective_ssh_auth, set_global_ssh_env_for_libgit2,
    try_ssh_key_files_directly,
};
use crate::config::global::{BGitGlobalConfig, PreferredAuth};
use crate::constants::MAX_AUTH_ATTEMPTS;

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

            // Get effective SSH auth configuration
            let (effective_socket, effective_pid) = get_effective_ssh_auth();
            debug!(
                "Using effective SSH auth - socket: {:?}, pid: {:?}",
                effective_socket, effective_pid
            );

            let identity_count = agent_identities_count_with_auth(
                effective_socket.as_deref(),
                effective_pid.as_deref(),
            )
            .unwrap_or(0);

            if identity_count == 0 && attempt_count <= MAX_AUTH_ATTEMPTS {
                debug!("ssh-agent has no identities, attempting to add keys from ~/.ssh");
                if let Ok(first_added) = add_all_ssh_keys_with_auth(
                    cfg,
                    effective_socket.as_deref(),
                    effective_pid.as_deref(),
                ) {
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

    let (effective_socket, effective_pid) = get_effective_ssh_auth();
    set_global_ssh_env_for_libgit2(effective_socket.as_deref(), effective_pid.as_deref());

    match Cred::ssh_key_from_agent(username) {
        Ok(cred) => {
            debug!("SSH agent authentication succeeded");

            Ok(cred)
        }
        Err(e) => {
            debug!("SSH agent authentication failed: {e}");

            // If agent auth failed, offer to add a key manually before falling back to direct files
            let (effective_socket, effective_pid) = get_effective_ssh_auth();
            if offer_manual_key_addition(effective_socket.as_deref(), effective_pid.as_deref()) {
                // Retry with agent after adding key
                debug!("Retrying SSH agent authentication after manual key addition");
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    debug!("SSH agent authentication succeeded after manual key addition");
                    return Ok(cred);
                }
            }

            // Fallback to trying SSH key files directly
            debug!("Falling back to direct SSH key file authentication");
            try_ssh_key_files_directly(username)
        }
    }
}

/// Offers user the option to manually add a specific SSH key when authentication fails
fn offer_manual_key_addition(socket_path: Option<&str>, agent_pid: Option<&str>) -> bool {
    let ssh_dir = home::home_dir()
        .map(|p| p.join(".ssh"))
        .unwrap_or_else(|| std::path::PathBuf::from(".ssh"));

    if !ssh_dir.exists() {
        debug!("No SSH directory found, cannot offer manual key addition");
        return false;
    }

    let key_files = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];
    let mut available_keys = Vec::new();

    for key_name in &key_files {
        let private_key_path = ssh_dir.join(key_name);
        let public_key_path = ssh_dir.join(format!("{key_name}.pub"));

        if private_key_path.exists() && public_key_path.exists() {
            available_keys.push((private_key_path, key_name.to_string()));
        }
    }

    if available_keys.is_empty() {
        debug!("No SSH key pairs found, cannot offer manual key addition");
        return false;
    }

    println!("SSH agent authentication failed. Available SSH keys:");
    let mut options = Vec::new();
    for (_, key_name) in &available_keys {
        options.push(format!("Add {} to SSH agent", key_name));
    }
    options.push("Skip manual key addition".to_string());

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to add an SSH key to the agent?")
        .items(&options)
        .default(0)
        .interact()
        .unwrap_or(options.len() - 1); // Default to "skip" on error

    if selection >= available_keys.len() {
        debug!("User chose to skip manual key addition");
        return false;
    }

    let (key_path, key_name) = &available_keys[selection];
    debug!("User selected to add key: {}", key_name);

    match add_key_interactive_with_auth(key_path, key_name, socket_path, agent_pid) {
        Ok(true) => {
            println!("Successfully added SSH key '{}' to agent!", key_name);
            true
        }
        Ok(false) => {
            debug!("User cancelled key addition for: {}", key_name);
            false
        }
        Err(e) => {
            debug!("Failed to add key {}: {}", key_name, e);
            false
        }
    }
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
