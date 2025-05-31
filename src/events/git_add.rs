use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use git2::{IndexAddOption, Repository};
use std::path::Path;

pub(crate) struct GitAdd {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub enum AddMode {
    All,
    Selective,
}

impl AtomicEvent for GitAdd {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitAdd {
            name: "git_add".to_owned(),
            pre_check_rules: vec![],
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
        // Get list of unstaged files
        let unstaged_files = super::git_status::get_unstaged_files_list()?;

        if unstaged_files.is_empty() {
            println!("No unstaged files found.");
            return Ok(false);
        }

        // Ask user to choose between adding all files or selecting specific files
        let add_mode = self.prompt_add_mode()?;

        match add_mode {
            AddMode::All => self.add_all_files(),
            AddMode::Selective => self.add_selective_files(unstaged_files),
        }
    }
}

impl GitAdd {
    /// Prompt user to choose between adding all files or selecting specific files
    fn prompt_add_mode(&self) -> Result<AddMode, Box<BGitError>> {
        let options = vec!["Add all unstaged files", "Select specific files to add"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose add mode:")
            .default(0)
            .items(&options)
            .interact()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get user selection: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        match selection {
            0 => Ok(AddMode::All),
            1 => Ok(AddMode::Selective),
            _ => Ok(AddMode::All),
        }
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

    /// Allow user to select specific files to add to staging area
    fn add_selective_files(
        &self,
        unstaged_files: Vec<super::git_status::FileStatus>,
    ) -> Result<bool, Box<BGitError>> {
        // Create display strings for the files
        let file_display: Vec<String> = unstaged_files
            .iter()
            .map(|file| format!("{} ({})", file.path, file.status_type))
            .collect();

        if file_display.is_empty() {
            println!("No files to select.");
            return Ok(false);
        }

        // Let user select multiple files
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select files to add (use Space to select, Enter to confirm):")
            .items(&file_display)
            .interact()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get file selections: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        if selections.is_empty() {
            println!("No files selected.");
            return Ok(false);
        }

        // Get the selected file paths
        let selected_files: Vec<&str> = selections
            .iter()
            .map(|&i| unstaged_files[i].path.as_str())
            .collect();

        // Add selected files to staging area
        self.add_specific_files(selected_files)?;

        println!(
            "Successfully added {} file(s) to staging area.",
            selections.len()
        );
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

        // Add each selected file to the index
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

        Ok(())
    }
}
