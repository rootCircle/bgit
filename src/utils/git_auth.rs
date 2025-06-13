use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Password};
use git2::{
    CertificateCheckStatus, Cred, CredentialType, Error, ErrorClass, ErrorCode, RemoteCallbacks,
};
use log::debug;
use std::path::Path;
use std::sync::{Arc, Mutex};

fn try_ssh_agent_auth(username: &str) -> Result<Cred, Error> {
    debug!("Attempting SSH agent authentication for user: {}", username);

    if std::env::var("SSH_AUTH_SOCK").is_err() {
        debug!("SSH_AUTH_SOCK not set, skipping ssh_key_from_agent");
        return Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            "SSH_AUTH_SOCK not available",
        ));
    }

    match Cred::ssh_key_from_agent(username) {
        Ok(cred) => {
            debug!("SSH agent authentication succeeded");
            Ok(cred)
        }
        Err(e) => {
            debug!("SSH agent authentication failed: {}", e);
            Err(e)
        }
    }
}

fn try_ssh_key_files(
    username: &str,
    key_index: usize,
    use_public_key: bool,
) -> Result<Cred, Error> {
    debug!(
        "Attempting SSH key file authentication for user: {}, key_index: {}, use_public_key: {}",
        username, key_index, use_public_key
    );

    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    debug!("Home directory resolved to: {}", home_dir);

    let ssh_dir = Path::new(&home_dir).join(".ssh");
    debug!("Checking .ssh directory at: {:?}", ssh_dir);

    // Common SSH key file names in order of preference
    let key_files = [
        ("id_ed25519", "id_ed25519.pub"),
        ("id_rsa", "id_rsa.pub"),
        ("id_ecdsa", "id_ecdsa.pub"),
        ("id_dsa", "id_dsa.pub"),
    ];

    if key_index >= key_files.len() {
        debug!("Key index {} out of range", key_index);
        return Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            "All SSH key files exhausted",
        ));
    }

    let (private_name, public_name) = key_files[key_index];
    let private_key = ssh_dir.join(private_name);
    let public_key = ssh_dir.join(public_name);

    if !private_key.exists() {
        debug!("Private key not found: {:?}", private_key);
        return Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            format!("Private key not found: {}", private_name),
        ));
    }

    debug!("Found private key: {:?}", private_key);

    if use_public_key {
        if public_key.exists() {
            debug!("Found public key: {:?}, trying with public key", public_key);
            match Cred::ssh_key(username, Some(&public_key), &private_key, None) {
                Ok(cred) => {
                    debug!(
                        "SSH key auth with public key succeeded for {}",
                        private_name
                    );
                    Ok(cred)
                }
                Err(e) => {
                    debug!("SSH key with public key failed for {}: {}", private_name, e);
                    Err(e)
                }
            }
        } else {
            debug!(
                "Public key not found for {}, skipping this attempt",
                private_name
            );
            Err(Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                format!("Public key not found for {}", private_name),
            ))
        }
    } else {
        debug!("Trying SSH key without public key for {}", private_name);
        match Cred::ssh_key(username, None, &private_key, None) {
            Ok(cred) => {
                debug!(
                    "SSH key auth without public key succeeded for {}",
                    private_name
                );
                Ok(cred)
            }
            Err(e) => {
                debug!(
                    "SSH key without public key failed for {}: {}",
                    private_name, e
                );
                Err(e)
            }
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
    if attempt_count > 20 {
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
            debug!("SSH key authentication is allowed, trying SSH methods");

            // Try SSH agent first (only on first attempt)
            if attempt_count == 1 {
                if let Ok(cred) = try_ssh_agent_auth(username) {
                    return Ok(cred);
                }
                // If SSH agent fails, fall through to try SSH key files on same attempt
            }

            // Try SSH key files with progression
            // Attempt 1+: Start with id_ed25519 if SSH agent failed
            // Attempt 1: id_ed25519 with public key
            // Attempt 2: id_ed25519 without public key
            // Attempt 3: id_rsa with public key
            // Attempt 4: id_rsa without public key
            // etc.
            let key_attempt = attempt_count - 1;
            let key_index = key_attempt / 2;
            let use_public_key = key_attempt % 2 == 0;

            if let Ok(cred) = try_ssh_key_files(username, key_index, use_public_key) {
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
