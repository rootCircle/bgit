use super::AtomicEvent;
use crate::bgit_error::BGitError;
use crate::rules::Rule;
use crate::utils::git_auth::setup_auth_callbacks;
use git2::Repository;
use log::debug;
use std::path::Path;

pub struct GitPush {
    pub pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    pub force_with_lease: bool,
    pub set_upstream: bool,
}

impl AtomicEvent for GitPush {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitPush {
            pre_check_rules: vec![],
            force_with_lease: false,
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
        let repo = Repository::discover(Path::new("."))
            .map_err(|e| self.to_bgit_error(&format!("Failed to discover repository: {e}")))?;

        // Get the current branch - handle unborn branch case
        let (head, branch_name) = match repo.head() {
            Ok(head) => {
                let branch_name = head
                    .shorthand()
                    .ok_or_else(|| self.to_bgit_error("Failed to get branch name"))?
                    .to_string();
                (head, branch_name)
            }
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                return Err(self.to_bgit_error("Cannot push from unborn branch (no commits to push). Create your first commit before pushing."));
            }
            Err(e) => {
                return Err(self.to_bgit_error(&format!("Failed to get HEAD reference: {e}")));
            }
        };

        // Get remote - handle case where no remote is configured
        let mut remote = match repo.find_remote("origin") {
            Ok(remote) => remote,
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                return Err(self.to_bgit_error("No remote 'origin' configured. Please add a remote repository first with: git remote add origin <repository-url>"));
            }
            Err(e) => {
                return Err(self.to_bgit_error(&format!("Failed to find remote 'origin': {e}")));
            }
        };

        // Prepare push options with authentication
        let mut push_options = Self::create_push_options();

        // Validation
        if self.force_with_lease {
            self.validate_force_with_lease(&repo, &head, &branch_name)?;
        } else {
            self.validate_push_safety(&repo, &head, &branch_name)?;
        }

        let refspec = if self.set_upstream {
            format!("refs/heads/{branch_name}:refs/heads/{branch_name}")
        } else {
            format!("refs/heads/{branch_name}")
        };

        // Perform the push with force-with-lease if enabled
        let refspecs = if self.force_with_lease {
            let force_lease_refspec =
                self.build_force_with_lease_refspec(&repo, &branch_name, &refspec)?;
            vec![force_lease_refspec]
        } else {
            vec![refspec]
        };

        remote
            .push(&refspecs, Some(&mut push_options))
            .map_err(|e| {
                self.to_bgit_error(&format!("Failed to push to remote: {e}. Please check your SSH keys or authentication setup."))
            })?;

        // Set upstream if requested and push was successful
        if self.set_upstream {
            self.set_upstream_branch(&repo, &branch_name)?;
        }

        Ok(true)
    }
}

impl GitPush {
    pub fn with_force_with_lease(&mut self, force_with_lease: bool) -> &mut Self {
        self.force_with_lease = force_with_lease;
        self
    }

    pub fn with_upstream_flag(&mut self, set_upstream: bool) -> &mut Self {
        self.set_upstream = set_upstream;
        self
    }

    /// Validate force-with-lease conditions
    fn validate_force_with_lease(
        &self,
        repo: &Repository,
        head: &git2::Reference,
        branch_name: &str,
    ) -> Result<(), Box<BGitError>> {
        let local_commit = head
            .peel_to_commit()
            .map_err(|e| self.to_bgit_error(&format!("Failed to get local commit: {e}")))?;

        // Check if remote branch exists and validate
        if let Ok(remote_ref) = repo.find_reference(&format!("refs/remotes/origin/{branch_name}")) {
            let remote_commit = remote_ref
                .peel_to_commit()
                .map_err(|e| self.to_bgit_error(&format!("Failed to get remote commit: {e}")))?;

            if local_commit.id() == remote_commit.id() {
                debug!("Local branch is up to date with remote, no force-with-lease needed");
                return Ok(());
            }
        }

        Ok(())
    }

    fn build_force_with_lease_refspec(
        &self,
        repo: &Repository,
        branch_name: &str,
        base_refspec: &str,
    ) -> Result<String, Box<BGitError>> {
        // Force-with-lease using the current remote tracking branch as the expected value
        if let Ok(remote_ref) = repo.find_reference(&format!("refs/remotes/origin/{branch_name}")) {
            let remote_oid = remote_ref.target().ok_or_else(|| {
                self.to_bgit_error("Failed to get remote reference target for force-with-lease")
            })?;

            Ok(format!("+{base_refspec}^{{{remote_oid}}}"))
        } else {
            Err(self
                .to_bgit_error("Cannot perform force-with-lease: no remote tracking branch found"))
        }
    }

    fn validate_push_safety(
        &self,
        repo: &Repository,
        head: &git2::Reference,
        branch_name: &str,
    ) -> Result<(), Box<BGitError>> {
        if let Ok(remote_ref) = repo.find_reference(&format!("refs/remotes/origin/{branch_name}")) {
            let local_commit = head
                .peel_to_commit()
                .map_err(|e| self.to_bgit_error(&format!("Failed to get local commit: {e}")))?;

            let remote_commit = remote_ref
                .peel_to_commit()
                .map_err(|e| self.to_bgit_error(&format!("Failed to get remote commit: {e}")))?;

            // If commits are the same, we're up to date
            if local_commit.id() == remote_commit.id() {
                return Ok(());
            }

            // Check if local is behind remote
            let merge_base = repo
                .merge_base(local_commit.id(), remote_commit.id())
                .map_err(|e| self.to_bgit_error(&format!("Failed to find merge base: {e}")))?;

            if merge_base == local_commit.id() && local_commit.id() != remote_commit.id() {
                return Err(self.to_bgit_error("Local branch is behind remote. Pull changes first"));
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
                self.to_bgit_error(&format!("Failed to find local branch {branch_name}: {e}"))
            })?;

        let upstream_name = format!("origin/{branch_name}");
        branch.set_upstream(Some(&upstream_name)).map_err(|e| {
            self.to_bgit_error(&format!("Failed to set upstream to {upstream_name}: {e}"))
        })?;

        Ok(())
    }

    /// Create push options with authentication
    fn create_push_options() -> git2::PushOptions<'static> {
        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(setup_auth_callbacks());
        push_options
    }
}
