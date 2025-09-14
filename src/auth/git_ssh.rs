use dialoguer::{Confirm, theme::ColorfulTheme};
use git2::{Cred, CredentialType, Error, ErrorClass, ErrorCode};
use log::debug;
use std::path::PathBuf;

use crate::auth::auth_utils::prompt_persist_preferred_auth;
use crate::auth::ssh::{
    add_all_ssh_keys, agent_identities_count, ensure_agent_ready, try_ssh_key_files_directly,
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
