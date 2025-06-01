use super::AtomicEvent;
use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_RULE, NO_STEP};
use crate::rules::Rule;
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
            rebase: false,
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
        // Open the repository in the current directory
        let repo = Repository::open(".").map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to open repository: {}", e),
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

        // Get remote reference
        let remote_branch_name = format!("origin/{}", branch_name);
        let remote_ref = repo.find_reference(&remote_branch_name).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!(
                    "Failed to find remote reference {}: {}",
                    remote_branch_name, e
                ),
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

        remote.fetch(&[branch_name], None, None).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to fetch from remote: {}", e),
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
    pub fn set_rebase(&mut self, rebase: bool) -> &mut Self {
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
        // fix unwrap used here
        let head_commit = repo.head().unwrap().peel_to_commit().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get HEAD commit: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Create AnnotatedCommit objects for rebase
        let remote_annotated = repo
            .find_annotated_commit(remote_commit.id())
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to create annotated commit for remote: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        let head_annotated = repo.find_annotated_commit(head_commit.id()).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to create annotated commit for head: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Simple rebase implementation - in a real scenario you'd want more sophisticated rebase handling
        let mut rebase = repo
            .rebase(None, Some(&head_annotated), Some(&remote_annotated), None)
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to start rebase: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        // Process rebase operations
        while let Some(operation) = rebase.next() {
            let _op = operation.map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Rebase operation failed: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;
            // fix unwrap used here

            rebase
                .commit(None, &repo.signature().unwrap(), None)
                .map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to commit during rebase: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_STEP,
                        self.get_name(),
                        NO_RULE,
                    ))
                })?;
        }

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
        // fix unwrap used here

        let head_commit = repo.head().unwrap().peel_to_commit().map_err(|e| {
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
}
