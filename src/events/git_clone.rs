use super::AtomicEvent;
use crate::bgit_error::BGitError;
use crate::rules::Rule;
use git2::{Cred, CredentialType};
use std::env;
use std::path::Path;

pub struct GitClone {
    pub pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    pub url: String,
}

impl AtomicEvent for GitClone {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitClone {
            pre_check_rules: vec![],
            url: String::new(),
        }
    }

    fn get_name(&self) -> &str {
        "git_clone"
    }

    fn get_action_description(&self) -> &str {
        "Clone a Git repository"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        // Check if URL is set
        if self.url.is_empty() {
            return Err(self.to_bgit_error("Repository URL is not set"));
        }
        let url = &self.url;
        let repo_name = match url.split("/").last() {
            Some(repo_name) => repo_name.strip_suffix(".git").unwrap_or(repo_name),
            None => {
                return Err(self.to_bgit_error("Failed to get repository name from URL"));
            }
        };

        // Create fetch options with authentication
        let fetch_options = Self::create_fetch_options();

        // Clone repository with authentication options
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        builder.clone(&self.url, Path::new(repo_name)).map_err(|e| {
            self.to_bgit_error(&format!("Failed to clone repository: {e}. Please check your SSH keys or authentication setup."))
        })?;

        self.update_cwd_path()?;

        Ok(true)
    }
}

impl GitClone {
    pub fn set_url(&mut self, url: &str) -> &mut Self {
        self.url = url.to_owned();
        self
    }

    fn update_cwd_path(&self) -> Result<(), Box<BGitError>> {
        let repo_name = match self.url.split("/").last() {
            Some(repo_name) => repo_name.strip_suffix(".git").unwrap_or(repo_name),
            None => {
                return Err(self.to_bgit_error("Failed to get repository name from URL"));
            }
        };

        match env::set_current_dir(repo_name) {
            Ok(_) => Ok(()),
            Err(_) => Err(self.to_bgit_error("Failed to update current working directory path")),
        }
    }

    /// Set up authentication callbacks for git operations
    fn setup_auth_callbacks() -> git2::RemoteCallbacks<'static> {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let mut callbacks = git2::RemoteCallbacks::new();
        let attempt_count = Arc::new(AtomicUsize::new(0));

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            let current_attempt = attempt_count.fetch_add(1, Ordering::SeqCst);
            // Limit authentication attempts to prevent infinite loops
            if current_attempt > 3 {
                return Err(git2::Error::new(
                    git2::ErrorCode::Auth,
                    git2::ErrorClass::Net,
                    "Maximum authentication attempts exceeded",
                ));
            }

            // If SSH key authentication is allowed
            if allowed_types.contains(CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url {
                    // Try SSH agent first (most common and secure)
                    match Cred::ssh_key_from_agent(username) {
                        Ok(cred) => {
                            return Ok(cred);
                        }
                        Err(e) => {
                            println!("SSH agent failed: {e}");
                        }
                    }

                    // Try to find SSH keys in standard locations
                    let home_dir = std::env::var("HOME")
                        .or_else(|_| std::env::var("USERPROFILE"))
                        .unwrap_or_else(|_| ".".to_string());

                    let ssh_dir = Path::new(&home_dir).join(".ssh");

                    // Common SSH key file names in order of preference
                    let key_files = [
                        ("id_ed25519", "id_ed25519.pub"),
                        ("id_rsa", "id_rsa.pub"),
                        ("id_ecdsa", "id_ecdsa.pub"),
                        ("id_dsa", "id_dsa.pub"),
                    ];

                    for (private_name, public_name) in &key_files {
                        let private_key = ssh_dir.join(private_name);
                        let public_key = ssh_dir.join(public_name);

                        if private_key.exists() {
                            // Try with public key if it exists
                            if public_key.exists() {
                                match Cred::ssh_key(username, Some(&public_key), &private_key, None)
                                {
                                    Ok(cred) => {
                                        return Ok(cred);
                                    }
                                    Err(e) => {
                                        eprintln!("SSH key with public key failed: {e}");
                                    }
                                }
                            }

                            // Try without public key
                            match Cred::ssh_key(username, None, &private_key, None) {
                                Ok(cred) => {
                                    return Ok(cred);
                                }
                                Err(e) => {
                                    eprintln!("SSH key without public key failed: {e}");
                                }
                            }
                        }
                    }
                } else {
                    eprintln!("No username provided for SSH authentication");
                }
            }

            // If username/password authentication is allowed (HTTPS)
            if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                // Try to get credentials from git config or environment
                if let (Ok(username), Ok(password)) =
                    (std::env::var("GIT_USERNAME"), std::env::var("GIT_PASSWORD"))
                {
                    return Cred::userpass_plaintext(&username, &password);
                }

                // For GitHub, you might want to use a personal access token
                if url.contains("github.com") {
                    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                        return Cred::userpass_plaintext("git", &token);
                    }
                }
            }

            // Default authentication (tries default SSH key)
            if allowed_types.contains(CredentialType::DEFAULT) {
                match Cred::default() {
                    Ok(cred) => {
                        return Ok(cred);
                    }
                    Err(e) => {
                        eprintln!("Default authentication failed: {e}");
                    }
                }
            }

            Err(git2::Error::new(
                git2::ErrorCode::Auth,
                git2::ErrorClass::Net,
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
            // In production, you should properly validate certificates
            // For now, we'll accept all certificates (not recommended for production)
            Ok(git2::CertificateCheckStatus::CertificateOk)
        });

        callbacks
    }

    /// Create fetch options with authentication
    fn create_fetch_options() -> git2::FetchOptions<'static> {
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(Self::setup_auth_callbacks());
        fetch_options
    }
}
