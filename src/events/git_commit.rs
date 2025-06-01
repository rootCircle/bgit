use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::{Repository, Signature};
use std::path::Path;

pub(crate) struct GitCommit {
    name: String,
    commit_message: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
}

impl AtomicEvent for GitCommit {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitCommit {
            name: "git_commit".to_owned(),
            commit_message: String::new(),
            pre_check_rules: vec![],
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        "Commit staged files with auto-generated message"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        // Check if there are any staged files
        if !self.has_staged_files()? {
            println!("No staged files found. Nothing to commit.");
            return Ok(false);
        }

        // Perform the commit with the provided message
        self.commit_changes(&self.commit_message)?;

        println!(
            "Successfully committed with message: \"{}\"",
            self.commit_message
        );
        Ok(true)
    }
}

impl GitCommit {
    /// Create a new GitCommit with a specific commit message
    pub fn with_message(commit_message: String) -> Self {
        GitCommit {
            name: "git_commit".to_owned(),
            commit_message,
            pre_check_rules: vec![],
        }
    }

    /// Check if there are any staged files ready to commit
    fn has_staged_files(&self) -> Result<bool, Box<BGitError>> {
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

        let index = repo.index().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get repository index: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Check if index has any entries (staged files)
        Ok(index.len() > 0)
    }

    /// Commit the staged changes with the provided message
    fn commit_changes(&self, message: &str) -> Result<(), Box<BGitError>> {
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

        // Get the current signature (author/committer)
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

        // Get the current HEAD commit
        let head = repo.head().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get HEAD reference: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        let parent_commit = head.peel_to_commit().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get HEAD commit: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Get the repository index and create a tree from it
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

        let tree_id = index.write_tree().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to write tree: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        let tree = repo.find_tree(tree_id).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to find tree: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Create the commit
        repo.commit(
            Some("HEAD"),      // Update HEAD
            &signature,        // Author
            &signature,        // Committer
            message,           // Commit message
            &tree,             // Tree
            &[&parent_commit], // Parents
        )
        .map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to create commit: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        Ok(())
    }
}
