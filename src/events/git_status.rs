use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::{Repository, Status, StatusOptions};
use std::path::Path;

pub(crate) struct GitStatus {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    mode: StatusMode,
}

#[derive(Debug, Clone)]
pub struct FileStatus {
    pub path: String,
    pub status_type: String,
}

#[derive(Debug, Clone)]
pub enum StatusMode {
    CheckOnly,
    #[allow(dead_code)]
    ListFiles,
}

impl AtomicEvent for GitStatus {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitStatus {
            name: "git_status".to_owned(),
            pre_check_rules: vec![],
            mode: StatusMode::CheckOnly, // Default mode
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        "Show the working tree status"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        match self.mode {
            StatusMode::CheckOnly => {
                let has_files = has_unstaged_or_new_files()?;
                if has_files {
                    println!("You have unstaged or new files.");
                } else {
                    println!("No unstaged or new files found.");
                }
                Ok(has_files)
            }
            StatusMode::ListFiles => {
                let files = get_unstaged_files_list()?;
                if files.is_empty() {
                    println!("No unstaged or new files found.");
                    Ok(false)
                } else {
                    println!("Unstaged and new files:");
                    for file in &files {
                        println!("  {} ({})", file.path, file.status_type);
                    }
                    Ok(true)
                }
            }
        }
    }
}

impl GitStatus {
    /// Create a new GitStatus with specific mode
    #[allow(dead_code)]
    pub fn with_mode(mode: StatusMode) -> Self {
        GitStatus {
            name: "git_status".to_owned(),
            pre_check_rules: vec![],
            mode,
        }
    }

    /// Get list of unstaged and new files with their status
    #[allow(dead_code)]
    pub fn get_unstaged_files(&self) -> Result<Vec<FileStatus>, Box<BGitError>> {
        get_unstaged_files_list()
    }
}

#[allow(dead_code)]
fn get_current_branch(repo: &Repository) -> Result<String, Box<BGitError>> {
    match repo.head() {
        Ok(head) => {
            if let Some(branch_name) = head.shorthand() {
                Ok(branch_name.to_string())
            } else {
                Ok("HEAD".to_string())
            }
        }
        Err(_) => {
            // Repository might be in initial state with no commits
            Ok("main".to_string())
        }
    }
}

/// Detects unstaged files (modified tracked files) or new files (untracked)
pub fn has_unstaged_or_new_files() -> Result<bool, Box<BGitError>> {
    let repo = Repository::discover(Path::new(".")).map_err(|e| {
        Box::new(BGitError::new(
            "BGitError",
            &format!("Failed to open repository: {}", e),
            BGitErrorWorkflowType::AtomicEvent,
            NO_EVENT,
            "has_unstaged_or_new_files",
            NO_RULE,
        ))
    })?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| {
        Box::new(BGitError::new(
            "BGitError",
            &format!("Failed to get repository status: {}", e),
            BGitErrorWorkflowType::AtomicEvent,
            NO_EVENT,
            "has_unstaged_or_new_files",
            NO_RULE,
        ))
    })?;

    for entry in statuses.iter() {
        let status = entry.status();

        // Check for working tree changes (unstaged)
        if status.intersects(
            Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_TYPECHANGE
                | Status::WT_RENAMED
                | Status::WT_NEW, // This includes untracked files
        ) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Get list of unstaged and new files with their status descriptions
pub fn get_unstaged_files_list() -> Result<Vec<FileStatus>, Box<BGitError>> {
    let repo = Repository::discover(Path::new(".")).map_err(|e| {
        Box::new(BGitError::new(
            "BGitError",
            &format!("Failed to open repository: {}", e),
            BGitErrorWorkflowType::AtomicEvent,
            NO_EVENT,
            "get_unstaged_files_list",
            NO_RULE,
        ))
    })?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| {
        Box::new(BGitError::new(
            "BGitError",
            &format!("Failed to get repository status: {}", e),
            BGitErrorWorkflowType::AtomicEvent,
            NO_EVENT,
            "get_unstaged_files_list",
            NO_RULE,
        ))
    })?;

    let mut unstaged_files = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();
        let file_path = entry.path().unwrap_or("").to_string();

        // Check for working tree changes (unstaged)
        if status.intersects(
            Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_TYPECHANGE
                | Status::WT_RENAMED
                | Status::WT_NEW,
        ) {
            let status_type = match status {
                s if s.contains(Status::WT_NEW) => "New file",
                s if s.contains(Status::WT_MODIFIED) => "Modified",
                s if s.contains(Status::WT_DELETED) => "Deleted",
                s if s.contains(Status::WT_TYPECHANGE) => "Type changed",
                s if s.contains(Status::WT_RENAMED) => "Renamed",
                _ => "Unknown",
            }
            .to_string();

            unstaged_files.push(FileStatus {
                path: file_path,
                status_type,
            });
        }
    }

    Ok(unstaged_files)
}

/// Detects only staged files (changes ready to be committed)
#[allow(dead_code)]
pub fn has_staged_files() -> Result<bool, Box<BGitError>> {
    let repo = Repository::discover(Path::new(".")).map_err(|e| {
        Box::new(BGitError::new(
            "BGitError",
            &format!("Failed to open repository: {}", e),
            BGitErrorWorkflowType::AtomicEvent,
            NO_EVENT,
            "has_staged_files",
            NO_RULE,
        ))
    })?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| {
        Box::new(BGitError::new(
            "BGitError",
            &format!("Failed to get repository status: {}", e),
            BGitErrorWorkflowType::AtomicEvent,
            NO_EVENT,
            "has_staged_files",
            NO_RULE,
        ))
    })?;

    for entry in statuses.iter() {
        let status = entry.status();

        // Check for staged changes (index)
        if status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Detects only new files (untracked files)
#[allow(dead_code)]
pub fn has_new_files() -> Result<bool, Box<BGitError>> {
    let repo = Repository::discover(Path::new(".")).map_err(|e| {
        Box::new(BGitError::new(
            "BGitError",
            &format!("Failed to open repository: {}", e),
            BGitErrorWorkflowType::AtomicEvent,
            NO_EVENT,
            "has_new_files",
            NO_RULE,
        ))
    })?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| {
        Box::new(BGitError::new(
            "BGitError",
            &format!("Failed to get repository status: {}", e),
            BGitErrorWorkflowType::AtomicEvent,
            NO_EVENT,
            "has_new_files",
            NO_RULE,
        ))
    })?;

    for entry in statuses.iter() {
        let status = entry.status();

        // Check for untracked files only
        if status.contains(Status::WT_NEW) {
            return Ok(true);
        }
    }

    Ok(false)
}
