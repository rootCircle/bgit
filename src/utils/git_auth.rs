use git2::{
    CertificateCheckStatus, Cred, CredentialType, Error, ErrorClass, ErrorCode, RemoteCallbacks,
};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

pub fn setup_auth_callbacks() -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    let attempt_count = Arc::new(AtomicUsize::new(0));

    callbacks.credentials(move |url, username_from_url, allowed_types| {
        let current_attempt = attempt_count.fetch_add(1, Ordering::SeqCst);
        println!(
            "[DEBUG] Authentication attempt #{} for URL: {}",
            current_attempt + 1,
            url
        );

        // Limit authentication attempts to prevent infinite loops
        if current_attempt > 3 {
            println!("[DEBUG] Maximum authentication attempts exceeded");
            return Err(Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                "Maximum authentication attempts exceeded",
            ));
        }

        // If SSH key authentication is allowed
        if allowed_types.contains(CredentialType::SSH_KEY) {
            println!("[DEBUG] SSH_KEY authentication allowed");
            if let Some(username) = username_from_url {
                println!("[DEBUG] Username from URL: {}", username);

                // match Cred::ssh_key_from_agent(username) {
                //     Ok(cred) => {
                //         return Ok(cred);
                //     }
                //     Err(e) => {
                //         println!("SSH agent failed: {}", e);
                //     }
                // }

                // Try to find SSH keys in standard locations
                let home_dir = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                println!("[DEBUG] Home directory resolved to: {}", home_dir);

                let ssh_dir = Path::new(&home_dir).join(".ssh");
                println!("[DEBUG] Checking .ssh directory at: {:?}", ssh_dir);

                // Common SSH key file names in order of preference
                let key_files = [
                    // ("id_ed25519", "id_ed25519.pub"),
                    ("id_rsa", "id_rsa.pub"),
                    // ("id_ecdsa", "id_ecdsa.pub"),
                    // ("id_dsa", "id_dsa.pub"),
                ];

                for (private_name, public_name) in &key_files {
                    let private_key = ssh_dir.join(private_name);
                    let public_key = ssh_dir.join(public_name);
                    println!(
                        "[DEBUG] Trying key pair: {:?}, {:?}",
                        private_key, public_key
                    );

                    if private_key.exists() {
                        println!("[DEBUG] Found private key: {:?}", private_key);

                        if public_key.exists() {
                            println!("[DEBUG] Found public key: {:?}", public_key);
                            match Cred::ssh_key(username, Some(&public_key), &private_key, None) {
                                Ok(cred) => {
                                    println!("[DEBUG] SSH key auth with public key succeeded");
                                    return Ok(cred);
                                }
                                Err(e) => {
                                    eprintln!("SSH key with public key failed: {}", e);
                                }
                            }
                        }

                        println!("[DEBUG] Trying SSH key without public key");
                        match Cred::ssh_key(username, None, &private_key, None) {
                            Ok(cred) => {
                                println!("[DEBUG] SSH key auth without public key succeeded");
                                return Ok(cred);
                            }
                            Err(e) => {
                                eprintln!("SSH key without public key failed: {}", e);
                            }
                        }
                    } else {
                        println!("[DEBUG] Private key not found: {:?}", private_key);
                    }
                }
            } else {
                eprintln!("No username provided for SSH authentication");
            }
        }

        // If username/password authentication is allowed (HTTPS)
        if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
            println!("[DEBUG] USER_PASS_PLAINTEXT authentication allowed");

            if let (Ok(username), Ok(password)) =
                (std::env::var("GIT_USERNAME"), std::env::var("GIT_PASSWORD"))
            {
                println!("[DEBUG] Using GIT_USERNAME and GIT_PASSWORD from environment");
                return Cred::userpass_plaintext(&username, &password);
            }

            if url.contains("github.com") {
                println!("[DEBUG] URL contains github.com, checking for GITHUB_TOKEN");
                if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                    println!("[DEBUG] Using GITHUB_TOKEN from environment");
                    return Cred::userpass_plaintext("git", &token);
                } else {
                    println!("[DEBUG] GITHUB_TOKEN not found in environment");
                }
            }
        }

        // Default authentication (tries default SSH key)
        if allowed_types.contains(CredentialType::DEFAULT) {
            println!("[DEBUG] Attempting default credentials");
            match Cred::default() {
                Ok(cred) => {
                    println!("[DEBUG] Default credentials succeeded");
                    return Ok(cred);
                }
                Err(e) => {
                    eprintln!("Default authentication failed: {}", e);
                }
            }
        }

        println!(
            "[DEBUG] Authentication failed after {} attempts for {}. Available methods: {:?}",
            current_attempt + 1,
            url,
            allowed_types
        );
        Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            format!(
                "Authentication failed after {} attempts for {}. Available methods: {:?}",
                current_attempt + 1,
                url,
                allowed_types
            ),
        ))
    });

    // Set up certificate check callback for HTTPS
    callbacks.certificate_check(|_cert, _host| {
        println!("[DEBUG] Skipping certificate verification (INSECURE)");
        Ok(CertificateCheckStatus::CertificateOk)
    });

    callbacks
}
