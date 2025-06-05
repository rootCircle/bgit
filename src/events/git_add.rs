use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::{IndexAddOption, Repository};
use std::path::Path;

pub(crate) struct GitAdd {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    add_mode: AddMode,
}

#[derive(Debug, Clone)]
pub enum AddMode {
    All,
    Selective(Vec<String>),
}

impl AtomicEvent for GitAdd {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitAdd {
            name: "git_add".to_owned(),
            pre_check_rules: vec![],
            add_mode: AddMode::All,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        "Add files to staging area"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        match &self.add_mode {
            AddMode::All => self.add_all_files(),
            AddMode::Selective(selected_files) => {
                if selected_files.is_empty() {
                    println!("No files selected.");
                    return Ok(false);
                }
                self.add_specific_files(selected_files.iter().map(|s| s.as_str()).collect())?;
                println!(
                    "Successfully added {} file(s) to staging area.",
                    selected_files.len()
                );
                Ok(true)
            }
        }
    }
}

impl GitAdd {
    pub fn with_add_mode(mut self, mode: AddMode) -> Self {
        self.add_mode = mode;
        self
    }

    /// Add all unstaged files to staging area
    fn add_all_files(&self) -> Result<bool, Box<BGitError>> {
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

        // Get the repository index
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

        // Using ["."], which indicates the current directory recursively.
        index
            .add_all(["."], IndexAddOption::DEFAULT, None)
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to add files to index: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        // Write the index changes to disk
        index.write().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to write index: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        println!("All unstaged files have been added to staging area.");
        Ok(true)
    }

    /// Add specific files to the staging area
    fn add_specific_files(&self, file_paths: Vec<&str>) -> Result<(), Box<BGitError>> {
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

        for file_path in file_paths {
            index.add_path(Path::new(file_path)).map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to add file '{}' to index: {}", file_path, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;
        }

        index.write().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to write index: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        Ok(())
    }
}
