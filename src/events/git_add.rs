use super::AtomicEvent;
use crate::{bgit_error::BGitError, config::global::BGitGlobalConfig, rules::Rule};
use git2::{IndexAddOption, Repository};
use std::path::Path;

pub(crate) struct GitAdd<'a> {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    add_mode: Option<AddMode>,
    _global_config: &'a BGitGlobalConfig,
}

#[derive(Debug, Clone)]
pub enum AddMode {
    All,
    Selective(Vec<String>),
}

impl<'a> AtomicEvent<'a> for GitAdd<'a> {
    fn new(_global_config: &'a BGitGlobalConfig) -> Self
    where
        Self: Sized,
    {
        GitAdd {
            name: "git_add".to_owned(),
            pre_check_rules: vec![],
            add_mode: None,
            _global_config,
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
            Some(AddMode::All) => self.add_all_files(),
            Some(AddMode::Selective(selected_files)) => {
                if selected_files.is_empty() {
                    return Err(self.to_bgit_error("No files selected for staging."));
                }
                self.add_specific_files(selected_files.iter().map(|s| s.as_str()).collect())?;
                println!(
                    "Successfully added {} file(s) to staging area.",
                    selected_files.len()
                );
                Ok(true)
            }
            None => {
                Err(self.to_bgit_error("No add mode specified. Use 'with_add_mode' to set it."))
            }
        }
    }
}

impl<'a> GitAdd<'a> {
    pub fn with_add_mode(mut self, mode: AddMode) -> Self {
        self.add_mode = Some(mode);
        self
    }

    /// Add all unstaged files to staging area
    fn add_all_files(&self) -> Result<bool, Box<BGitError>> {
        // Open the repository at the current directory
        let repo = Repository::discover(Path::new("."))
            .map_err(|e| self.to_bgit_error(&format!("Failed to open repository: {e}")))?;

        // Get the repository index
        let mut index = repo
            .index()
            .map_err(|e| self.to_bgit_error(&format!("Failed to get repository index: {e}")))?;

        // Using ["."], which indicates the current directory recursively.
        index
            .add_all(["."], IndexAddOption::DEFAULT, None)
            .map_err(|e| self.to_bgit_error(&format!("Failed to add files to index: {e}")))?;

        // Write the index changes to disk
        index
            .write()
            .map_err(|e| self.to_bgit_error(&format!("Failed to write index: {e}")))?;

        Ok(true)
    }

    /// Add specific files to the staging area
    fn add_specific_files(&self, file_paths: Vec<&str>) -> Result<(), Box<BGitError>> {
        // Open the repository at the current directory
        let repo = Repository::discover(Path::new("."))
            .map_err(|e| self.to_bgit_error(&format!("Failed to open repository: {e}")))?;

        let mut index = repo
            .index()
            .map_err(|e| self.to_bgit_error(&format!("Failed to get repository index: {e}")))?;

        for file_path in file_paths {
            index.add_path(Path::new(file_path)).map_err(|e| {
                self.to_bgit_error(&format!("Failed to add file '{file_path}' to index: {e}"))
            })?;
        }

        index
            .write()
            .map_err(|e| self.to_bgit_error(&format!("Failed to write index: {e}")))?;

        Ok(())
    }
}
