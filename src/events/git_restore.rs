use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::{build::CheckoutBuilder, Repository, ResetType};
use std::path::Path;

pub(crate) struct GitRestore {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    mode: Option<RestoreMode>,
}

#[derive(Debug, Clone)]
pub enum RestoreMode {
    RestoreAllUnstaged,
    UnstageAll,
}

impl AtomicEvent for GitRestore {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitRestore {
            name: "git_restore".to_owned(),
            pre_check_rules: vec![],
            mode: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        "Restore files from staging area or working directory"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        let restore_mode = if let Some(mode) = &self.mode {
            mode
        } else {
            return Err(Box::new(BGitError::new(
                "BGitError",
                "Restore mode not specified for git restore operation",
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            )));
        };
        match restore_mode {
            RestoreMode::RestoreAllUnstaged => self.restore_all_unstaged(),
            RestoreMode::UnstageAll => self.unstage_all_files(),
        }
    }
}

impl GitRestore {
    /// Prompt user to choose between different restore modes
    /// Create a new GitRestore with a specific mode (bypasses user prompt)
    pub fn with_mode(mut self, mode: RestoreMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Restore all unstaged changes (equivalent to `git restore .`)
    fn restore_all_unstaged(&self) -> Result<bool, Box<BGitError>> {
        // Open the repository at the current directory
        let repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to open repository: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Get the index (staging area)
        let mut index = repo.index().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get repository index: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Write the index as a tree
        let index_tree_oid = index.write_tree().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to write index tree: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Get the tree object from the index
        let index_tree = repo.find_tree(index_tree_oid).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to find index tree: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Set up checkout options to force overwrite working directory
        let mut checkout_opts = CheckoutBuilder::new();
        checkout_opts.force(); // This will overwrite working directory files
        checkout_opts.remove_untracked(false); // Don't remove untracked files
        checkout_opts.update_index(false); // Don't update the index

        // Checkout the index tree to working directory
        repo.checkout_tree(index_tree.as_object(), Some(&mut checkout_opts))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to checkout index tree to working directory: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        Ok(true)
    }

    /// Unstage all files (equivalent to `git restore --staged .`)
    fn unstage_all_files(&self) -> Result<bool, Box<BGitError>> {
        // Open the repository at the current directory
        let repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to open repository: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Get HEAD commit
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

        let head_commit = head.peel_to_commit().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get HEAD commit: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Reset index to HEAD (unstage all files)
        repo.reset(head_commit.as_object(), ResetType::Mixed, None)
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to unstage files: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        Ok(true)
    }
}
