use std::path::Path;

use super::AtomicEvent;
use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_RULE, NO_STEP};
use crate::rules::Rule;
use crate::utils::git_auth::setup_auth_callbacks;
use git2::Repository;

pub struct GitPull {
    pub pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    pub rebase: bool,
}

impl AtomicEvent for GitPull {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitPull {
            pre_check_rules: vec![],
            rebase: true,
        }
    }

    fn get_name(&self) -> &str {
        "git_pull"
    }

    fn get_action_description(&self) -> &str {
        "Pull changes from remote repository"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }
    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        // Discover the repository starting from the current directory
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

        // Fetch from remote first
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

        // Set up fetch options with authentication
        let mut fetch_options = Self::create_fetch_options();

        // Fetch all references to ensure we have the latest remote state
        remote.fetch(&[&"refs/heads/*:refs/remotes/origin/*".to_string()], Some(&mut fetch_options), None).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to fetch from remote: {}. Please check your SSH keys or authentication setup.", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Try to find the remote reference with better error handling
        let remote_branch_name = format!("refs/remotes/origin/{}", branch_name);
        let remote_ref = repo
            .find_reference(&remote_branch_name)
            .or_else(|_| {
                // If the exact branch name doesn't exist, try common alternatives
                let alternatives = vec![
                    format!("refs/remotes/origin/main"),
                    format!("refs/remotes/origin/master"),
                    format!("refs/remotes/origin/develop"),
                ];

                for alt in alternatives {
                    if let Ok(reference) = repo.find_reference(&alt) {
                        return Ok(reference);
                    }
                }

                // If no alternatives work, check what remote branches actually exist
                let remote_branches: Vec<String> = repo
                    .branches(Some(git2::BranchType::Remote))
                    .map_err(|e| format!("Failed to list remote branches: {}", e))
                    .unwrap()
                    .filter_map(|branch_result| {
                        branch_result.ok().and_then(|(branch, _)| {
                            branch.name().ok().flatten().map(|name| name.to_string())
                        })
                    })
                    .collect();

                Err(git2::Error::new(
                    git2::ErrorCode::NotFound,
                    git2::ErrorClass::Reference,
                    format!(
                        "Remote branch 'origin/{}' not found. Available remote branches: {:?}",
                        branch_name, remote_branches
                    ),
                ))
            })
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to find remote reference: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        if self.rebase {
            self.execute_rebase(&repo, &remote_ref)?;
        } else {
            self.execute_merge(&repo, &remote_ref)?;
        }

        Ok(true)
    }
}

impl GitPull {
    pub fn with_rebase(mut self, rebase: bool) -> Self {
        self.rebase = rebase;
        self
    }

    fn execute_rebase(
        &self,
        repo: &Repository,
        remote_ref: &git2::Reference,
    ) -> Result<(), Box<BGitError>> {
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

        let head_commit = repo
            .head()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get HEAD reference: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?
            .peel_to_commit()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get HEAD commit: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        // Check if we're already up to date
        if head_commit.id() == remote_commit.id() {
            return Ok(());
        }

        // Check if remote is ancestor of head (nothing to rebase)
        let merge_base = repo
            .merge_base(head_commit.id(), remote_commit.id())
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

        if merge_base == remote_commit.id() {
            return Ok(());
        }

        // Create AnnotatedCommit objects for rebase
        // Note: The branch we want to rebase (our current branch) should be 'branch'
        // The upstream/onto target should be 'upstream'
        let upstream_annotated = repo
            .find_annotated_commit(remote_commit.id())
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to create annotated commit for upstream: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        // For rebase, we typically want to rebase current HEAD onto the remote
        // Parameters: branch, upstream, onto, opts
        // branch: what to rebase (None means current HEAD)
        // upstream: the upstream branch
        // onto: where to rebase onto (None means same as upstream)
        let mut rebase = repo
            .rebase(None, Some(&upstream_annotated), None, None)
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to start rebase: {}. This might indicate conflicts or uncommitted changes.", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        // Process rebase operations
        let mut operation_count = 0;
        while let Some(_) = rebase.next() {
            operation_count += 1;

            // Check if there are conflicts
            let index = repo.index().map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get repository index: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

            if index.has_conflicts() {
                // Abort the rebase to prevent data loss
                rebase.abort().map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to abort rebase after conflicts: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_STEP,
                        self.get_name(),
                        NO_RULE,
                    ))
                })?;

                return Err(Box::new(BGitError::new(
                    "BGitError",
                    "Rebase conflicts detected. The rebase has been aborted to prevent data loss. Please resolve conflicts manually and retry.",
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                )));
            }

            // Get signature for committing
            let signature = repo.signature().map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get signature: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

            // Commit the rebased operation
            let _commit_id = rebase.commit(None, &signature, None).map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!(
                        "Failed to commit during rebase operation {}: {}",
                        operation_count, e
                    ),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;
        }

        // Finish the rebase
        rebase.finish(None).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to finish rebase: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        Ok(())
    }

    fn execute_merge(
        &self,
        repo: &Repository,
        remote_ref: &git2::Reference,
    ) -> Result<(), Box<BGitError>> {
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

        // Fix unwrap here
        let head_commit = repo
            .head()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get HEAD reference: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?
            .peel_to_commit()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get HEAD commit: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        // Check if we're already up to date
        if head_commit.id() == remote_commit.id() {
            return Ok(());
        }

        // Perform merge
        let merge_base = repo
            .merge_base(head_commit.id(), remote_commit.id())
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

        // If remote commit is ancestor of head, we're already up to date
        if merge_base == remote_commit.id() {
            return Ok(());
        }

        // If head is ancestor of remote, fast-forward
        if merge_base == head_commit.id() {
            let mut head_ref = repo.head().map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get HEAD reference: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

            head_ref
                .set_target(remote_commit.id(), "Fast-forward merge")
                .map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to fast-forward: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_STEP,
                        self.get_name(),
                        NO_RULE,
                    ))
                })?;

            // Update working directory
            repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                .map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to checkout after fast-forward: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_STEP,
                        self.get_name(),
                        NO_RULE,
                    ))
                })?;
        } else {
            return Err(Box::new(BGitError::new(
                "BGitError",
                "Merge conflicts detected - manual resolution required",
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            )));
        }

        Ok(())
    }

    /// Create fetch options with authentication
    fn create_fetch_options() -> git2::FetchOptions<'static> {
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(setup_auth_callbacks());
        fetch_options
    }
}
