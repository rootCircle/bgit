use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::{BranchType, Repository, StashApplyOptions, StashFlags};
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) enum BranchOperation {
    #[allow(dead_code)]
    Create,
    #[allow(dead_code)]
    Delete,
    List,
    CheckCurrentBranch,
    MoveChanges,
}

pub(crate) struct GitBranch {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    branch_name: Option<String>,
    operation: BranchOperation,
    force_delete: bool,
    // New fields for move changes operation
    target_branch_name: Option<String>,
    stash_message: Option<String>,
}

impl GitBranch {
    // pub fn create_branch(branch_name: String) -> Self {
    //     GitBranch {
    //         name: "git_branch".to_owned(),
    //         pre_check_rules: vec![],
    //         branch_name: Some(branch_name),
    //         operation: BranchOperation::Create,
    //         force_delete: false,
    //         target_branch_name: None,
    //         stash_message: None,
    //     }
    // }

    // pub fn delete_branch(branch_name: String) -> Self {
    //     GitBranch {
    //         name: "git_branch".to_owned(),
    //         pre_check_rules: vec![],
    //         branch_name: Some(branch_name),
    //         operation: BranchOperation::Delete,
    //         force_delete: false,
    //         target_branch_name: None,
    //         stash_message: None,
    //     }
    // }

    // pub fn force_delete_branch(branch_name: String) -> Self {
    //     GitBranch {
    //         name: "git_branch".to_owned(),
    //         pre_check_rules: vec![],
    //         branch_name: Some(branch_name),
    //         operation: BranchOperation::Delete,
    //         force_delete: true,
    //         target_branch_name: None,
    //         stash_message: None,
    //     }
    // }

    // pub fn list_branches() -> Self {
    //     GitBranch {
    //         name: "git_branch".to_owned(),
    //         pre_check_rules: vec![],
    //         branch_name: None,
    //         operation: BranchOperation::List,
    //         force_delete: false,
    //         target_branch_name: None,
    //         stash_message: None,
    //     }
    // }

    pub fn check_current_branch() -> Self {
        GitBranch {
            name: "git_branch".to_owned(),
            pre_check_rules: vec![],
            branch_name: None,
            operation: BranchOperation::CheckCurrentBranch,
            force_delete: false,
            target_branch_name: None,
            stash_message: None,
        }
    }

    // New constructor for move changes operation
    pub fn move_changes_to_branch(target_branch_name: String) -> Self {
        GitBranch {
            name: "git_branch".to_owned(),
            pre_check_rules: vec![],
            branch_name: None,
            operation: BranchOperation::MoveChanges,
            force_delete: false,
            target_branch_name: Some(target_branch_name),
            stash_message: Some("Moving changes to new branch".to_string()),
        }
    }

    // pub fn set_branch_name(&mut self, branch_name: String) {
    //     self.branch_name = Some(branch_name);
    // }

    // pub fn set_operation(&mut self, operation: BranchOperation) {
    //     self.operation = operation;
    // }

    // pub fn set_force_delete(&mut self, force: bool) {
    //     self.force_delete = force;
    // }

    // // New setter methods for move changes operation
    // pub fn set_target_branch_name(&mut self, target_branch_name: String) {
    //     self.target_branch_name = Some(target_branch_name);
    // }

    pub fn set_stash_message(&mut self, message: String) {
        self.stash_message = Some(message);
    }
}

impl AtomicEvent for GitBranch {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitBranch {
            name: "git_branch".to_owned(),
            pre_check_rules: vec![],
            branch_name: None,
            operation: BranchOperation::List,
            force_delete: false,
            target_branch_name: None,
            stash_message: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        match self.operation {
            BranchOperation::Create => "Create a new branch",
            BranchOperation::Delete => "Delete a branch",
            BranchOperation::List => "List all branches",
            BranchOperation::CheckCurrentBranch => {
                "Check if current branch is master, main, or dev"
            }
            BranchOperation::MoveChanges => "Move current changes to a new branch",
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
        let mut repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to open repository: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        match self.operation {
            BranchOperation::Create => self.create_branch_impl(&repo),
            BranchOperation::Delete => self.delete_branch_impl(&repo),
            BranchOperation::List => self.list_branches_impl(&repo),
            BranchOperation::CheckCurrentBranch => self.check_current_branch_impl(&repo),
            BranchOperation::MoveChanges => self.move_changes_impl(&mut repo),
        }
    }
}

impl GitBranch {
    fn create_branch_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
        // Check if branch name is provided
        let branch_name = match &self.branch_name {
            Some(name) => name,
            None => {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    "Branch name not provided for create operation",
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                )));
            }
        };

        // Get the current HEAD commit
        let head = repo.head().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get HEAD: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        let target_commit = head.peel_to_commit().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get target commit: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Check if branch already exists
        match repo.find_branch(branch_name, BranchType::Local) {
            Ok(_) => {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    &format!("Branch '{}' already exists", branch_name),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                )));
            }
            Err(_) => {
                // Branch doesn't exist, which is what we want
            }
        }

        // Create the new branch
        repo.branch(branch_name, &target_commit, false)
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to create branch '{}': {}", branch_name, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        println!("Created branch '{}'", branch_name);
        Ok(true)
    }

    fn delete_branch_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
        // Check if branch name is provided
        let branch_name = match &self.branch_name {
            Some(name) => name,
            None => {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    "Branch name not provided for delete operation",
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                )));
            }
        };

        // Get current HEAD to prevent deleting the current branch
        let current_branch = match repo.head() {
            Ok(head) => {
                if head.is_branch() {
                    head.shorthand().map(|s| s.to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        };

        // Check if trying to delete the current branch
        if let Some(ref current) = current_branch {
            if current == branch_name {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    &format!(
                        "Cannot delete the currently checked out branch '{}'",
                        branch_name
                    ),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                )));
            }
        }

        // Find the branch to delete
        let mut branch = repo
            .find_branch(branch_name, BranchType::Local)
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Branch '{}' not found: {}", branch_name, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        // Check if branch is merged (unless force delete is enabled)
        if !self.force_delete {
            let is_merged = self.is_branch_merged(repo, &branch)?;
            if !is_merged {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    &format!(
                        "Branch '{}' is not fully merged. Use force delete to override.",
                        branch_name
                    ),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                )));
            }
        }

        // Delete the branch
        branch.delete().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to delete branch '{}': {}", branch_name, e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        println!("Deleted branch '{}'", branch_name);
        Ok(true)
    }

    fn list_branches_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
        // Get current branch for marking
        let current_branch = match repo.head() {
            Ok(head) => {
                if head.is_branch() {
                    head.shorthand().map(|s| s.to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        };

        // Get all local branches
        let branches = repo.branches(Some(BranchType::Local)).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to list branches: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        println!("Local branches:");
        let mut branch_count = 0;

        for branch_result in branches {
            let (branch, _branch_type) = branch_result.map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to process branch: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

            if let Some(branch_name) = branch.name().map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get branch name: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })? {
                let marker = if Some(branch_name.to_string()) == current_branch {
                    "* "
                } else {
                    "  "
                };
                println!("{}{}", marker, branch_name);
                branch_count += 1;
            }
        }

        if branch_count == 0 {
            println!("No local branches found.");
        }

        Ok(true)
    }

    fn check_current_branch_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
        // Get current branch
        let head = repo.head().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get HEAD: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        if !head.is_branch() {
            println!("Currently in detached HEAD state (not on any branch)");
            return Ok(false);
        }

        let current_branch_name = head.shorthand().ok_or_else(|| {
            Box::new(BGitError::new(
                "BGitError",
                "Failed to get current branch name",
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        println!("Current branch: {}", current_branch_name);

        // Check if current branch is one of the main branches
        let is_main_branch = match current_branch_name {
            "master" | "main" | "dev" => {
                println!("✓ Currently on a main branch ({})", current_branch_name);
                true
            }
            _ => {
                println!("✗ Not on a main branch (master, main, or dev)");
                false
            }
        };

        Ok(is_main_branch)
    }

    // New implementation for move changes operation
    fn move_changes_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        let target_branch_name = match &self.target_branch_name {
            Some(name) => name,
            None => {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    "Target branch name not provided for move changes operation",
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                )));
            }
        };

        // Check if target branch already exists
        if repo.find_branch(target_branch_name, BranchType::Local).is_ok() {
            return Err(Box::new(BGitError::new(
                "BGitError",
                &format!("Target branch '{}' already exists", target_branch_name),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            )));
        }

        // Check if there are any changes to move
        if !self.has_changes(repo)? {
            return Err(Box::new(BGitError::new(
                "BGitError",
                "No changes found to move to new branch",
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            )));
        }

        // Step 1: Save current changes to stash
        let stash_message = self
            .stash_message
            .as_deref()
            .unwrap_or("Moving changes to new branch");
        let _stash_id = self.save_changes_to_stash(repo, stash_message)?;
        println!("Saved changes to stash: {}", stash_message);

        // Step 2: Create new branch from current HEAD
        // Limit the scope of target_commit to release the immutable borrow
        let branch_ref_name = {
            let target_commit = {
                let head = repo.head().map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to get HEAD: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_EVENT,
                        &self.name,
                        NO_RULE,
                    ))
                })?;

                head.peel_to_commit().map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to get target commit: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_EVENT,
                        &self.name,
                        NO_RULE,
                    ))
                })?
            };

            repo.branch(target_branch_name, &target_commit, false)
                .map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to create branch '{}': {}", target_branch_name, e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_EVENT,
                        &self.name,
                        NO_RULE,
                    ))
                })?;

            println!("Created branch '{}'", target_branch_name);

            // Get the branch reference name
            let branch = repo
                .find_branch(target_branch_name, BranchType::Local)
                .map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!(
                            "Failed to find newly created branch '{}': {}",
                            target_branch_name, e
                        ),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_EVENT,
                        &self.name,
                        NO_RULE,
                    ))
                })?;

            let branch_ref = branch.get();
            branch_ref.name().unwrap().to_string()
            // target_commit is dropped here, releasing the immutable borrow
        };

        // Step 3: Checkout the new branch
        repo.set_head(&branch_ref_name).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to set HEAD to new branch: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to checkout new branch: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        println!("Switched to branch '{}'", target_branch_name);

        // Step 4: Pop the stash to apply changes to the new branch
        // Now we can safely mutably borrow repo because target_commit has been dropped
        let mut apply_options = StashApplyOptions::default();
        repo.stash_pop(0, Some(&mut apply_options)).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to apply stashed changes: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        println!("Applied changes to branch '{}'", target_branch_name);
        println!(
            "Successfully moved changes to new branch '{}'",
            target_branch_name
        );

        Ok(true)
    }
    // Helper method to save changes to stash
    fn save_changes_to_stash(
        &self,
        repo: &mut Repository,
        message: &str,
    ) -> Result<git2::Oid, Box<BGitError>> {
        let signature = repo.signature().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get signature: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        let stash_id = repo
            .stash_save(&signature, message, Some(StashFlags::DEFAULT))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to save stash: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        Ok(stash_id)
    }

    // Helper method to check if there are any changes to move
    fn has_changes(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);

        let statuses = repo.statuses(Some(&mut status_options)).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get repository status: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

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

    fn is_branch_merged(
        &self,
        repo: &Repository,
        branch: &git2::Branch,
    ) -> Result<bool, Box<BGitError>> {
        // Get the branch commit
        let branch_commit = branch.get().peel_to_commit().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get branch commit: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Get HEAD commit
        let head_commit = repo
            .head()
            .and_then(|head| head.peel_to_commit())
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get HEAD commit: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        // Check if branch commit is ancestor of HEAD (i.e., merged)
        let is_ancestor = repo
            .graph_descendant_of(head_commit.id(), branch_commit.id())
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to check merge status: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        Ok(is_ancestor)
    }
}
