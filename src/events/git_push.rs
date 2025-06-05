use std::path::Path;

use super::AtomicEvent;
use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_RULE, NO_STEP};
use crate::rules::Rule;
use git2::{Cred, CredentialType, Repository};

pub struct GitPush {
    pub pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    pub force: bool,
    pub set_upstream: bool,
}

impl AtomicEvent for GitPush {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitPush {
            pre_check_rules: vec![],
            force: false,
            set_upstream: false,
        }
    }

    fn get_name(&self) -> &str {
        "git_push"
    }

    fn get_action_description(&self) -> &str {
        "Push changes to remote repository"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        let repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to discover repository: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Get the current branch
        let head = repo.head().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get HEAD reference: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        let branch_name = head.shorthand().ok_or_else(|| {
            Box::new(BGitError::new(
                "BGitError",
                "Failed to get branch name",
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Get remote
        let mut remote = repo.find_remote("origin").map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to find remote 'origin': {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Prepare push options with authentication
        let mut push_options = Self::create_push_options();

        // Check if we need to force push and validate state
        if !self.force {
            self.validate_push_safety(&repo, &head, branch_name)?;
        }

        // Determine refspec
        let refspec = if self.set_upstream {
            format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)
        } else {
            format!("refs/heads/{}", branch_name)
        };

        // Perform the push
        let refspecs = if self.force {
            vec![format!("+{}", refspec)]
        } else {
            vec![refspec]
        };

        remote
            .push(&refspecs, Some(&mut push_options))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to push to remote: {}. Please check your SSH keys or authentication setup.", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        // Set upstream if requested and push was successful
        if self.set_upstream {
            self.set_upstream_branch(&repo, branch_name)?;
        }

        Ok(true)
    }
}

impl GitPush {
    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = force;
        self
    }

    pub fn set_upstream_flag(&mut self, set_upstream: bool) -> &mut Self {
        self.set_upstream = set_upstream;
        self
    }

    fn validate_push_safety(
        &self,
        repo: &Repository,
        head: &git2::Reference,
        branch_name: &str,
    ) -> Result<(), Box<BGitError>> {
        // Check if we're up to date with remote
        if let Ok(remote_ref) = repo.find_reference(&format!("refs/remotes/origin/{}", branch_name))
        {
            let local_commit = head.peel_to_commit().map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get local commit: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

            let remote_commit = remote_ref.peel_to_commit().map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get remote commit: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

            // If commits are the same, we're up to date
            if local_commit.id() == remote_commit.id() {
                return Ok(());
            }

            // Check if local is behind remote
            let merge_base = repo
                .merge_base(local_commit.id(), remote_commit.id())
                .map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to find merge base: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_STEP,
                        self.get_name(),
                        NO_RULE,
                    ))
                })?;

            if merge_base == local_commit.id() && local_commit.id() != remote_commit.id() {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    "Local branch is behind remote. Pull changes first or use --force",
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                )));
            }
        }

        Ok(())
    }

    fn set_upstream_branch(
        &self,
        repo: &Repository,
        branch_name: &str,
    ) -> Result<(), Box<BGitError>> {
        let mut branch = repo
            .find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to find local branch {}: {}", branch_name, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        let upstream_name = format!("origin/{}", branch_name);
        branch.set_upstream(Some(&upstream_name)).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to set upstream to {}: {}", upstream_name, e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        Ok(())
    }

    /// Set up authentication callbacks for git operations
    fn setup_auth_callbacks() -> git2::RemoteCallbacks<'static> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let mut callbacks = git2::RemoteCallbacks::new();
        let attempt_count = Arc::new(AtomicUsize::new(0));

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            let current_attempt = attempt_count.fetch_add(1, Ordering::SeqCst);
            // Limit authentication attempts to prevent infinite loops
            if current_attempt > 3 {
                return Err(git2::Error::new(
                    git2::ErrorCode::Auth,
                    git2::ErrorClass::Net,
                    "Maximum authentication attempts exceeded"
                ));
            }
            
            
            // If SSH key authentication is allowed
            if allowed_types.contains(CredentialType::SSH_KEY) {
                if let Some(username) = username_from_url {
                    
                    // Try SSH agent first (most common and secure)
                    match Cred::ssh_key_from_agent(username) {
                        Ok(cred) => {
                            return Ok(cred);
                        },
                        Err(e) => {
                            println!("SSH agent failed: {}", e);
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
                                match Cred::ssh_key(username, Some(&public_key), &private_key, None) {
                                    Ok(cred) => {
                                        return Ok(cred);
                                    },
                                    Err(e) => {
                                        println!("SSH key with public key failed: {}", e);
                                    }
                                }
                            }
                            
                            // Try without public key
                            match Cred::ssh_key(username, None, &private_key, None) {
                                Ok(cred) => {
                                    return Ok(cred);
                                },
                                Err(e) => {
                                    println!("SSH key without public key failed: {}", e);
                                }
                            }
                        }
                    }
                } else {
                    println!("No username provided for SSH authentication");
                }
            }
            
            // If username/password authentication is allowed (HTTPS)
            if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                
                // Try to get credentials from git config or environment
                if let (Ok(username), Ok(password)) = (
                    std::env::var("GIT_USERNAME"),
                    std::env::var("GIT_PASSWORD")
                ) {
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
                    },
                    Err(e) => {
                        println!("Default authentication failed: {}", e);
                    }
                }
            }
            
            Err(git2::Error::new(
                git2::ErrorCode::Auth,
                git2::ErrorClass::Net,
                format!(
                    "Authentication failed after {} attempts for {}. Available methods: {:?}",
                    current_attempt + 1, url, allowed_types
                )
            ))
        });

        // Add push update reference callback for better error reporting
        callbacks.push_update_reference(|refname, status| match status {
            Some(msg) => {
                println!("Push failed for {}: {}", refname, msg);
                Err(git2::Error::from_str(msg))
            }
            None => {
                println!("Push successful for {}", refname);
                Ok(())
            }
        });

        // Set up certificate check callback for HTTPS
        callbacks.certificate_check(|_cert, _host| {
            // In production, you should properly validate certificates
            // For now, we'll accept all certificates (not recommended for production)
            println!("Certificate check for host: {}", _host);
            Ok(git2::CertificateCheckStatus::CertificateOk)
        });

        callbacks
    }

    /// Create push options with authentication
    fn create_push_options() -> git2::PushOptions<'static> {
        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(Self::setup_auth_callbacks());
        push_options
    }
}
