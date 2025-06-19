use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Password};
use git2::{
    CertificateCheckStatus, Cred, CredentialType, Error, ErrorClass, ErrorCode, RemoteCallbacks,
};
use log::debug;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

fn parse_ssh_agent_output(output: &str) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    for line in output.lines() {
        if line.contains('=') && (line.contains("SSH_AUTH_SOCK") || line.contains("SSH_AGENT_PID"))
        {
            if let Some(var_part) = line.split(';').next() {
                if let Some((key, value)) = var_part.split_once('=') {
                    env_vars.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    env_vars
}

fn spawn_ssh_agent_and_add_keys() -> Result<(), Error> {
    debug!("SSH_AUTH_SOCK not set, spawning ssh-agent");

    let output = Command::new("ssh-agent").arg("-s").output().map_err(|e| {
        Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            format!("Failed to spawn ssh-agent: {}", e),
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
    debug!("ssh-agent output: {}", agent_output);

    let env_vars = parse_ssh_agent_output(&agent_output);

    for (key, value) in &env_vars {
        unsafe {
            std::env::set_var(key, value);
        }
        debug!("Set environment variable: {}={}", key, value);
    }

    if env_vars.get("SSH_AUTH_SOCK").is_none() {
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
        debug!("SSH directory {:?} does not exist", ssh_dir);
        return Ok(()); // Not an error, just no keys to add
    }

    let key_files = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];

    let mut added_count = 0;

    for key_name in &key_files {
        let key_path = ssh_dir.join(key_name);

        if key_path.exists() {
            debug!("Found SSH key: {:?}", key_path);

            let output = Command::new("ssh-add")
                .arg(&key_path)
                .env(
                    "SSH_AUTH_SOCK",
                    std::env::var("SSH_AUTH_SOCK").unwrap_or_default(),
                )
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        debug!("Successfully added key: {}", key_name);
                        added_count += 1;
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        debug!("Failed to add key {}: {}", key_name, stderr);

                        // If it's a passphrase-protected key, we might need to handle it differently
                        if stderr.contains("Bad passphrase")
                            || stderr.contains("incorrect passphrase")
                        {
                            debug!(
                                "Key {} appears to be passphrase-protected, skipping automatic addition",
                                key_name
                            );
                        }
                    }
                }
                Err(e) => {
                    debug!("Error running ssh-add for {}: {}", key_name, e);
                }
            }
        } else {
            debug!("SSH key not found: {:?}", key_path);
        }
    }

    debug!("Added {} SSH keys to ssh-agent", added_count);

    // Don't fail if no keys were added - they might be passphrase-protected
    // or the user might authenticate differently
    if added_count == 0 {
        debug!("No SSH keys were automatically added, but this might be expected");
    }

    Ok(())
}

fn try_ssh_key_files_directly(username: &str) -> Result<Cred, Error> {
    debug!("Trying SSH key files directly for user: {}", username);

    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    let ssh_dir = Path::new(&home_dir).join(".ssh");
    let key_files = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];

    for key_name in &key_files {
        let private_key_path = ssh_dir.join(key_name);
        let public_key_path = ssh_dir.join(format!("{}.pub", key_name));

        if private_key_path.exists() && public_key_path.exists() {
            debug!("Trying SSH key pair: {} / {}.pub", key_name, key_name);

            match Cred::ssh_key(
                username,
                Some(&public_key_path),
                &private_key_path,
                None, // No passphrase for now
            ) {
                Ok(cred) => {
                    debug!("SSH key authentication succeeded with {}", key_name);
                    return Ok(cred);
                }
                Err(e) => {
                    debug!("SSH key authentication failed with {}: {}", key_name, e);
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

fn try_ssh_agent_auth(username: &str) -> Result<Cred, Error> {
    debug!("Attempting SSH agent authentication for user: {}", username);

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
            debug!("SSH agent authentication failed: {}", e);

            // Fallback to trying SSH key files directly
            debug!("Falling back to direct SSH key file authentication");
            try_ssh_key_files_directly(username)
        }
    }
}

fn try_userpass_authentication(username_from_url: Option<&str>) -> Result<Cred, Error> {
    debug!("USER_PASS_PLAINTEXT authentication is allowed, prompting for credentials");

    // Prompt for username if not provided in URL
    let username = if let Some(user) = username_from_url {
        user.to_string()
    } else {
        Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your username")
            .interact()
            .map_err(|e| {
                Error::new(
                    ErrorCode::Auth,
                    ErrorClass::Net,
                    format!("Failed to read username: {}", e),
                )
            })?
    };

    let token = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter your personal access token")
        .interact()
        .map_err(|e| {
            Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                format!("Failed to read token: {}", e),
            )
        })?;

    if !username.is_empty() && !token.is_empty() {
        debug!("Creating credentials with username and token");
        match Cred::userpass_plaintext(&username, &token) {
            Ok(cred) => {
                debug!("Username/token authentication succeeded");
                Ok(cred)
            }
            Err(e) => {
                debug!("Username/token authentication failed: {}", e);
                Err(e)
            }
        }
    } else {
        debug!("Username or token is empty, skipping userpass authentication");
        Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            "Username or token cannot be empty",
        ))
    }
}

fn ssh_authenticate_git(
    url: &str,
    username_from_url: Option<&str>,
    allowed_types: CredentialType,
    attempt_count: usize,
) -> Result<Cred, Error> {
    debug!(
        "Git authentication attempt #{} for URL: {}",
        attempt_count, url
    );
    debug!("Username from URL: {:?}", username_from_url);
    debug!("Allowed credential types: {:?}", allowed_types);

    // Prevent infinite loops
    if attempt_count > 3 {
        debug!(
            "Too many authentication attempts ({}), failing to prevent infinite loop",
            attempt_count
        );
        return Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            "Too many authentication attempts",
        ));
    }

    // Try SSH key authentication if allowed
    if allowed_types.contains(CredentialType::SSH_KEY) {
        if let Some(username) = username_from_url {
            debug!("SSH key authentication is allowed, trying SSH agent");

            if let Ok(cred) = try_ssh_agent_auth(username) {
                return Ok(cred);
            }
        } else {
            debug!("No username provided for SSH authentication");
        }
    }

    debug!(
        "All authentication methods failed for attempt {}",
        attempt_count
    );
    Err(Error::new(
        ErrorCode::Auth,
        ErrorClass::Net,
        format!("Authentication failed - attempt {}", attempt_count),
    ))
}

pub fn setup_auth_callbacks() -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();

    // Track attempt count across callback invocations
    let attempt_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    callbacks.credentials(move |url, username_from_url, allowed_types| {
        let mut count = attempt_count.lock().unwrap();
        *count += 1;
        let current_attempt = *count;
        drop(count);

        if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
            try_userpass_authentication(username_from_url)
        } else {
            ssh_authenticate_git(url, username_from_url, allowed_types, current_attempt)
        }
    });

    // Set up certificate check callback for HTTPS
    callbacks.certificate_check(|_cert, _host| {
        debug!("Skipping certificate verification (INSECURE)");
        Ok(CertificateCheckStatus::CertificateOk)
    });

    callbacks
}
