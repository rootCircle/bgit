use super::AtomicEvent;
use crate::auth::git_auth::setup_auth_callbacks;
use crate::bgit_error::BGitError;
use crate::config::global::BGitGlobalConfig;
use crate::rules::Rule;
use git2::{Oid, Repository};
use log::{debug, info};
use std::path::Path;

pub struct GitPush<'a> {
    pub pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    pub force_with_lease: bool,
    pub set_upstream: bool,
    pub _global_config: &'a BGitGlobalConfig,
}

impl<'a> AtomicEvent<'a> for GitPush<'a> {
    fn new(_global_config: &'a BGitGlobalConfig) -> Self
    where
        Self: Sized,
    {
        GitPush {
            pre_check_rules: vec![],
            force_with_lease: false,
            set_upstream: false,
            _global_config,
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

        // Determine which remote to use (prefer branch upstream > single remote > 'origin')
        let remote_name = self
            .determine_remote_name(&repo, &branch_name)
            .map_err(|e| self.to_bgit_error(&e.to_string()))?;

        // Get remote - handle case where no remote is configured
        let mut remote = repo.find_remote(&remote_name).map_err(|e| {
            self.to_bgit_error(&format!("Failed to find remote '{remote_name}': {e}"))
        })?;

        // Prepare push options with authentication and callbacks
        let mut push_options = Self::create_push_options();

        if self.force_with_lease {
            // Best-effort native force-with-lease emulation with libgit2:
            // 1) Capture expected remote OID from tracking ref before fetching
            let tracking_ref = format!("refs/remotes/{remote_name}/{branch_name}");
            let expected_remote_oid = repo
                .refname_to_id(&tracking_ref)
                .unwrap_or_else(|_| Oid::zero());

            // 2) Fetch latest state for the branch to update tracking ref
            let mut fetch_opts = git2::FetchOptions::new();
            fetch_opts.remote_callbacks(setup_auth_callbacks());
            let fetch_refspec = format!(
                "refs/heads/{0}:refs/remotes/{1}/{0}",
                branch_name, remote_name
            );
            remote
                .fetch(&[fetch_refspec], Some(&mut fetch_opts), None)
                .map_err(|e| self.to_bgit_error(&format!("Failed to fetch from remote: {e}")))?;

            // 3) Compare actual vs expected; if diverged, abort
            let actual_remote_oid = repo
                .refname_to_id(&tracking_ref)
                .unwrap_or_else(|_| Oid::zero());
            if actual_remote_oid != expected_remote_oid {
                return Err(self.to_bgit_error(&format!(
                    "Lease broken: remote '{remote_name}/{branch_name}' is at {actual_remote_oid}, expected {expected_remote_oid}. Aborting push."
                )));
            }

            // 4) Lease holds — perform forced update
            let refspec = if self.set_upstream {
                format!("+refs/heads/{branch_name}:refs/heads/{branch_name}")
            } else {
                format!("+refs/heads/{branch_name}")
            };

            remote.push(&[refspec], Some(&mut push_options)).map_err(|e| {
                let transport_hint = self.transport_hint(remote.url());
                self.to_bgit_error(&format!(
                    "Failed to push to remote {transport_hint} (force-with-lease): {e}. If authentication is required, ensure your credentials are set up."
                ))
            })?;
        } else {
            // Pre-flight safety check for regular push
            self.validate_push_safety(&repo, &head, &branch_name)?;

            let refspec = if self.set_upstream {
                format!("refs/heads/{branch_name}:refs/heads/{branch_name}")
            } else {
                format!("refs/heads/{branch_name}")
            };

            remote.push(&[refspec], Some(&mut push_options)).map_err(|e| {
                let transport_hint = self.transport_hint(remote.url());
                self.to_bgit_error(&format!(
                    "Failed to push to remote {transport_hint}: {e}. If authentication is required, ensure your credentials are set up."
                ))
            })?;
        }

        // Set upstream if requested or if there is no upstream yet
        if self.set_upstream || !self.has_upstream(&repo, &branch_name)? {
            self.set_upstream_branch(&repo, &remote_name, &branch_name)?;
            info!("Set upstream to {remote_name}/{branch_name}");
        }

        Ok(true)
    }
}

impl<'a> GitPush<'a> {
    pub fn with_force_with_lease(&mut self, force_with_lease: bool) -> &mut Self {
        self.force_with_lease = force_with_lease;
        self
    }

    pub fn with_upstream_flag(&mut self, set_upstream: bool) -> &mut Self {
        self.set_upstream = set_upstream;
        self
    }

    fn validate_push_safety(
        &self,
        repo: &Repository,
        head: &git2::Reference,
        branch_name: &str,
    ) -> Result<(), Box<BGitError>> {
        let remote_name = self
            .determine_remote_name(repo, branch_name)
            .unwrap_or_else(|_| String::from("origin"));
        if let Ok(remote_ref) =
            repo.find_reference(&format!("refs/remotes/{remote_name}/{branch_name}"))
        {
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
                return Err(
                    self.to_bgit_error("Local branch is behind remote. Pull changes first.")
                );
            }
        }

        Ok(())
    }

    fn set_upstream_branch(
        &self,
        repo: &Repository,
        remote_name: &str,
        branch_name: &str,
    ) -> Result<(), Box<BGitError>> {
        let mut branch = repo
            .find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| {
                self.to_bgit_error(&format!("Failed to find local branch {branch_name}: {e}"))
            })?;

        let upstream_name = format!("{remote_name}/{branch_name}");
        branch.set_upstream(Some(&upstream_name)).map_err(|e| {
            self.to_bgit_error(&format!("Failed to set upstream to {upstream_name}: {e}"))
        })?;

        Ok(())
    }

    fn has_upstream(&self, repo: &Repository, branch_name: &str) -> Result<bool, Box<BGitError>> {
        let branch = repo
            .find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| {
                self.to_bgit_error(&format!("Failed to find local branch {branch_name}: {e}"))
            })?;
        Ok(branch.upstream().is_ok())
    }

    // Determine the remote to use for pushes: prefer branch upstream remote, else if exactly one remote exists, use it, else try 'origin', else error.
    fn determine_remote_name(
        &self,
        repo: &Repository,
        branch_name: &str,
    ) -> Result<String, String> {
        // Try branch upstream
        if let Ok(branch) = repo.find_branch(branch_name, git2::BranchType::Local)
            && let Ok(upstream) = branch.upstream()
            && let Some(name) = upstream.get().name()
        {
            // name like: refs/remotes/<remote>/<branch>
            let parts: Vec<&str> = name.split('/').collect();
            if parts.len() >= 4 && parts[0] == "refs" && parts[1] == "remotes" {
                return Ok(parts[2].to_string());
            }
        }

        // If exactly one remote is configured, use it
        if let Ok(remotes) = repo.remotes() {
            if remotes.len() == 1
                && let Some(r) = remotes.get(0)
            {
                return Ok(r.to_string());
            }
            // If 'origin' exists, prefer it
            for i in 0..remotes.len() {
                if let Some(r) = remotes.get(i)
                    && r == "origin"
                {
                    return Ok("origin".to_string());
                }
            }
        }

        Err("No suitable remote configured. Add a remote or set an upstream (git branch --set-upstream-to <remote>/<branch>).".to_string())
    }

    /// Create push options with authentication
    fn create_push_options() -> git2::PushOptions<'static> {
        let mut push_options = git2::PushOptions::new();
        let mut callbacks = setup_auth_callbacks();
        // Surface ref update errors clearly during push
        callbacks.push_update_reference(|refname, status| match status {
            Some(msg) => {
                debug!("Push failed for {refname}: {msg}");
                Err(git2::Error::from_str(msg))
            }
            None => {
                debug!("Push successful for {refname}");
                Ok(())
            }
        });
        push_options.remote_callbacks(callbacks);
        push_options
    }

    fn transport_hint(&self, url_opt: Option<&str>) -> &'static str {
        if let Some(u) = url_opt {
            if u.starts_with("http://") || u.starts_with("https://") {
                "(HTTPS) - check token/credentials"
            } else if u.starts_with("ssh://")
                || u.starts_with("git@")
                || (u.contains('@') && u.contains(':'))
            {
                "(SSH) - check keys/agent"
            } else {
                ""
            }
        } else {
            ""
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::global::BGitGlobalConfig;
    use git2::Signature;
    use std::fs;
    use tempfile::TempDir;

    fn init_repo_with_commit() -> (TempDir, Repository, String) {
        let td = TempDir::with_prefix("bgit_unit_").unwrap();
        let repo = Repository::init(td.path()).unwrap();

        // Configure user
        repo.config().unwrap().set_str("user.name", "Test").unwrap();
        repo.config()
            .unwrap()
            .set_str("user.email", "test@example.com")
            .unwrap();

        // Create initial commit on main
        fs::write(td.path().join("README.md"), b"hello").unwrap();
        let mut idx = repo.index().unwrap();
        idx.add_path(Path::new("README.md")).unwrap();
        idx.write().unwrap();
        let tree_id = idx.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let _ = repo
            .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();
        drop(tree);
        // Ensure branch name exists
        let branch_name = {
            let head_ref = repo.head().unwrap();
            head_ref.shorthand().unwrap().to_string()
        };
        (td, repo, branch_name)
    }

    #[test]
    fn determine_remote_prefers_upstream() {
        let (_td, repo, branch) = init_repo_with_commit();
        // add two remotes
        repo.remote("foo", "https://example.com/foo.git").unwrap();
        repo.remote("origin", "https://example.com/origin.git")
            .unwrap();

        // Simulate upstream to foo/<branch> by creating the tracking ref
        let head_id = repo.head().unwrap().target().unwrap();
        repo.reference(&format!("refs/remotes/foo/{branch}"), head_id, true, "test")
            .unwrap();

        // Also set branch upstream in config
        repo.config()
            .unwrap()
            .set_str(&format!("branch.{branch}.remote"), "foo")
            .unwrap();
        repo.config()
            .unwrap()
            .set_str(
                &format!("branch.{branch}.merge"),
                &format!("refs/heads/{branch}"),
            )
            .unwrap();

        let global = BGitGlobalConfig::default();
        let gp = GitPush::new(&global);
        let chosen = gp.determine_remote_name(&repo, &branch).unwrap();
        assert_eq!(chosen, "foo");
    }

    #[test]
    fn expected_remote_oid_uses_remote_name() {
        let (_td, repo, branch) = init_repo_with_commit();
        repo.remote("foo", "https://example.com/foo.git").unwrap();

        // Create tracking ref for foo/<branch> pointing to HEAD
        let head_id = repo.head().unwrap().target().unwrap();
        repo.reference(&format!("refs/remotes/foo/{branch}"), head_id, true, "test")
            .unwrap();

        let tracking = format!("refs/remotes/foo/{branch}");
        let oid = repo.refname_to_id(&tracking).unwrap();
        assert_eq!(oid, head_id);
    }

    #[test]
    fn has_upstream_checks_presence() {
        let (_td, repo, branch) = init_repo_with_commit();
        repo.remote("foo", "https://example.com/foo.git").unwrap();

        let global = BGitGlobalConfig::default();
        let gp = GitPush::new(&global);
        // Initially no upstream
        assert!(!gp.has_upstream(&repo, &branch).unwrap());

        // Set upstream to foo/branch
        // Ensure the tracking reference exists for the remote branch
        let head_id = repo.head().unwrap().target().unwrap();
        repo.reference(&format!("refs/remotes/foo/{branch}"), head_id, true, "test")
            .unwrap();
        let mut br = repo.find_branch(&branch, git2::BranchType::Local).unwrap();
        br.set_upstream(Some(&format!("foo/{branch}"))).unwrap();
        assert!(gp.has_upstream(&repo, &branch).unwrap());
    }
}
