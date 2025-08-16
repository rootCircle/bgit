use super::AtomicEvent;
use crate::{bgit_error::BGitError, config::global::BGitGlobalConfig, rules::Rule};
use git2::{Repository, ResetType, build::CheckoutBuilder};
use std::path::Path;

pub(crate) struct GitRestore<'a> {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    mode: Option<RestoreMode>,
    _global_config: &'a BGitGlobalConfig,
}

#[derive(Debug, Clone)]
pub enum RestoreMode {
    RestoreAllUnstaged,
    UnstageAll,
}

impl<'a> AtomicEvent<'a> for GitRestore<'a> {
    fn new(_global_config: &'a BGitGlobalConfig) -> Self
    where
        Self: Sized,
    {
        GitRestore {
            name: "git_restore".to_owned(),
            pre_check_rules: vec![],
            mode: None,
            _global_config,
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
            return Err(self.to_bgit_error("Restore mode not specified for git restore operation"));
        };
        match restore_mode {
            RestoreMode::RestoreAllUnstaged => self.restore_all_unstaged(),
            RestoreMode::UnstageAll => self.unstage_all_files(),
        }
    }
}

impl<'a> GitRestore<'a> {
    /// Prompt user to choose between different restore modes
    /// Create a new GitRestore with a specific mode (bypasses user prompt)
    pub fn with_mode(mut self, mode: RestoreMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Restore all unstaged changes (equivalent to `git restore .`)
    fn restore_all_unstaged(&self) -> Result<bool, Box<BGitError>> {
        // Open the repository at the current directory
        let repo = Repository::discover(Path::new("."))
            .map_err(|e| self.to_bgit_error(&format!("Failed to open repository: {e}")))?;

        // Get the index (staging area)
        let mut index = repo
            .index()
            .map_err(|e| self.to_bgit_error(&format!("Failed to get repository index: {e}")))?;

        // Write the index as a tree
        let index_tree_oid = index
            .write_tree()
            .map_err(|e| self.to_bgit_error(&format!("Failed to write index tree: {e}")))?;

        // Get the tree object from the index
        let index_tree = repo
            .find_tree(index_tree_oid)
            .map_err(|e| self.to_bgit_error(&format!("Failed to find index tree: {e}")))?;

        // Set up checkout options to force overwrite working directory
        let mut checkout_opts = CheckoutBuilder::new();
        checkout_opts.force(); // This will overwrite working directory files
        checkout_opts.remove_untracked(false); // Don't remove untracked files
        checkout_opts.update_index(false); // Don't update the index

        // Checkout the index tree to working directory
        repo.checkout_tree(index_tree.as_object(), Some(&mut checkout_opts))
            .map_err(|e| {
                self.to_bgit_error(&format!(
                    "Failed to checkout index tree to working directory: {e}"
                ))
            })?;

        Ok(true)
    }

    /// Unstage all files (equivalent to `git restore --staged .`)
    fn unstage_all_files(&self) -> Result<bool, Box<BGitError>> {
        // Open the repository at the current directory
        let repo = Repository::discover(Path::new("."))
            .map_err(|e| self.to_bgit_error(&format!("Failed to open repository: {e}")))?;

        // Get HEAD commit - handle unborn branch case
        let head_commit = match repo.head() {
            Ok(head) => head
                .peel_to_commit()
                .map_err(|e| self.to_bgit_error(&format!("Failed to get HEAD commit: {e}")))?,
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                return Err(self.to_bgit_error("Cannot restore staged files in unborn branch (no commits exist yet). Use 'git reset' or remove files from staging manually."));
            }
            Err(e) => {
                return Err(self.to_bgit_error(&format!("Failed to get HEAD: {e}")));
            }
        };

        // Reset index to HEAD (unstage all files)
        repo.reset(head_commit.as_object(), ResetType::Mixed, None)
            .map_err(|e| self.to_bgit_error(&format!("Failed to unstage files: {e}")))?;

        Ok(true)
    }
}
