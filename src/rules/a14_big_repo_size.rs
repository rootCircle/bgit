use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_STEP};
use crate::rules::{Rule, RuleLevel, RuleOutput};
use std::path::Path;
use std::process::Command;

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
            max_size_mb: 100, // Default 100MB limit
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
        // Check if we're in a git repository
        if !Path::new(".git").exists() {
            return Ok(RuleOutput::Exception("Not in a git repository".to_string()));
        }

        // Get repository size using git count-objects
        let output = Command::new("git").arg("count-objects").arg("-vH").output();

        match output {
            Err(e) => Ok(RuleOutput::Exception(format!(
                "Failed to execute git command: {}",
                e
            ))),
            Ok(output_response) => {
                if !output_response.status.success() {
                    return Ok(RuleOutput::Exception(
                        "Failed to get repository size".to_string(),
                    ));
                }

                let output_str = String::from_utf8_lossy(&output_response.stdout);
                let repo_size_bytes = self.parse_repo_size(&output_str)?;
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
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        println!("Attempting to reduce repository size...");

        // Try to run git gc (garbage collection) to compress and clean up
        println!("Running git gc --aggressive --prune=now");
        let gc_output = Command::new("git")
            .arg("gc")
            .arg("--aggressive")
            .arg("--prune=now")
            .output();

        match gc_output {
            Err(e) => Err(Box::new(BGitError::new(
                "Failed to execute git gc command",
                &e.to_string(),
                BGitErrorWorkflowType::Rules,
                NO_STEP,
                NO_EVENT,
                self.get_name(),
            ))),
            Ok(gc_response) => {
                if !gc_response.status.success() {
                    println!("Git gc failed, trying alternative cleanup methods...");

                    // Try to clean up untracked files
                    println!("Cleaning untracked files with git clean -fd");
                    let clean_output = Command::new("git").arg("clean").arg("-fd").output();

                    match clean_output {
                        Err(_) => Ok(false),
                        Ok(clean_response) => {
                            if clean_response.status.success() {
                                println!("Repository cleanup completed partially");
                                Ok(true)
                            } else {
                                println!("Could not automatically fix repository size issue");
                                println!("Consider manually removing large files or using git-lfs for large assets");
                                Ok(false)
                            }
                        }
                    }
                } else {
                    println!("Git garbage collection completed successfully");
                    Ok(true)
                }
            }
        }
    }
}

impl IsRepoSizeTooBig {
    // Helper method to parse repository size from git count-objects output
    fn parse_repo_size(&self, output: &str) -> Result<u64, Box<BGitError>> {
        for line in output.lines() {
            if line.starts_with("size-pack") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    match parts[1].parse::<u64>() {
                        Ok(size) => return Ok(size),
                        Err(_) => continue,
                    }
                }
            }
        }

        // Fallback: try to get size from "size" field
        for line in output.lines() {
            if line.starts_with("size ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    match parts[1].parse::<u64>() {
                        Ok(size) => return Ok(size * 1024), // Convert KB to bytes
                        Err(_) => continue,
                    }
                }
            }
        }

        Err(Box::new(BGitError::new(
            "Could not parse repository size from git output",
            output,
            BGitErrorWorkflowType::Rules,
            NO_STEP,
            NO_EVENT,
            self.get_name(),
        )))
    }

    // Method to set custom size limit
    #[allow(dead_code)]
    pub fn with_max_size_mb(mut self, max_size_mb: u64) -> Self {
        self.max_size_mb = max_size_mb;
        self
    }
}
