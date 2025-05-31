use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::{Repository, StashApplyOptions, StashFlags};
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) enum StashOperation {
    #[allow(dead_code)]
    Save,
    Pop,
    #[allow(dead_code)]
    Apply,
    List,
    #[allow(dead_code)]
    Drop,
    #[allow(dead_code)]
    Clear,
}

pub(crate) struct GitStash {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    operation: StashOperation,
    stash_message: Option<String>,
    stash_index: Option<usize>,
    include_untracked: bool,
    keep_index: bool,
}

impl GitStash {
    #[allow(dead_code)]
    pub fn save_stash(message: Option<String>) -> Self {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: StashOperation::Save,
            stash_message: message,
            stash_index: None,
            include_untracked: false,
            keep_index: false,
        }
    }

    pub fn pop_stash(index: Option<usize>) -> Self {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: StashOperation::Pop,
            stash_message: None,
            stash_index: index,
            include_untracked: false,
            keep_index: false,
        }
    }
    #[allow(dead_code)]
    pub fn apply_stash(index: Option<usize>) -> Self {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: StashOperation::Apply,
            stash_message: None,
            stash_index: index,
            include_untracked: false,
            keep_index: false,
        }
    }
    #[allow(dead_code)]
    pub fn list_stashes() -> Self {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: StashOperation::List,
            stash_message: None,
            stash_index: None,
            include_untracked: false,
            keep_index: false,
        }
    }
    #[allow(dead_code)]
    pub fn drop_stash(index: Option<usize>) -> Self {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: StashOperation::Drop,
            stash_message: None,
            stash_index: index,
            include_untracked: false,
            keep_index: false,
        }
    }
    #[allow(dead_code)]
    pub fn clear_stashes() -> Self {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: StashOperation::Clear,
            stash_message: None,
            stash_index: None,
            include_untracked: false,
            keep_index: false,
        }
    }
    #[allow(dead_code)]
    pub fn set_message(&mut self, message: String) {
        self.stash_message = Some(message);
    }
    #[allow(dead_code)]
    pub fn set_index(&mut self, index: usize) {
        self.stash_index = Some(index);
    }
    #[allow(dead_code)]
    pub fn set_include_untracked(&mut self, include: bool) {
        self.include_untracked = include;
    }
    #[allow(dead_code)]
    pub fn set_keep_index(&mut self, keep: bool) {
        self.keep_index = keep;
    }
}

impl AtomicEvent for GitStash {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: StashOperation::List,
            stash_message: None,
            stash_index: None,
            include_untracked: false,
            keep_index: false,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        match self.operation {
            StashOperation::Save => "Save changes to stash",
            StashOperation::Pop => "Pop stash and apply changes",
            StashOperation::Apply => "Apply stash without removing it",
            StashOperation::List => "List all stashes",
            StashOperation::Drop => "Drop a stash",
            StashOperation::Clear => "Clear all stashes",
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
            StashOperation::Save => self.save_stash_impl(&mut repo),
            StashOperation::Pop => self.pop_stash_impl(&mut repo),
            StashOperation::Apply => self.apply_stash_impl(&mut repo),
            StashOperation::List => self.list_stashes_impl(&mut repo),
            StashOperation::Drop => self.drop_stash_impl(&mut repo),
            StashOperation::Clear => self.clear_stashes_impl(&mut repo),
        }
    }
}

impl GitStash {
    fn save_stash_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        // Get the signature for the stash
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

        // Prepare stash flags
        let mut flags = StashFlags::DEFAULT;
        if self.include_untracked {
            flags |= StashFlags::INCLUDE_UNTRACKED;
        }
        if self.keep_index {
            flags |= StashFlags::KEEP_INDEX;
        }

        // Create stash message
        let message = self.stash_message.as_deref().unwrap_or("WIP");

        // Save the stash
        let stash_id = repo
            .stash_save(&signature, message, Some(flags))
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

        println!("Saved working directory and index state: {}", message);
        println!("Stash ID: {}", stash_id);
        Ok(true)
    }

    fn pop_stash_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        let index = self.stash_index.unwrap_or(0);

        // Check if stash exists
        self.check_stash_exists(repo, index)?;

        let mut apply_options = StashApplyOptions::default();

        repo.stash_pop(index, Some(&mut apply_options))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to pop stash at index {}: {}", index, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        println!("Popped stash at index {}", index);
        Ok(true)
    }

    fn apply_stash_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        let index = self.stash_index.unwrap_or(0);

        // Check if stash exists
        self.check_stash_exists(repo, index)?;

        let mut apply_options = StashApplyOptions::default();

        repo.stash_apply(index, Some(&mut apply_options))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to apply stash at index {}: {}", index, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        println!("Applied stash at index {}", index);
        Ok(true)
    }

    fn list_stashes_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        let mut stash_count = 0;
        let mut callback = |index: usize, message: &str, _oid: &git2::Oid| -> bool {
            println!("stash@{{{}}}: {}", index, message);
            stash_count += 1;
            true
        };

        repo.stash_foreach(&mut callback).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to list stashes: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        if stash_count == 0 {
            println!("No stashes found.");
        }

        Ok(true)
    }

    fn drop_stash_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        let index = self.stash_index.unwrap_or(0);

        // Check if stash exists
        self.check_stash_exists(repo, index)?;

        repo.stash_drop(index).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to drop stash at index {}: {}", index, e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        println!("Dropped stash at index {}", index);
        Ok(true)
    }

    fn clear_stashes_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        let mut stash_count = 0;
        let mut callback = |_index: usize, _message: &str, _oid: &git2::Oid| -> bool {
            stash_count += 1;
            true
        };

        repo.stash_foreach(&mut callback).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to count stashes: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        if stash_count == 0 {
            println!("No stashes to clear.");
            return Ok(true);
        }

        // Drop all stashes (starting from the highest index)
        for i in (0..stash_count).rev() {
            repo.stash_drop(i).map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to drop stash at index {}: {}", i, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;
        }

        println!("Cleared all {} stashes", stash_count);
        Ok(true)
    }

    fn check_stash_exists(
        &self,
        repo: &mut Repository,
        index: usize,
    ) -> Result<(), Box<BGitError>> {
        let mut stash_exists = false;
        let mut callback = |stash_index: usize, _message: &str, _oid: &git2::Oid| -> bool {
            if stash_index == index {
                stash_exists = true;
                false // Stop iteration
            } else {
                true // Continue
            }
        };

        repo.stash_foreach(&mut callback).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to check stash existence: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        if !stash_exists {
            return Err(Box::new(BGitError::new(
                "BGitError",
                &format!("Stash at index {} does not exist", index),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            )));
        }

        Ok(())
    }
}
