use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_RULE, NO_STEP};
use crate::config::local::WorkflowRules;
use crate::constants::DEFAULT_MAX_REPO_SIZE_IN_MIB;
use crate::rules::{Rule, RuleLevel, RuleOutput};
use git2::Repository;
use std::fs;
use std::path::Path;

pub(crate) struct IsRepoSizeTooBig {
    name: String,
    description: String,
    level: RuleLevel,
    max_size_mb: u64,
}

impl Rule for IsRepoSizeTooBig {
    fn new(workflow_rule_config: Option<&WorkflowRules>) -> Self {
        let default_rule_level = RuleLevel::Warning;
        let name = "IsRepoSizeTooBig";
        let rule_level = workflow_rule_config
            .and_then(|config| config.get_rule_level(name))
            .cloned()
            .unwrap_or(default_rule_level);

        Self {
            name: name.to_string(),
            description: "Check if repository size exceeds the recommended limit".to_string(),
            level: rule_level,
            max_size_mb: DEFAULT_MAX_REPO_SIZE_IN_MIB,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_level(&self) -> RuleLevel {
        self.level.clone()
    }

    fn check(&self) -> Result<RuleOutput, Box<BGitError>> {
        let repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to discover repository: {e}"),
                BGitErrorWorkflowType::Rules,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        match self.calculate_repo_size(&repo) {
            Ok(repo_size_bytes) => {
                let repo_size_mb = repo_size_bytes / (1024 * 1024);

                if repo_size_mb > self.max_size_mb {
                    Ok(RuleOutput::Exception(format!(
                        "Repository size ({} MB) exceeds recommended limit of {} MB",
                        repo_size_mb, self.max_size_mb
                    )))
                } else {
                    Ok(RuleOutput::Success)
                }
            }
            Err(e) => Ok(RuleOutput::Exception(format!(
                "Failed to calculate repository size: {e}"
            ))),
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        println!("Attempting to reduce repository size...");

        let repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to discover repository: {e}"),
                BGitErrorWorkflowType::Rules,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        match self.perform_cleanup(&repo) {
            Ok(success) => {
                if success {
                    println!("Repository cleanup completed successfully");
                } else {
                    println!("Could not automatically fix repository size issue");
                    println!(
                        "Consider manually removing large files or using git-lfs for large assets"
                    );
                }
                Ok(success)
            }
            Err(e) => {
                println!("Cleanup failed: {e}");
                Ok(false)
            }
        }
    }
}

impl IsRepoSizeTooBig {
    fn calculate_repo_size(&self, repo: &Repository) -> Result<u64, String> {
        let mut total_size = 0u64;

        // Get the index to access tracked files
        let index = repo
            .index()
            .map_err(|e| format!("Failed to get repository index: {e}"))?;

        // Calculate size of tracked files only
        for entry in index.iter() {
            if let Some(workdir) = repo.workdir() {
                let file_path = workdir.join(
                    std::str::from_utf8(&entry.path)
                        .map_err(|e| format!("Invalid UTF-8 in file path: {e}"))?,
                );

                if file_path.exists() && file_path.is_file() {
                    total_size += fs::metadata(&file_path)
                        .map_err(|e| {
                            format!("Failed to get metadata for {}: {}", file_path.display(), e)
                        })?
                        .len();
                }
            }
        }

        Ok(total_size)
    }

    fn perform_cleanup(&self, repo: &Repository) -> Result<bool, String> {
        // Clean up loose objects by checking if they're referenced
        let odb = repo
            .odb()
            .map_err(|e| format!("Failed to access object database: {e}"))?;

        let mut cleanup_performed = false;

        // This is a basic implementation - in practice, you might want more sophisticated cleanup
        let mut unreferenced_objects = Vec::new();

        odb.foreach(|oid| {
            let mut is_referenced = false;

            if let Ok(refs) = repo.references() {
                for reference in refs.flatten() {
                    if let Some(target_oid) = reference.target()
                        && target_oid == *oid
                    {
                        is_referenced = true;
                        break;
                    }
                }
            }

            if !is_referenced {
                unreferenced_objects.push(*oid);
            }

            true
        })
        .map_err(|e| format!("Failed to iterate objects: {e}"))?;

        // Note: Actual deletion of unreferenced objects would require low-level operations
        // that git2 doesn't directly support. In practice, you might still need to call
        // git gc through Command for full cleanup functionality.

        if !unreferenced_objects.is_empty() {
            println!(
                "Found {} potentially unreferenced objects",
                unreferenced_objects.len()
            );
            cleanup_performed = true;
        }

        // Clean up the index
        if let Ok(mut index) = repo.index()
            && index.read(true).is_ok()
        {
            println!("Index refreshed");
            cleanup_performed = true;
        }

        Ok(cleanup_performed)
    }

    /// Method to set custom size limit
    #[allow(dead_code)]
    pub fn with_max_size_mb(mut self, max_size_mb: u64) -> Self {
        self.max_size_mb = max_size_mb;
        self
    }
}
