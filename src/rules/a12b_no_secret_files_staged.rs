use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_STEP};
use crate::rules::{Rule, RuleLevel, RuleOutput};
use git2::{Repository, Status, StatusOptions};
use log::{info, warn};
use regex::Regex;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub(crate) struct NoSecretFilesStaged {
    name: String,
    description: String,
    level: RuleLevel,
}

impl Rule for NoSecretFilesStaged {
    fn new() -> Self {
        NoSecretFilesStaged {
            name: "NoSecretFilesStaged".to_string(),
            description: "Prevent staging of files that might contain secrets".to_string(),
            level: RuleLevel::Error,
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
        let repo = match Repository::open(".") {
            Ok(repo) => repo,
            Err(e) => {
                return Ok(RuleOutput::Exception(format!(
                    "Failed to open repository: {}",
                    e
                )));
            }
        };

        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = match repo.statuses(Some(&mut status_options)) {
            Ok(statuses) => statuses,
            Err(e) => {
                return Ok(RuleOutput::Exception(format!(
                    "Failed to get repository status: {}",
                    e
                )));
            }
        };

        let secret_patterns = self.get_secret_file_patterns();
        let mut found_secrets = Vec::new();

        for entry in statuses.iter() {
            let file_path = match entry.path() {
                Some(path) => path,
                None => continue,
            };

            let status = entry.status();

            // Check if file is staged or modified (but not ignored)
            if (status.contains(Status::INDEX_NEW)
                || status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::WT_NEW)
                || status.contains(Status::WT_MODIFIED))
                && self.is_secret_file(file_path, &secret_patterns)
            {
                found_secrets.push(file_path.to_string());
            }
        }

        if found_secrets.is_empty() {
            Ok(RuleOutput::Success)
        } else {
            Ok(RuleOutput::Exception(format!(
                "Potential secret files detected: {}",
                found_secrets.join(", ")
            )))
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        let repo = match Repository::open(".") {
            Ok(repo) => repo,
            Err(e) => {
                return Err(Box::new(BGitError::new(
                    "Failed to open repository",
                    &e.to_string(),
                    BGitErrorWorkflowType::Rules,
                    NO_STEP,
                    NO_EVENT,
                    self.get_name(),
                )));
            }
        };

        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = match repo.statuses(Some(&mut status_options)) {
            Ok(statuses) => statuses,
            Err(e) => {
                return Err(Box::new(BGitError::new(
                    "Failed to get repository status",
                    &e.to_string(),
                    BGitErrorWorkflowType::Rules,
                    NO_STEP,
                    NO_EVENT,
                    self.get_name(),
                )));
            }
        };

        let secret_patterns = self.get_secret_file_patterns();
        let mut files_to_ignore = Vec::new();

        for entry in statuses.iter() {
            let file_path = match entry.path() {
                Some(path) => path,
                None => continue,
            };

            let status = entry.status();

            if (status.contains(Status::INDEX_NEW)
                || status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::WT_NEW)
                || status.contains(Status::WT_MODIFIED))
                && self.is_secret_file(file_path, &secret_patterns)
            {
                files_to_ignore.push(file_path.to_string());
            }
        }

        if files_to_ignore.is_empty() {
            return Ok(true);
        }

        // Add files to .gitignore
        match self.add_to_gitignore(&files_to_ignore) {
            Ok(_) => {
                info!("Added the following files to .gitignore:");
                for file in &files_to_ignore {
                    info!("  {}", file);
                }

                // Unstage files if they were staged
                if let Err(e) = self.unstage_files(&repo, &files_to_ignore) {
                    warn!("Warning: Failed to unstage some files: {}", e);
                }

                Ok(true)
            }
            Err(e) => Err(Box::new(BGitError::new(
                "Failed to add files to .gitignore",
                &e.to_string(),
                BGitErrorWorkflowType::Rules,
                NO_STEP,
                NO_EVENT,
                self.get_name(),
            ))),
        }
    }
}

impl NoSecretFilesStaged {
    fn get_secret_file_patterns(&self) -> Vec<Regex> {
        let patterns = vec![
            r"^\.env$",
            r"^\.env\..*",
            r".*\.pem$",
            r".*\.key$",
            r".*\.p12$",
            r".*\.pfx$",
            r"^id_rsa$",
            r"^id_dsa$",
            r"^id_ecdsa$",
            r"^id_ed25519$",
            r".*_rsa$",
            r".*_dsa$",
            r".*_ecdsa$",
            r".*_ed25519$",
            r"^\.ssh/.*",
            r"^config/secrets\..*",
            r"^secrets\..*",
            r".*secret.*\.json$",
            r".*secret.*\.yaml$",
            r".*secret.*\.yml$",
            r".*credentials.*\.json$",
            r".*credentials.*\.yaml$",
            r".*credentials.*\.yml$",
            r"^\.aws/credentials$",
            r"^\.aws/config$",
            r"^\.docker/config\.json$",
            r".*\.token$",
            r".*\.password$",
        ];

        patterns
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect()
    }

    fn is_secret_file(&self, file_path: &str, patterns: &[Regex]) -> bool {
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(file_path);

        patterns
            .iter()
            .any(|pattern| pattern.is_match(file_path) || pattern.is_match(file_name))
    }

    fn add_to_gitignore(&self, files: &[String]) -> Result<(), std::io::Error> {
        let repo_root = match Repository::open(".") {
            Ok(repo) => repo
                .workdir()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| Path::new(".").to_path_buf()),
            Err(_) => Path::new(".").to_path_buf(),
        };
        let gitignore_path = repo_root.join(".gitignore");

        // Read existing .gitignore content
        let existing_entries = if gitignore_path.exists() {
            let file = std::fs::File::open(&gitignore_path)?;
            let reader = BufReader::new(file);
            reader.lines().collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };

        // Open .gitignore in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&gitignore_path)?;

        // Add new entries that don't already exist
        for file_path in files {
            if !existing_entries.contains(file_path) {
                writeln!(file, "{}", file_path)?;
            }
        }

        Ok(())
    }

    fn unstage_files(&self, repo: &Repository, files: &[String]) -> Result<(), git2::Error> {
        let mut index = repo.index()?;

        for file_path in files {
            // Try to remove from index if it exists
            if index.get_path(Path::new(file_path), 0).is_some() {
                index.remove_path(Path::new(file_path))?;
            }
        }

        index.write()?;
        Ok(())
    }
}
