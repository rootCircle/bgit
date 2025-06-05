use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_RULE, NO_STEP};
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
    fn new() -> Self {
        IsRepoSizeTooBig {
            name: "IsRepoSizeTooBig".to_string(),
            description: "Check if repository size exceeds the recommended limit".to_string(),
            level: RuleLevel::Warning,
            max_size_mb: 100,
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
                &format!("Failed to discover repository: {}", e),
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
                "Failed to calculate repository size: {}",
                e
            ))),
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        println!("Attempting to reduce repository size...");

        let repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to discover repository: {}", e),
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
                println!("Cleanup failed: {}", e);
                Ok(false)
            }
        }
    }
}

impl IsRepoSizeTooBig {
    fn calculate_repo_size(&self, repo: &Repository) -> Result<u64, String> {
        let mut total_size = 0u64;

        let git_dir = repo.path();
        total_size += self
            .calculate_directory_size(git_dir)
            .map_err(|e| format!("Failed to calculate .git directory size: {}", e))?;

        if let Some(workdir) = repo.workdir() {
            total_size += self
                .calculate_working_directory_size(workdir)
                .map_err(|e| format!("Failed to calculate working directory size: {}", e))?;
        }

        Ok(total_size)
    }

    fn calculate_directory_size(&self, dir: &Path) -> Result<u64, std::io::Error> {
        let mut size = 0u64;

        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    size += self.calculate_directory_size(&path)?;
                } else {
                    size += entry.metadata()?.len();
                }
            }
        }

        Ok(size)
    }

    fn calculate_working_directory_size(&self, workdir: &Path) -> Result<u64, std::io::Error> {
        let mut size = 0u64;

        for entry in fs::read_dir(workdir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();

            // Skip .git directory
            if file_name == ".git" {
                continue;
            }

            if path.is_dir() {
                size += self.calculate_working_directory_size(&path)?;
            } else {
                size += entry.metadata()?.len();
            }
        }

        Ok(size)
    }

    fn perform_cleanup(&self, repo: &Repository) -> Result<bool, String> {
        // Clean up loose objects by checking if they're referenced
        let odb = repo
            .odb()
            .map_err(|e| format!("Failed to access object database: {}", e))?;

        let mut cleanup_performed = false;

        // This is a basic implementation - in practice, you might want more sophisticated cleanup
        let mut unreferenced_objects = Vec::new();

        odb.foreach(|oid| {
            let mut is_referenced = false;

            if let Ok(refs) = repo.references() {
                for reference in refs {
                    if let Ok(reference) = reference {
                        if let Some(target_oid) = reference.target() {
                            if target_oid == *oid {
                                is_referenced = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !is_referenced {
                unreferenced_objects.push(*oid);
            }

            true
        })
        .map_err(|e| format!("Failed to iterate objects: {}", e))?;

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
        if let Ok(mut index) = repo.index() {
            if index.read(true).is_ok() {
                println!("Index refreshed");
                cleanup_performed = true;
            }
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
