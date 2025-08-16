use super::AtomicEvent;
use crate::{bgit_error::BGitError, config::global::BGitGlobalConfig, rules::Rule};
use git2::{BranchType, Repository, StashApplyOptions, StashFlags, build::CheckoutBuilder};
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) enum BranchOperation {
    CheckCurrentBranch,
    MoveChanges,
}

pub(crate) struct GitBranch<'a> {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    operation: Option<BranchOperation>,
    target_branch_name: Option<String>,
    stash_message: Option<String>,
    _global_config: &'a BGitGlobalConfig,
}

impl<'a> GitBranch<'a> {
    pub fn check_current_branch(_global_config: &'a BGitGlobalConfig) -> Self {
        GitBranch {
            name: "git_branch".to_owned(),
            pre_check_rules: vec![],
            operation: Some(BranchOperation::CheckCurrentBranch),
            target_branch_name: None,
            stash_message: None,
            _global_config,
        }
    }

    // New constructor for move changes operation
    pub fn move_changes_to_branch(
        _global_config: &'a BGitGlobalConfig,
        target_branch_name: String,
    ) -> Self {
        GitBranch {
            name: "git_branch".to_owned(),
            pre_check_rules: vec![],
            operation: Some(BranchOperation::MoveChanges),
            target_branch_name: Some(target_branch_name),
            stash_message: Some("Moving changes to new branch".to_string()),
            _global_config,
        }
    }

    pub fn set_stash_message(&mut self, message: String) {
        self.stash_message = Some(message);
    }
}

impl<'a> AtomicEvent<'a> for GitBranch<'a> {
    fn new(_global_config: &'a BGitGlobalConfig) -> Self
    where
        Self: Sized,
    {
        GitBranch {
            name: "git_branch".to_owned(),
            pre_check_rules: vec![],
            operation: None,
            target_branch_name: None,
            stash_message: None,
            _global_config,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        match &self.operation {
            Some(BranchOperation::CheckCurrentBranch) => {
                "Check if current branch is master, main, or dev"
            }
            Some(BranchOperation::MoveChanges) => "Move current changes to a new branch",
            None => "Branch operation (no operation specified)",
        }
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        // Open the repository at the current directory
        let mut repo = Repository::discover(Path::new("."))
            .map_err(|e| self.to_bgit_error(&format!("Failed to open repository: {e}")))?;

        match &self.operation {
            Some(BranchOperation::CheckCurrentBranch) => self.check_current_branch_impl(&repo),
            Some(BranchOperation::MoveChanges) => self.move_changes_impl(&mut repo),
            None => Err(self.to_bgit_error("No branch operation specified")),
        }
    }
}

impl<'a> GitBranch<'a> {
    fn check_current_branch_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
        // Get current branch
        let head_result = repo.head();

        let current_branch_name = match head_result {
            Ok(head) => {
                if !head.is_branch() {
                    return Ok(false);
                }

                let branch_name = head
                    .shorthand()
                    .ok_or_else(|| self.to_bgit_error("Failed to get current branch name"))?;
                branch_name.to_string()
            }
            Err(e) => {
                // Handle unborn branch case (repository with no commits)
                if e.code() == git2::ErrorCode::UnbornBranch {
                    match repo.head_detached() {
                        Ok(false) => {
                            let reference = repo.find_reference("HEAD").map_err(|e| {
                                self.to_bgit_error(&format!(
                                    "Failed to find HEAD reference in unborn branch: {e}"
                                ))
                            })?;

                            let target = reference.symbolic_target().ok_or_else(|| {
                                self.to_bgit_error(
                                    "HEAD reference is not symbolic in unborn branch",
                                )
                            })?;

                            let branch_name =
                                target.strip_prefix("refs/heads/").ok_or_else(|| {
                                    self.to_bgit_error(&format!(
                                        "Invalid HEAD reference format in unborn branch: {target}"
                                    ))
                                })?;

                            branch_name.to_string()
                        }
                        Ok(true) => {
                            return Err(self.to_bgit_error(
                                "Repository is in detached HEAD state with unborn branch",
                            ));
                        }
                        Err(e) => {
                            return Err(self.to_bgit_error(&format!(
                                "Failed to check HEAD detached state: {e}"
                            )));
                        }
                    }
                } else {
                    return Err(self.to_bgit_error(&format!("Failed to get HEAD: {e}")));
                }
            }
        };

        // Check if current branch is one of the main branches
        let is_main_branch = matches!(current_branch_name.as_str(), "master" | "main" | "dev");

        Ok(is_main_branch)
    }

    // Checkout to new a branch and carry forward the current code changes
    fn move_changes_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        let target_branch_name = match &self.target_branch_name {
            Some(name) => name,
            None => {
                return Err(self
                    .to_bgit_error("Target branch name not provided for move changes operation"));
            }
        };

        // Check if target branch already exists
        if repo
            .find_branch(target_branch_name, BranchType::Local)
            .is_ok()
        {
            return Err(self.to_bgit_error(&format!(
                "Target branch '{target_branch_name}' already exists"
            )));
        }

        // Check if there are any changes to move
        if !self.has_changes(repo)? {
            return Err(self.to_bgit_error("No changes found to move to new branch"));
        }

        // Step 1: Save current changes to stash with index
        let stash_message = self
            .stash_message
            .as_deref()
            .unwrap_or("Moving changes to new branch");
        let _stash_id = self.save_changes_to_stash(repo, stash_message)?;

        // Step 2: Create new branch from current HEAD
        let branch_ref_name = {
            let target_commit = {
                match repo.head() {
                    Ok(head) => head.peel_to_commit().map_err(|e| {
                        self.to_bgit_error(&format!("Failed to get target commit: {e}"))
                    })?,
                    Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                        return Err(self.to_bgit_error("Cannot move changes from unborn branch (no commits exist yet). Make your first commit before creating new branches."));
                    }
                    Err(e) => {
                        return Err(self.to_bgit_error(&format!("Failed to get HEAD: {e}")));
                    }
                }
            };

            repo.branch(target_branch_name, &target_commit, false)
                .map_err(|e| {
                    self.to_bgit_error(&format!(
                        "Failed to create branch '{target_branch_name}': {e}"
                    ))
                })?;

            let branch = repo
                .find_branch(target_branch_name, BranchType::Local)
                .map_err(|e| {
                    self.to_bgit_error(&format!(
                        "Failed to find newly created branch '{target_branch_name}': {e}"
                    ))
                })?;

            let branch_ref = branch.get();
            branch_ref.name().unwrap().to_string()
        };

        // Step 3: Checkout the new branch
        repo.set_head(&branch_ref_name)
            .map_err(|e| self.to_bgit_error(&format!("Failed to set HEAD to new branch: {e}")))?;

        repo.checkout_head(Some(CheckoutBuilder::default().force()))
            .map_err(|e| self.to_bgit_error(&format!("Failed to checkout new branch: {e}")))?;

        // Step 4: Pop the stash with checkout strategy to preserve staging
        let mut apply_options = StashApplyOptions::default();
        apply_options.checkout_options(CheckoutBuilder::default());
        // Use reinstantiate_index to preserve the staging state from the stash
        apply_options.reinstantiate_index();

        repo.stash_pop(0, Some(&mut apply_options))
            .map_err(|e| self.to_bgit_error(&format!("Failed to apply stashed changes: {e}")))?;

        Ok(true)
    }
    // Helper method to save changes to stash
    fn save_changes_to_stash(
        &self,
        repo: &mut Repository,
        message: &str,
    ) -> Result<git2::Oid, Box<BGitError>> {
        let signature = repo
            .signature()
            .map_err(|e| self.to_bgit_error(&format!("Failed to get signature: {e}")))?;

        let stash_id = repo
            .stash_save(&signature, message, Some(StashFlags::DEFAULT))
            .map_err(|e| self.to_bgit_error(&format!("Failed to save stash: {e}")))?;

        Ok(stash_id)
    }

    // Helper method to check if there are any changes to move
    fn has_changes(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .map_err(|e| self.to_bgit_error(&format!("Failed to get repository status: {e}")))?;

        // Check if there are any modified, added, deleted, or untracked files
        for entry in statuses.iter() {
            let status = entry.status();
            if status.intersects(
                git2::Status::WT_MODIFIED
                    | git2::Status::WT_DELETED
                    | git2::Status::WT_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED
                    | git2::Status::INDEX_NEW,
            ) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
